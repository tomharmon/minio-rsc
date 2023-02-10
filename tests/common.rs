use minio_rsc::{provider::StaticProvider, Minio};

pub fn get_test_minio() -> Minio {
    let provider = StaticProvider::new("minio-access-key-test", "minio-secret-key-test", None);
    Minio::builder()
        .host("localhost:9022")
        .provider(provider)
        .secure(false)
        .build()
        .unwrap()
}
