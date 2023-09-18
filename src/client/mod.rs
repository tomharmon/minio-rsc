//! Minio client
pub mod args;
mod bucket;
mod client;
mod executor;
mod mutilpart_upload;
mod operate_bucket;
mod operate_object;
mod presigned;
mod querymap;
pub mod response;
mod select_object_reader;

pub use args::{
    BucketArgs, CopySource, KeyArgs, ListMultipartUploadsArgs, ListObjectsArgs,
    MultipartUploadTask, ObjectLockConfig, PresignedArgs, Tags,
};
pub use bucket::Bucket;
pub use client::*;
pub use executor::BaseExecutor;
pub use querymap::QueryMap;
pub use response::{
    CompleteMultipartUploadResult, ListBucketResult, ListMultipartUploadsResult, ListPartsResult,
    ObjectStat,
};
pub use select_object_reader::{Message, SelectObjectReader};
