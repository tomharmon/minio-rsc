//! rust for minio
//!
//! # Example
//! ```
//! use minio_rsc::client::Minio;
//! use minio_rsc::provider::StaticProvider;
//! use tokio;
//! #[tokio::main]
//! async fn it_works(){
//!     let provider = StaticProvider::new("minio-access-key-test", "minio-secret-key-test", None);
//!     let minio = Minio::builder()
//!         .endpoint("localhost:9022")
//!         .provider(provider)
//!         .secure(false)
//!         .build()
//!         .unwrap();
//! }
//! ```

pub mod client;
mod credentials;
pub mod errors;
pub mod executor;
pub mod provider;
mod signer;
pub mod sse;
pub mod time;
pub mod types;
mod utils;

pub use crate::client::Minio;
pub use crate::credentials::Credentials;
pub use crate::signer::{presign_v4, sign_v4_authorization};
