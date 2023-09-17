//! Minio client
mod args;
mod bucket;
mod client;
mod executor;
mod mutilpart_upload;
mod operate_bucket;
mod operate_object;
mod presigned;
mod querymap;
mod response;

pub use args::{
    BucketArgs, CopySource, KeyArgs, ListMultipartUploadsArgs, ListObjectsArgs,
    MultipartUploadTask, PresignedArgs,
};
pub use bucket::Bucket;
pub use client::*;
pub use executor::BaseExecutor;
pub use querymap::QueryMap;
