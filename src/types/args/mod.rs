//! Request parameters

mod bucket_args;
mod copy_source;
mod list_multipart_uploads_args;
mod list_objects_args;
mod mutil_part_upload_args;
mod object_args;
mod presigned_args;
mod select_object_content;

pub use bucket_args::BucketArgs;
pub use copy_source::CopySource;
use hyper::HeaderMap;
pub use list_multipart_uploads_args::*;
pub use list_objects_args::*;
pub use mutil_part_upload_args::MultipartUploadArgs;
pub use object_args::ObjectArgs;
pub use presigned_args::*;
pub use select_object_content::*;

use super::QueryMap;

pub(crate) trait BaseArgs {
    fn args_query_map(&self) -> QueryMap {
        QueryMap::default()
    }

    fn args_headers(&self) -> HeaderMap {
        HeaderMap::new()
    }
}
