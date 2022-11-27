//! rust for minio
//!
//! # Example
/// ```
/// use minio_rsc::client::Minio;
/// use minio_rsc::provider::StaticProvider;
/// use tokio;
/// #[tokio::main]
/// async fn it_works(){
///     let provider = StaticProvider::new("minio-test", "minio-test", None);
///     let minio = Minio::builder()
///         .host("localhost:9022")
///         .provider(provider)
///         .secure(false)
///         .builder()
///         .unwrap();
/// }
/// ```
///
pub mod client;
mod credentials;
mod errors;
mod executor;
pub mod provider;
mod signer;
mod sse;
pub mod time;
pub mod types;
mod utils;

pub use crate::client::Minio;
pub use crate::credentials::Credentials;
pub use crate::signer::sign_v4_authorization;
