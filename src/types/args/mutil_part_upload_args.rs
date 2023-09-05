use hyper::HeaderMap;

use crate::{
    sse::{Sse, SseCustomerKey},
    types::response::InitiateMultipartUploadResult,
};

/// Custom request parameters for multiUpload operations.
///
/// Used in `abort_multipart_upload`, `complete_multipart_upload`, `create_multipart_upload`,
/// `MultipartUploadArgs`, `upload_part`, `upload_part_copy` method.
#[derive(Debug, Clone)]
pub struct MultipartUploadArgs {
    bucket: String,
    key: String,
    upload_id: String,
    bucket_owner: Option<String>,
    content_type: Option<String>,
    ssec_header: Option<HeaderMap>,
}

impl From<InitiateMultipartUploadResult> for MultipartUploadArgs {
    fn from(i: InitiateMultipartUploadResult) -> Self {
        Self::new(i.bucket, i.key, i.upload_id)
    }
}

impl MultipartUploadArgs {
    pub fn new(bucket: String, key: String, upload_id: String) -> Self {
        Self {
            bucket,
            key,
            upload_id,
            content_type: None,
            ssec_header: None,
            bucket_owner: None,
        }
    }

    pub fn bucket(&self) -> &str {
        self.bucket.as_ref()
    }

    pub fn key(&self) -> &str {
        self.key.as_ref()
    }

    pub fn upload_id(&self) -> &str {
        self.upload_id.as_ref()
    }

    pub fn content_type(&self) -> Option<&String> {
        self.content_type.as_ref()
    }

    pub fn bucket_owner(&self) -> Option<&String> {
        self.bucket_owner.as_ref()
    }

    pub fn ssec_header(&self) -> Option<&HeaderMap> {
        self.ssec_header.as_ref()
    }

    pub fn set_ssec(&mut self, ssec: SseCustomerKey) {
        self.ssec_header = Some(ssec.headers());
    }

    pub fn set_ssec_header(&mut self, ssec_header: Option<HeaderMap>) {
        self.ssec_header = ssec_header;
    }

    pub fn set_bucket_owner(&mut self, bucket_owner: Option<String>) {
        self.bucket_owner = bucket_owner;
    }
}
