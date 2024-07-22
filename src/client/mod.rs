//! Minio client
mod args;
mod bucket;
mod client;
mod executor;
mod mutilpart_upload;
mod operate_bucket;
#[cfg(feature = "ext")]
mod operate_ext;
mod operate_object;
mod presigned;
mod querymap;
mod response;
mod select_object_reader;

pub use args::{
    BucketArgs, CopySource, KeyArgs, ListMultipartUploadsArgs, ListObjectVersionsArgs,
    ListObjectsArgs, MultipartUploadTask, ObjectLockConfig, PresignedArgs, Tags,
};
pub use bucket::Bucket;
pub use client::*;
pub use executor::BaseExecutor;
pub use querymap::QueryMap;
pub use response::ObjectStat;
pub use select_object_reader::{Message, SelectObjectReader};
