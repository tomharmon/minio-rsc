use hyper::HeaderMap;

use crate::{
    sse::{Sse, SseCustomerKey},
    utils::urlencode,
};

use super::{BaseArgs, ObjectArgs};

/// A source object definition for `copy_object` and `upload_part_copy` method.
#[derive(Debug, Clone)]
pub struct CopySource {
    bucket_name: String,
    object_name: String,
    region: Option<String>,
    offset: usize,
    length: usize,
    version_id: Option<String>,
    metadata_replace: bool,
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
            metadata_replace: false,
            ssec: None,
            match_etag: None,
            not_match_etag: None,
            modified_since: None,
            unmodified_since: None,
            offset: 0,
            length: 0,
        }
    }

    /// Set object region
    pub fn region(mut self, region: Option<String>) -> Self {
        self.region = region;
        self
    }

    /// Used only in `upload_part_copy` method.
    ///
    /// **Note**: length must be greater than 0, or both length and offset are 0.
    pub fn range(mut self, offset: usize, length: usize) -> Self {
        self.offset = offset;
        self.length = length;
        self
    }

    /// When copying an object, preserve all metadata if set `false` (default) or specify new metadata.
    pub fn metadata_replace(mut self, metadata_replace: bool) -> Self {
        self.metadata_replace = metadata_replace;
        self
    }

    /// Set version-ID of the object
    pub fn version_id<T: Into<String>>(mut self, version_id: T) -> Self {
        self.version_id = Some(version_id.into());
        self
    }

    /// Set server-side encryption customer key
    pub fn ssec(mut self, ssec: &SseCustomerKey) -> Self {
        let mut header = ssec.headers();
        header.extend(ssec.copy_headers());
        self.ssec = Some(header);
        self
    }

    pub fn match_etag(mut self, match_etag: Option<String>) -> Self {
        self.match_etag = match_etag;
        self
    }

    pub fn not_match_etag(mut self, not_match_etag: Option<String>) -> Self {
        self.not_match_etag = not_match_etag;
        self
    }

    pub fn modified_since(mut self, modified_since: Option<String>) -> Self {
        self.modified_since = modified_since;
        self
    }

    pub fn unmodified_since(mut self, unmodified_since: Option<String>) -> Self {
        self.unmodified_since = unmodified_since;
        self
    }
}

impl From<ObjectArgs> for CopySource {
    fn from(value: ObjectArgs) -> Self {
        Self {
            bucket_name: value.bucket_name,
            object_name: value.object_name,
            region: value.region,
            version_id: value.version_id,
            ssec: value.ssec_headers,
            metadata_replace: false,
            match_etag: None,
            not_match_etag: None,
            modified_since: None,
            unmodified_since: None,
            offset: value.offset,
            length: value.length,
        }
    }
}

impl BaseArgs for CopySource {
    fn args_headers(&self) -> HeaderMap {
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
        if self.metadata_replace {
            header.insert("x-amz-metadata-directive", "REPLACE".parse().unwrap());
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
