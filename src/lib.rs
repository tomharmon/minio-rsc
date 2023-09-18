#![cfg_attr(not(doctest), doc = include_str!("../README.md"))]

pub mod client;
mod credentials;
mod data;
pub mod error;
pub mod provider;
mod signer;
pub mod sse;
pub mod time;
pub mod datatype;
mod utils;
pub mod xml;

pub use crate::client::Minio;
pub use crate::credentials::Credentials;
pub use crate::data::Data;
pub use crate::signer::{presign_v4, sign_request_v4, sign_v4_authorization};
