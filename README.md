# minio-rsc
[![Crates.io](https://img.shields.io/crates/v/minio-rsc)](https://crates.io/crates/minio-rsc)
[![Documentation](https://docs.rs/minio-rsc/badge.svg)](https://docs.rs/minio-rsc)
[![License](https://img.shields.io/crates/l/minio-rsc)](#license)

Rust Library for Minio. API is compliant with the Amazon S3 protocol.

## Minio client
```rust
use minio_rsc::Minio;
use minio_rsc::provider::StaticProvider;
use minio_rsc::errors::Result;
use minio_rsc::types::args::{BucketArgs, ObjectArgs};

async fn example() -> Result<()>{
    let provider = StaticProvider::new("minio-access-key-test", "minio-secret-key-test", None);
    let minio = Minio::builder()
        .endpoint("localhost:9022")
        .provider(provider)
        .secure(false)
        .build()
        .unwrap();
    let (buckets, owner) = minio.list_buckets().await?;

    minio.make_bucket(BucketArgs::new("bucket1"), false).await?;
    minio.make_bucket("bucket2", true).await?;
    minio.put_object(("bucket1","hello.txt"), "hello minio!".into()).await?;
    minio.stat_object(("bucket1","hello.txt")).await?;
    minio.get_object(("bucket1","hello.txt")).await?;
    minio.get_object(
            ObjectArgs::new("bucket1","hello.txt")
                .version_id(Some("cdabf31a-9752-4265-b137-6b3961fbaf9b".to_string()))
        ).await?;

    // if fs-tokio feature enabled
    // download file to local
    minio.fget_object(("bucket1","hello.txt"), "local.txt").await?;
    // upload file to minio
    minio.fput_object(("bucket1","hello.txt"), "local.txt").await?;
    minio.remove_object(("bucket1","hello.txt")).await?;

    minio.remove_bucket("bucket1").await?;
    minio.remove_bucket("bucket2").await?;

    Ok(())
}
```

## Operations
| Bucket operations | Object operations |
|-|-|
| [make_bucket](https://docs.rs/minio-rsc/latest/minio_rsc/client/struct.Minio.html#method.make_bucket) | [get_object](https://docs.rs/minio-rsc/latest/minio_rsc/client/struct.Minio.html#method.get_object) |
| [list_buckets](https://docs.rs/minio-rsc/latest/minio_rsc/client/struct.Minio.html#method.list_buckets) | [fget_object](https://docs.rs/minio-rsc/latest/minio_rsc/client/struct.Minio.html#method.fget_object) |
| [bucket_exists](https://docs.rs/minio-rsc/latest/minio_rsc/client/struct.Minio.html#method.bucket_exists) | [copy_object](https://docs.rs/minio-rsc/latest/minio_rsc/client/struct.Minio.html#method.copy_object) |
| [remove_bucket](https://docs.rs/minio-rsc/latest/minio_rsc/client/struct.Minio.html#method.remove_bucket) | [stat_object](https://docs.rs/minio-rsc/latest/minio_rsc/client/struct.Minio.html#method.stat_object) |
| [list_objects](https://docs.rs/minio-rsc/latest/minio_rsc/client/struct.Minio.html#method.list_objects) | [remove_object](https://docs.rs/minio-rsc/latest/minio_rsc/client/struct.Minio.html#method.remove_object) |
| [get_bucket_tags](https://docs.rs/minio-rsc/latest/minio_rsc/client/struct.Minio.html#method.get_bucket_tags) | [put_object](https://docs.rs/minio-rsc/latest/minio_rsc/client/struct.Minio.html#method.put_object) |
| [set_bucket_tags](https://docs.rs/minio-rsc/latest/minio_rsc/client/struct.Minio.html#method.set_bucket_tags) | [fput_object](https://docs.rs/minio-rsc/latest/minio_rsc/client/struct.Minio.html#method.fput_object) |
| [delete_bucket_tags](https://docs.rs/minio-rsc/latest/minio_rsc/client/struct.Minio.html#method.delete_bucket_tags) | [presigned_get_object](https://docs.rs/minio-rsc/latest/minio_rsc/client/struct.Minio.html#method.presigned_get_object) |
| [get_bucket_versioning](https://docs.rs/minio-rsc/latest/minio_rsc/client/struct.Minio.html#method.get_bucket_versioning) | [presigned_put_object](https://docs.rs/minio-rsc/latest/minio_rsc/client/struct.Minio.html#method.presigned_put_object) |
| [set_bucket_versioning](https://docs.rs/minio-rsc/latest/minio_rsc/client/struct.Minio.html#method.set_bucket_versioning) | [is_object_legal_hold_enabled](https://docs.rs/minio-rsc/latest/minio_rsc/client/struct.Minio.html#method.is_object_legal_hold_enabled) |
| [get_object_lock_config](https://docs.rs/minio-rsc/latest/minio_rsc/client/struct.Minio.html#method.get_object_lock_config) | [enable_object_legal_hold_enabled](https://docs.rs/minio-rsc/latest/minio_rsc/client/struct.Minio.html#method.enable_object_legal_hold_enabled) |
| [set_object_lock_config](https://docs.rs/minio-rsc/latest/minio_rsc/client/struct.Minio.html#method.set_bobject_lock_config) | [disable_object_legal_hold_enabled](https://docs.rs/minio-rsc/latest/minio_rsc/client/struct.Minio.html#method.disable_object_legal_hold_enabled) |
| [delete_object_lock_config](https://docs.rs/minio-rsc/latest/minio_rsc/client/struct.Minio.html#method.delete_bobject_lock_config) | [get_object_tags](https://docs.rs/minio-rsc/latest/minio_rsc/client/struct.Minio.html#method.get_object_tags) |
|  | [set_object_tags](https://docs.rs/minio-rsc/latest/minio_rsc/client/struct.Minio.html#method.set_object_tags) |
|  | [delete_object_tags](https://docs.rs/minio-rsc/latest/minio_rsc/client/struct.Minio.html#method.delete_object_tags) |
| | [get_object_retention](https://docs.rs/minio-rsc/latest/minio_rsc/client/struct.Minio.html#method.get_object_retention) |
| | [put_object_retention](https://docs.rs/minio-rsc/latest/minio_rsc/client/struct.Minio.html#method.put_object_retention) |
| | [select_object_content](https://docs.rs/minio-rsc/latest/minio_rsc/client/struct.Minio.html#method.select_object_content) |
| |  |

## Features
- `fs-tokio` which provides asynchronous local file operations based on the tokio. [fput_object](https://docs.rs/minio-rsc/latest/minio_rsc/client/struct.Minio.html#method.fput_object), [fget_object](https://docs.rs/minio-rsc/latest/minio_rsc/client/struct.Minio.html#method.fget_object)

## Custom requests
Implemented by [BaseExecutor](https://docs.rs/minio-rsc/latest/minio_rsc/client/struct.BaseExecutor.html)

```rust
use minio_rsc::Minio;
use hyper::Method;
use minio_rsc::errors::Result;
use reqwest::Response;
use bytes::Bytes;

async fn get_object(minio:Minio)-> Result<Response> {
    let executor = minio.executor(Method::GET);
    let res: Response = executor
        .bucket_name("bucket")
        .object_name("test.txt")
        .query("versionId", "cdabf31a-9752-4265-b137-6b3961fbaf9b")
        .send_ok()
        .await?;
    Ok(res)
}

async fn put_object(minio:Minio, data:Bytes)-> Result<()> {
    let executor = minio.executor(Method::PUT);
    let res: Response = executor
        .bucket_name("bucket")
        .object_name("test.txt")
        .body(data)
        .send_ok()
        .await?;
    Ok(())
}
```