mod common;

use common::{create_bucket_if_not_exist, get_test_minio};
use futures_util::{stream, StreamExt};
use minio_rsc::errors::Result;
use minio_rsc::types::args::ObjectArgs;
use minio_rsc::types::response::Tags;
use minio_rsc::types::ObjectLockConfiguration;
use tokio;
use minio_rsc::types::args::CopySource;

#[tokio::main]
#[test]
async fn test_base_operate() -> Result<()> {
    dotenv::dotenv().ok();
    let minio = get_test_minio();

    let bucket_name = "test-object-base";
    let object_name = "/test/test.txt";
    create_bucket_if_not_exist(&minio, bucket_name).await?;

    let args: ObjectArgs = ObjectArgs::new(bucket_name, object_name);
    let txt = "hello minio";
    minio.put_object(args.clone().content_type(Some("text/plain".to_string())), txt.into()).await?;
    assert_eq!(minio.get_object(args.clone()).await?.text().await?, txt);
    assert_eq!(minio.stat_object(args.clone()).await?.unwrap().object_name(),object_name);
    assert_eq!(minio.stat_object(args.clone()).await?.unwrap().content_type(),"text/plain");

    let mut tags: Tags = minio.get_object_tags(args.clone()).await?;
    tags.insert("key1", "value1");
    minio.set_object_tags(args.clone(), tags).await?;
    let tags = minio.get_object_tags(args.clone()).await?;
    assert_eq!(tags.get("key1").unwrap(),"value1");
    minio.delete_object_tags(args.clone()).await?;
    let tags = minio.get_object_tags(args.clone()).await?;
    assert!(tags.is_empty());

    let copy = CopySource::from(args.clone()).metadata_replace(true);
    let args2: ObjectArgs = args.clone().content_type(Some("image/jpeg".to_string()));
    minio.copy_object(args2.clone(), copy).await?;
    assert_eq!(minio.stat_object(args.clone()).await?.unwrap().content_type(),"image/jpeg");

    minio.remove_object(args.clone()).await?;
    minio.remove_bucket(bucket_name).await?;
    Ok(())
}

#[tokio::main]
#[test]
#[cfg(feature = "fs-tokio")]
async fn test_file_operate() -> Result<()> {
    dotenv::dotenv().ok();
    let minio = get_test_minio();

    let bucket_name = "test-file-operate";
    let object_name = "/test/test.txt";
    let loacl_file = "tests/test.txt";
    create_bucket_if_not_exist(&minio, bucket_name).await?;

    let args: ObjectArgs = ObjectArgs::new(bucket_name, object_name);
    minio.stat_object(args.clone()).await?;
    minio.put_object(args.clone(), "hello minio".into()).await?;

    minio.fget_object(args.clone(), loacl_file).await?;
    minio.fput_object(args.clone(), loacl_file).await?;

    minio.fput_object((bucket_name,"lena_std.jpeg"), "tests/lena_std.jpeg").await?;
    minio.remove_object((bucket_name,"lena_std.jpeg")).await?;

    minio.stat_object(args.clone()).await?;
    minio.remove_object(args.clone()).await?;
    minio.remove_bucket(bucket_name).await?;
    Ok(())
}


#[tokio::main]
#[test]
async fn test_put_stream() -> Result<()> {
    dotenv::dotenv().ok();
    let minio = get_test_minio();

    let bucket_name = "test-put-stream";
    let object_name = "test.txt";
    let len = 22*1024*1024; // 22MB
    let size = 128*1024;
    let num = len / size;
    let mut bytes = bytes::BytesMut::with_capacity(size);
    for _ in 0..size{
        bytes.extend_from_slice("A".as_bytes());
    }
    create_bucket_if_not_exist(&minio, bucket_name).await?;
    let stm = stream::repeat(bytes.freeze()).take(num).map(|f|Ok(f));
    let args: ObjectArgs = ObjectArgs::new(bucket_name, object_name);
    minio.put_object_stream(args.clone(), Box::pin(stm), Some(len)).await?;
    assert_eq!(minio.stat_object(args.clone()).await?.unwrap().size(),len);

    let mut bytes = bytes::BytesMut::with_capacity(size);
    for _ in 0..size{
        bytes.extend_from_slice("A".as_bytes());
    }
    create_bucket_if_not_exist(&minio, bucket_name).await?;
    let stm = stream::repeat(bytes.freeze()).take(num).map(|f|Ok(f));
    minio.put_object_stream(args.clone(), Box::pin(stm), None).await?;

    assert_eq!(minio.stat_object(args.clone()).await?.unwrap().size(),len);

    minio.remove_object(args.clone()).await?;
    minio.remove_bucket(bucket_name).await?;
    Ok(())
}