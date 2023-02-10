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
}
```

## Operations
| Bucket operations | Object operations |
|-|-|
| make_bucket | get_object |
| list_buckets | fget_object |
| bucket_exists | copy_object |
| remove_bucket | stat_object |
| list_objects | remove_object |
|  | put_object |
|  | presigned_get_object |
|  | presigned_put_object |
