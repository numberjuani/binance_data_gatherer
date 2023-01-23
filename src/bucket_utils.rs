use std::path::Path;

use aws_sdk_s3::{
    error::PutObjectError,
    types::{ByteStream, SdkError},
    Client,
};

///https://github.com/awslabs/aws-sdk-rust
/// Returns if there was success or not
pub async fn upload_object(
    bucket_name: &str,
    file_name: &str,
    key: &str,
) -> Result<bool, SdkError<PutObjectError>> {
    //Environment variables: AWS_ACCESS_KEY_ID, AWS_SECRET_ACCESS_KEY, and AWS_REGION are req'd for this next line to work
    let shared_config = aws_config::load_from_env().await;
    let client = Client::new(&shared_config);
    let body = ByteStream::from_path(Path::new(file_name)).await;
    match client
        .put_object()
        .bucket(bucket_name)
        .key(key)
        .body(body.unwrap())
        .send()
        .await
    {
        Ok(_) => {
            Ok(true)
        }
        Err(e) => {
            Err(e)
        }
    }
}
