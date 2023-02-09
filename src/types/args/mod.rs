mod list_multipart_uploads_args;
mod list_objects_args;
mod presigned_args;

use hyper::HeaderMap;
pub use list_multipart_uploads_args::*;
pub use list_objects_args::*;
pub use presigned_args::*;

use crate::{
    sse::{Sse, SseCustomerKey},
    utils::urlencode,
};

use super::{response::InitiateMultipartUploadResult, QueryMap};

pub(crate) trait BaseArgs {
    fn extra_query_map(&self) -> QueryMap {
        QueryMap::default()
    }

    fn extra_headers(&self) -> HeaderMap {
        HeaderMap::new()
    }
}

pub struct CopySource {
    bucket_name: String,
    object_name: String,
    region: Option<String>,
    offset: usize,
    length: usize,
    version_id: Option<String>,
    ssec: Option<HeaderMap>,
    match_etag: Option<String>,
    not_match_etag: Option<String>,
    modified_since: Option<String>,
    unmodified_since: Option<String>,
}

impl CopySource {
    pub fn new<T1: Into<String>, T2: Into<String>>(bucket_name: T1, object_name: T2) -> Self {
        Self {
            bucket_name: bucket_name.into(),
            object_name: object_name.into(),
            region: None,
            version_id: None,
            ssec: None,
            match_etag: None,
            not_match_etag: None,
            modified_since: None,
            unmodified_since: None,
            offset: 0,
            length: 0,
        }
    }

    pub fn offset(mut self, offset: usize) -> Self {
        self.offset = offset;
        self
    }

    pub fn length(mut self, length: usize) -> Self {
        self.length = length;
        self
    }

    pub fn version_id<T: Into<String>>(mut self, version_id: T) -> Self {
        self.version_id = Some(version_id.into());
        self
    }

    pub fn ssec(mut self, ssec: &SseCustomerKey) -> Self {
        let mut header = ssec.headers();
        header.extend(ssec.copy_headers());
        self.ssec = Some(header);
        self
    }
}

impl BaseArgs for CopySource {
    fn extra_headers(&self) -> HeaderMap {
        let mut header = HeaderMap::new();
        let mut copy_source =
            urlencode(&format!("/{}/{}", self.bucket_name, self.object_name), true);
        if let Some(version_id) = &self.version_id {
            copy_source = copy_source + "?versionId=" + version_id;
        }
        header.insert("x-amz-copy-source", copy_source.parse().unwrap());
        if let Some(value) = &self.match_etag {
            header.insert("x-amz-copy-source-if-match", value.parse().unwrap());
        }
        if let Some(value) = &self.not_match_etag {
            header.insert("x-amz-copy-source-if-none-match", value.parse().unwrap());
        }
        if let Some(value) = &self.modified_since {
            header.insert(
                "x-amz-copy-source-if-modified-since",
                value.parse().unwrap(),
            );
        }
        if let Some(value) = &self.unmodified_since {
            header.insert(
                "x-amz-copy-source-if-unmodified-since",
                value.parse().unwrap(),
            );
        }
        if self.offset > 0 || self.length > 0 {
            let ranger = if self.length > 0 {
                format!("bytes={}-{}", self.offset, self.offset + self.length - 1)
            } else {
                format!("bytes={}-", self.offset)
            };
            if let Ok(value) = ranger.parse() {
                header.insert("x-amz-copy-source-range", value);
            }
        }
        if let Some(ssec) = &self.ssec {
            header.extend(ssec.clone());
            for (k, v) in ssec {
                header.insert(k, v.to_owned());
            }
        }
        header
    }
}

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

    pub fn bucket_owner(&self) -> Option<&String> {
        self.bucket_owner.as_ref()
    }
}
