mod common;
use common::get_test_minio;
use minio_rsc::errors::Result;
use minio_rsc::types::args::{BucketArgs, ObjectArgs, PresignedArgs};
use tokio;

#[tokio::main]
#[test]
async fn test_bucket() -> Result<()> {
    dotenv::dotenv().ok();
    let minio = get_test_minio();
    minio.make_bucket(BucketArgs::new("bucket-test1")).await?;
    assert!(minio.bucket_exists("bucket-test1").await.is_ok());
    assert!(minio.remove_bucket("bucket-test1").await.is_ok());
    println!("{:?}", minio.list_buckets().await.unwrap().0);
    println!("{:?}", minio.list_objects("file").await);
    Ok(())
}

#[tokio::main]
#[test]
async fn test_multi_upload() -> Result<()> {
    dotenv::dotenv().ok();

    let minio = get_test_minio();

    let bucket_name = "bucket-test1";
    let object_name = "/test/1.txt";
    let exists = minio.bucket_exists(bucket_name).await.is_ok();
    if !exists {
        minio.make_bucket(bucket_name).await?;
    }
    let multipart_upload = minio
        .create_multipart_upload(bucket_name, object_name, Some("text/plain"), None, None)
        .await?;
    let part1 = minio
        .upload_part(
            &multipart_upload,
            1,
            "test_multi_upload".as_bytes().to_vec(),
        )
        .await?;
    let reult = minio
        .complete_multipart_upload(&multipart_upload, vec![part1], None)
        .await?;

    Ok(())
}

#[tokio::main]
#[test]
async fn test_presigned() -> Result<()> {
    dotenv::dotenv().ok();
    let minio = get_test_minio();

    let url = minio
        .presigned_get_object(PresignedArgs::new("bucket", "file.txt").expires(24 * 3600))
        .await?;
    println!("{}", url);
    let url = minio
        .presigned_put_object(PresignedArgs::new("bucket", "file.txt").expires(24 * 3600))
        .await?;
    println!("{}", url);
    Ok(())
}

#[tokio::main]
#[test]
async fn test_operate_object() -> Result<()> {
    dotenv::dotenv().ok();
    let minio = get_test_minio();

    let bucket_name = "bucket-test1";
    let object_name = "/test/1.txt";
    let exists = minio.bucket_exists(bucket_name).await.is_ok();
    if !exists {
        minio.make_bucket(bucket_name).await?;
    }
    let args = ObjectArgs::new(bucket_name, object_name);
    minio.stat_object(args.clone()).await;
    minio
        .put_object(args.clone(), "hello minio".as_bytes().to_vec())
        .await;
    minio.fget_object(args.clone(), "tests/test.txt").await;
    minio.stat_object(args.clone());
    minio.remove_object(args.clone());
    minio.remove_bucket(bucket_name).await;
    Ok(())
}
