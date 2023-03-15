# minio-rsc
Rust Library for Minio

## Minio client
```rust
use minio_rsc::client::Minio;
use minio_rsc::provider::StaticProvider;
use tokio;

#[tokio::main]
async fn main() {
    let provider = StaticProvider::new("minio-access-key-test", "minio-secret-key-test", None);
    let minio = Minio::builder()
        .host("localhost:9022")
        .provider(provider)
        .secure(false)
        .build()
        .unwrap();
    let (buckets, owner) = minio.list_buckets().await.unwrap();
}
```

## Operations
| Bucket operations | Object operations |
|-|-|
| [make_bucket](https://docs.rs/minio-rsc/latest/minio_rsc/struct.Minio.html#method.make_bucket) | [get_object](https://docs.rs/minio-rsc/latest/minio_rsc/struct.Minio.html#method.get_object) |
| [list_buckets](https://docs.rs/minio-rsc/latest/minio_rsc/struct.Minio.html#method.list_buckets) | [fget_object](https://docs.rs/minio-rsc/latest/minio_rsc/struct.Minio.html#method.fget_object) |
| [bucket_exists](https://docs.rs/minio-rsc/latest/minio_rsc/struct.Minio.html#method.bucket_exists) | [copy_object](https://docs.rs/minio-rsc/latest/minio_rsc/struct.Minio.html#method.copy_object) |
| [remove_bucket](https://docs.rs/minio-rsc/latest/minio_rsc/struct.Minio.html#method.remove_bucket) | [stat_object](https://docs.rs/minio-rsc/latest/minio_rsc/struct.Minio.html#method.stat_object) |
| [list_objects](https://docs.rs/minio-rsc/latest/minio_rsc/struct.Minio.html#method.list_objects) | [remove_object](https://docs.rs/minio-rsc/latest/minio_rsc/struct.Minio.html#method.remove_object) |
| [get_bucket_tags](https://docs.rs/minio-rsc/latest/minio_rsc/struct.Minio.html#method.get_bucket_tags) | [put_object](https://docs.rs/minio-rsc/latest/minio_rsc/struct.Minio.html#method.put_object) |
| [set_bucket_tags](https://docs.rs/minio-rsc/latest/minio_rsc/struct.Minio.html#method.set_bucket_tags) | [fput_object](https://docs.rs/minio-rsc/latest/minio_rsc/struct.Minio.html#method.fput_object) |
| [delete_bucket_tags](https://docs.rs/minio-rsc/latest/minio_rsc/struct.Minio.html#method.delete_bucket_tags) | [presigned_get_object](https://docs.rs/minio-rsc/latest/minio_rsc/struct.Minio.html#method.presigned_get_object) |
| [get_bucket_versioning](https://docs.rs/minio-rsc/latest/minio_rsc/struct.Minio.html#method.get_bucket_versioning) | [presigned_put_object](https://docs.rs/minio-rsc/latest/minio_rsc/struct.Minio.html#method.presigned_put_object) |
| [set_bucket_versioning](https://docs.rs/minio-rsc/latest/minio_rsc/struct.Minio.html#method.set_bucket_versioning) | [is_object_legal_hold_enabled](https://docs.rs/minio-rsc/latest/minio_rsc/struct.Minio.html#method.is_object_legal_hold_enabled) |
| [get_object_lock_config](https://docs.rs/minio-rsc/latest/minio_rsc/struct.Minio.html#method.get_object_lock_config) | [enable_object_legal_hold_enabled](https://docs.rs/minio-rsc/latest/minio_rsc/struct.Minio.html#method.enable_object_legal_hold_enabled) |
| [set_object_lock_config](https://docs.rs/minio-rsc/latest/minio_rsc/struct.Minio.html#method.set_bobject_lock_config) | [disable_object_legal_hold_enabled](https://docs.rs/minio-rsc/latest/minio_rsc/struct.Minio.html#method.disable_object_legal_hold_enabled) |
| [delete_object_lock_config](https://docs.rs/minio-rsc/latest/minio_rsc/struct.Minio.html#method.delete_bobject_lock_config) | [get_object_tags](https://docs.rs/minio-rsc/latest/minio_rsc/struct.Minio.html#method.get_object_tags) |
|  | [set_object_tags](https://docs.rs/minio-rsc/latest/minio_rsc/struct.Minio.html#method.set_object_tags) |
|  | [delete_object_tags](https://docs.rs/minio-rsc/latest/minio_rsc/struct.Minio.html#method.delete_object_tags) |
| | [get_object_retention](https://docs.rs/minio-rsc/latest/minio_rsc/struct.Minio.html#method.get_object_retention) |
| | [put_object_retention](https://docs.rs/minio-rsc/latest/minio_rsc/struct.Minio.html#method.put_object_retention) |
| |  |
