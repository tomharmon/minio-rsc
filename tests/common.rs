use minio_rsc::{provider::StaticProvider, Minio};
use minio_rsc::errors::Result;

pub fn get_test_minio() -> Minio {
    let provider = StaticProvider::new("minio-access-key-test", "minio-secret-key-test", None);
    Minio::builder()
        .endpoint("localhost:9022")
        .provider(provider)
        .virtual_hosted_style(true)
        .multi_chunked_encoding(true)
        .secure(false)
        .build()
        .unwrap()
}

pub async fn create_bucket_if_not_exist(minio: &Minio, bucket_name: &str)-> Result<()> {
    let exists = minio.bucket_exists(bucket_name).await?;
    if !exists {
        minio.make_bucket(bucket_name, false).await?;
    }
    return Ok(());
}
