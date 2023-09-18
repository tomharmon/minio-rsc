mod common;

use common::get_test_minio;
use minio_rsc::error::Result;
use tokio;

pub const MIN_PART_SIZE: usize = 5 * 1024 * 1024; // 5MiB

#[tokio::main]
#[test]
async fn test_mutilpart_upload() -> Result<()> {
    dotenv::dotenv().ok();
    let minio = get_test_minio();
    let bucket = "test-mutilpart-upload";
    let object_key = "test.obj";

    minio.make_bucket(bucket, false).await?;

    let task = minio.create_multipart_upload(bucket, object_key).await?;

    let mut parts: Vec<minio_rsc::datatype::Part> = vec![];
    for i in 0..10 {
        let mut bytes = bytes::BytesMut::with_capacity(MIN_PART_SIZE);
        for _ in 0..MIN_PART_SIZE {
            bytes.extend_from_slice("A".as_bytes());
        }
        let p = minio.upload_part(&task, i + 1, bytes.freeze()).await?;
        parts.push(p);
    }
    minio.complete_multipart_upload(&task, parts, None).await?;

    let task = minio.create_multipart_upload(bucket, object_key).await?;
    minio.abort_multipart_upload(&task).await?;

    assert!(minio.remove_object(bucket, object_key).await.is_ok());
    assert!(minio.remove_bucket(bucket).await.is_ok());

    Ok(())
}
