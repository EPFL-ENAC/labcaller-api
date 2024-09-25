use aws_sdk_s3::config::Credentials;
use aws_sdk_s3::operation::create_multipart_upload::CreateMultipartUploadOutput;
use aws_sdk_s3::types::{CompletedMultipartUpload, CompletedPart};
use aws_sdk_s3::{config::Region, Client as S3Client};
use aws_smithy_types::byte_stream::{ByteStream, Length};
use config::Config;
use std::env;
use std::path::Path;

mod config;

const CHUNK_SIZE: u64 = 1024 * 1024 * 5;

#[tokio::main]
pub async fn main() {
    run_example().await
}

async fn run_example() -> () {
    let config = Config::from_env();

    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: {} <file_path>", args[0]);
        std::process::exit(1);
    }
    let file_path = &args[1];
    let file_name = file_path.split('/').last().unwrap();

    let region = Region::new("us-east-1");
    let credentials = Credentials::new(
        &config.s3_access_key,
        &config.s3_secret_key,
        None,
        None,
        "manual",
    );
    let shared_config = aws_config::from_env()
        .credentials_provider(credentials)
        .endpoint_url(&config.s3_url)
        .region(region)
        .load()
        .await;
    // let shared_config = aws_config::load_from_env().await;
    let client = S3Client::new(&shared_config);

    let key = format!("{}/{}", config.s3_prefix, file_name);

    let multipart_upload_res: CreateMultipartUploadOutput = client
        .create_multipart_upload()
        .bucket(&config.s3_bucket)
        .key(&key)
        .send()
        .await
        .expect("Couldn't create multipart upload");

    let upload_id = multipart_upload_res.upload_id().unwrap();

    let path = Path::new(&file_path);
    let file_size = tokio::fs::metadata(path)
        .await
        .expect("File not found")
        .len();

    let mut chunk_count = (file_size / CHUNK_SIZE) + 1;
    let mut size_of_last_chunk = file_size % CHUNK_SIZE;
    if size_of_last_chunk == 0 {
        size_of_last_chunk = CHUNK_SIZE;
        chunk_count -= 1;
    }

    // snippet-start:[s3.rust.upload_part]
    let mut upload_parts: Vec<aws_sdk_s3::types::CompletedPart> = Vec::new();

    for chunk_index in 0..chunk_count {
        let this_chunk = if chunk_count - 1 == chunk_index {
            size_of_last_chunk
        } else {
            CHUNK_SIZE
        };
        let stream = ByteStream::read_from()
            .path(path)
            .offset(chunk_index * CHUNK_SIZE)
            .length(Length::Exact(this_chunk))
            .build()
            .await
            .unwrap();

        // Chunk index needs to start at 0, but part numbers start at 1.
        let part_number = (chunk_index as i32) + 1;
        let upload_part_res = client
            .upload_part()
            .key(&key)
            .bucket(&config.s3_bucket)
            .upload_id(upload_id)
            .body(stream)
            .part_number(part_number)
            .send()
            .await
            .expect("Couldn't upload part");

        upload_parts.push(
            CompletedPart::builder()
                .e_tag(upload_part_res.e_tag.unwrap_or_default())
                .part_number(part_number)
                .build(),
        );
    }

    let completed_multipart_upload: CompletedMultipartUpload = CompletedMultipartUpload::builder()
        .set_parts(Some(upload_parts))
        .build();

    let _complete_multipart_upload_res = client
        .complete_multipart_upload()
        .bucket(&config.s3_bucket)
        .key(&key)
        .multipart_upload(completed_multipart_upload)
        .upload_id(upload_id)
        .send()
        .await
        .expect("Couldn't complete multipart upload");

    ()
}
