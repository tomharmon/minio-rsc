mod common;

use common::get_test_minio;
use minio_rsc::errors::Result;
use minio_rsc::types::args::PresignedArgs;
use minio_rsc::types::response::Tags;
use minio_rsc::types::ObjectLockConfiguration;
use tokio;

#[tokio::main]
#[test]
async fn test_bucket() -> Result<()> {
    let minio = get_test_minio();
    let bucket1 = "bucket-test-1";
    let bucket2 = "bucket-test-2";

    println!("\r\n====== begin create buckets");
    minio.make_bucket(bucket1, false).await?;
    minio.make_bucket(bucket2, true).await?;

    println!("\r\n====== begin test list_buckets");
    let (buckets, owner) = minio.list_buckets().await?;
    for b in buckets {
        println!("bucket: {} owner {}", b.name, owner.display_name);
    }

    println!("\r\n====== begin test tagging");
    assert!(minio.get_bucket_tags(bucket1).await?.is_none());
    minio.set_bucket_tags(bucket1, Tags::new()).await?;
    let mut tags = Tags::new();
    tags.insert("key1", "value1")
        .insert("key2", "value2")
        .insert("key3", "value3");
    minio.set_bucket_tags(bucket1, tags).await?;
    let tags = minio.get_bucket_tags(bucket1).await?.unwrap();
    assert!(tags.contains_key("key2"));
    assert!(tags.get("key2").unwrap() == "value2");
    minio.delete_bucket_tags(bucket1).await?;
    assert!(minio.get_bucket_tags(bucket1).await?.is_none());

    // test bucket versioning
    println!("\r\n====== begin test versioning");
    let mut versing = minio.get_bucket_versioning(bucket1).await?;
    assert!(!versing.is_status_enabled());
    versing.set_status_enable(true);
    minio.set_bucket_versioning(bucket1, versing).await?;
    let versing = minio.get_bucket_versioning(bucket1).await?;
    assert!(versing.is_status_enabled());

    println!("\r\n====== begin test object_lock_configuration");
    let conf = ObjectLockConfiguration::new(12, true, false);
    assert!(minio.set_object_lock_config(bucket2, conf).await.is_ok());

    println!("get {:?}", minio.get_object_lock_config(bucket2).await);
    assert!(minio.delete_object_lock_config(bucket2).await.is_ok());
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
    let minio = get_test_minio();

    let url = minio
        .presigned_get_object(PresignedArgs::new("bucket", "file.txt").expires(24 * 3600))
        .await?;
    println!("get url {}", url);
    let url = minio
        .presigned_put_object(PresignedArgs::new("bucket", "file.txt").expires(24 * 3600))
        .await?;
    println!("put url {}", url);
    Ok(())
}
