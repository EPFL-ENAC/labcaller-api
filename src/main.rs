mod config;
mod s3;

use crate::s3::upload::upload_stream;
use config::Config;

#[tokio::main]
pub async fn main() {
    // Example usage with a file
    let config = Config::from_env();

    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: {} <file_path>", args[0]);
        std::process::exit(1);
    }
    let file_path = &args[1];
    let file_name = file_path.split('/').last().unwrap();

    // Open the file and create a stream
    let file = tokio::fs::File::open(file_path)
        .await
        .expect("Failed to open file");
    let stream = tokio::io::BufReader::new(file);

    // Call the upload function with the stream and key
    upload_stream(stream, file_name, &config).await;
}
