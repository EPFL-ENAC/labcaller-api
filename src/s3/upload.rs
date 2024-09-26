use crate::config::Config;
use aws_config::BehaviorVersion;
use aws_sdk_s3::config::Credentials;
use aws_sdk_s3::operation::create_multipart_upload::CreateMultipartUploadOutput;
use aws_sdk_s3::types::{CompletedMultipartUpload, CompletedPart};
use aws_sdk_s3::{config::Region, Client as S3Client};
use aws_smithy_types::byte_stream::ByteStream;
use futures::future::join_all;
use std::sync::Arc;
use tokio::io::{AsyncRead, AsyncReadExt};
use tokio::sync::Semaphore;

const PART_SIZE: usize = 5 * 1024 * 1024; // Minimum part size for S3 multipart upload
const MAX_CONCURRENT_UPLOADS: usize = 10; // Limit the number of concurrent uploads

pub async fn upload_stream<R>(mut stream: R, file_key: &str, config: &Config)
where
    R: AsyncRead + Unpin + Send + 'static,
{
    let region = Region::new("us-east-1");
    let credentials = Credentials::new(
        &config.s3_access_key,
        &config.s3_secret_key,
        None,
        None,
        "manual",
    );
    let shared_config = aws_config::defaults(BehaviorVersion::latest())
        .region(region.clone())
        .credentials_provider(credentials)
        .endpoint_url(&config.s3_url)
        .load()
        .await;

    let client = Arc::new(S3Client::new(&shared_config));

    // Add the prefix to the key
    let key = format!("{}/{}", config.s3_prefix, file_key);

    // Start the multipart upload
    let multipart_upload_res: CreateMultipartUploadOutput = client
        .create_multipart_upload()
        .bucket(&config.s3_bucket)
        .key(&key)
        .send()
        .await
        .expect("Couldn't create multipart upload");

    let upload_id = multipart_upload_res.upload_id().unwrap().to_string();

    let semaphore = Arc::new(Semaphore::new(MAX_CONCURRENT_UPLOADS)); // Limit concurrency
    let mut upload_futures = Vec::new();
    let mut part_number = 1;
    let mut eof = false;

    while !eof {
        let client = Arc::clone(&client);
        let key = key.clone();
        let upload_id = upload_id.clone();
        let bucket = config.s3_bucket.clone();
        let permit = Arc::clone(&semaphore).acquire_owned().await.unwrap(); // Acquire semaphore permit

        let mut buffer = vec![0u8; PART_SIZE];
        let mut bytes_read = 0;

        // Read data into buffer
        while bytes_read < PART_SIZE {
            match stream.read(&mut buffer[bytes_read..]).await {
                Ok(0) => {
                    eof = true;
                    break;
                }
                Ok(n) => {
                    bytes_read += n;
                }
                Err(e) => {
                    eprintln!("Error reading stream: {}", e);
                    return;
                }
            }
        }

        if bytes_read == 0 {
            break; // No more data to read
        }

        let data = buffer[..bytes_read].to_vec();

        // Upload each part concurrently using tokio::spawn
        let upload_future = tokio::spawn(async move {
            let part = client
                .upload_part()
                .key(&key)
                .bucket(&bucket)
                .upload_id(&upload_id)
                .body(ByteStream::from(data))
                .part_number(part_number)
                .send()
                .await
                .expect("Couldn't upload part");

            drop(permit); // Release semaphore permit when the upload is done

            CompletedPart::builder()
                .e_tag(part.e_tag().unwrap_or_default())
                .part_number(part_number)
                .build()
        });

        upload_futures.push(upload_future);

        // Move to the next part
        part_number += 1;
    }

    // Wait for all upload parts to complete
    let completed_parts = join_all(upload_futures)
        .await
        .into_iter()
        .map(|result| result.unwrap())
        .collect::<Vec<CompletedPart>>();

    let completed_multipart_upload: CompletedMultipartUpload = CompletedMultipartUpload::builder()
        .set_parts(Some(completed_parts))
        .build();

    client
        .complete_multipart_upload()
        .bucket(&config.s3_bucket)
        .key(&key)
        .multipart_upload(completed_multipart_upload)
        .upload_id(&upload_id)
        .send()
        .await
        .expect("Couldn't complete multipart upload");
}
