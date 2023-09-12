//! Minio client
mod args;
mod bucket;
mod client;
mod executor;
mod mutilpart_upload;
mod operate_bucket;
mod operate_object;
mod presigned;

pub use args::{BucketArgs, CopySource, KeyArgs, ListObjectsArgs, PresignedArgs};
pub use bucket::Bucket;
pub use client::*;
pub use executor::BaseExecutor;
