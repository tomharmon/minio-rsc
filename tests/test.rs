mod common;
use std::collections::HashMap;

use common::get_test_minio;
use hyper::Method;
use minio_rsc::errors::{Result, XmlError};
use minio_rsc::types::args::{BucketArgs, ObjectArgs, PresignedArgs};
use minio_rsc::types::response::Tags;
use minio_rsc::types::{ObjectLockConfiguration, VersioningConfiguration};
use tokio;

#[tokio::main]
#[test]
async fn test_bucket() -> Result<()> {
    dotenv::dotenv().ok();
    let minio = get_test_minio();
    let bucket1 = "bucket-test-1";
    let bucket2 = "bucket-test-2";
    minio.make_bucket(bucket1, false).await?;
    minio.make_bucket(bucket2, true).await?;
    println!("{:?}", minio.list_buckets().await);

    println!("====== begin test tagging");
    println!("{:?}", minio.get_bucket_tags(bucket1).await?.is_none());
    println!("{:?}", minio.set_bucket_tags(bucket1, Tags::new()).await);
    let mut tags = minio.get_bucket_tags(bucket1).await?.unwrap();
    tags.insert("key1".to_string(), "value1".to_string());
    tags.insert("key2".to_string(), "value2".to_string());
    tags.insert("key3".to_string(), "value3".to_string());
    println!("{:?}", minio.set_bucket_tags(bucket1, tags).await);
    println!("{:?}", minio.get_bucket_tags(bucket1).await?);
    println!("{:?}", minio.delete_bucket_tags(bucket1).await?);
    println!("{:?}", minio.get_bucket_tags(bucket1).await?.is_none());

    println!("====== begin test versioning");
    let mut versing = minio.get_bucket_versioning(bucket1).await?;
    println!("get {:?}", versing);
    versing.set_status_enable(!versing.is_status_enabled());
    minio.set_bucket_versioning(bucket1, versing).await?;
    let versing = minio.get_bucket_versioning(bucket1).await?;
    println!("get {:?}", versing);

    println!("====== begin test object_lock_configuration");
    println!(
        "set {:?}",
        minio
            .set_object_lock_config(bucket2, ObjectLockConfiguration::new())
            .await
    );
    let mut conf = ObjectLockConfiguration::new();
    conf.set_mode(true);
    conf.set_duration(1, true);
    println!("get {:?}", minio.get_object_lock_config(bucket1).await);
    println!("del {:?}", minio.delete_object_lock_config(bucket2).await);
    println!("get {:?}", minio.get_object_lock_config(bucket2).await);
    println!(
        "set {:?}",
        minio.set_object_lock_config(bucket2, conf).await
    );
    println!("get {:?}", minio.get_object_lock_config(bucket2).await);

    println!("====== begin clear test bucket");
    assert!(minio.bucket_exists(bucket1).await?);
    assert!(minio.remove_bucket(bucket1).await.is_ok());
    assert!(!minio.bucket_exists(bucket1).await?);
    assert!(minio.remove_bucket(bucket2).await.is_ok());

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

    let bucket_name = "bucket-test-1";
    let object_name = "/test/1.txt";
    let exists = minio.bucket_exists(bucket_name).await?;
    if !exists {
        minio.make_bucket(bucket_name, false).await?;
    }
    let args = ObjectArgs::new(bucket_name, object_name);
    minio.stat_object(args.clone()).await?;
    minio
        .put_object(args.clone(), "hello minio".as_bytes().to_vec())
        .await?;
    minio.fget_object(args.clone(), "tests/test.txt").await?;
    minio.stat_object(args.clone()).await?;
    minio.remove_object(args.clone()).await?;
    minio.remove_bucket(bucket_name).await?;
    Ok(())
}
