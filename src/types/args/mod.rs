//! Request parameters

mod list_multipart_uploads_args;
mod mutil_part_upload_args;
mod select_object_content;

use hyper::HeaderMap;
pub use list_multipart_uploads_args::*;
pub use mutil_part_upload_args::MultipartUploadArgs;
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
