use std::collections::HashMap;

use hyper::{header::HeaderName, HeaderMap};

use crate::{
    errors::Result,
    sse::{Sse, SseCustomerKey},
};

/// Custom request parameters for opject operations.
/// ## parmas
/// - `bucket_name`: The bucket name.
/// - `object_name`: The object name.
/// - `region`: *Optional*, The bucket region.
/// - `expected_bucket_owner`: *Optional*, The account ID of the expected bucket owner.
/// - `version_id`: *Optional*, Version-ID of the object.
/// - `content_type`: *Optional*, Content type of the object.
/// - `ssec`: *Optional*, Server-side encryption customer key.
/// - `offset`: *Optional*, Start byte position of object data.
/// - `length`: *Optional*, Number of bytes of object data from offset.
/// - `metadata`: *Optional*, user-defined metadata.
/// - `extra_headers`: *Optional*, Extra headers for advanced usage.
///
/// **Note**: Some parameters are only valid in specific methods
#[derive(Debug, Clone)]
pub struct ObjectArgs {
    pub(crate) bucket_name: String,
    pub(crate) object_name: String,
    pub(crate) region: Option<String>,
    pub(crate) expected_bucket_owner: Option<String>,
    pub(crate) version_id: Option<String>,
    pub(crate) content_type: Option<String>,
    pub(crate) ssec_headers: Option<HeaderMap>,
    pub(crate) offset: usize,
    pub(crate) length: usize,
    pub(crate) extra_headers: Option<HeaderMap>,
    pub(crate) metadata: HashMap<String, String>,
}

impl ObjectArgs {
    pub fn new<S1: Into<String>, S2: Into<String>>(bucket_name: S1, object_name: S2) -> Self {
        Self {
            bucket_name: bucket_name.into(),
            object_name: object_name.into(),
            region: None,
            expected_bucket_owner: None,
            extra_headers: None,
            version_id: None,
            content_type: None,
            ssec_headers: None,
            offset: 0,
            length: 0,
            metadata: Default::default(),
        }
    }

    /// Set object region
    pub fn region(mut self, region: Option<String>) -> Self {
        self.region = region;
        self
    }

    /// Set version-ID of the object
    pub fn version_id(mut self, version_id: Option<String>) -> Self {
        self.version_id = version_id;
        self
    }

    /// Set content-type of the object
    pub fn content_type(mut self, content_type: Option<String>) -> Self {
        self.content_type = content_type;
        self
    }

    /// Set the account ID of the expected bucket owner.
    pub fn expected_bucket_owner(mut self, expected_bucket_owner: Option<String>) -> Self {
        self.expected_bucket_owner = expected_bucket_owner;
        self
    }

    /// Set extra headers for advanced usage.
    pub fn extra_headers(mut self, extra_headers: Option<HeaderMap>) -> Self {
        self.extra_headers = extra_headers;
        self
    }

    /// Set server-side encryption customer key
    pub fn ssec(mut self, ssec: &SseCustomerKey) -> Self {
        self.ssec_headers = Some(ssec.headers());
        self
    }

    /// Returns the range of this [`ObjectArgs`].
    pub(crate) fn range(&self) -> Option<String> {
        if self.offset > 0 || self.length > 0 {
            Some(if self.length > 0 {
                format!("bytes={}-{}", self.offset, self.offset + self.length - 1)
            } else {
                format!("bytes={}-", self.offset)
            })
        } else {
            None
        }
    }

    /// Set start byte position of object data when `download` an object.
    /// Valid in the download operation of the object.
    ///
    /// Default: 0
    pub fn offset(mut self, offset: usize) -> Self {
        self.offset = offset;
        self
    }

    /// Set number of bytes of object data from offset when `download` an object.
    /// If set length 0, it means to the end of the object.
    ///
    /// Default: 0
    pub fn length(mut self, length: usize) -> Self {
        self.length = length;
        self
    }

    /// Set user-defined metadata when `uploading` an object.
    /// Metadata is a set of key-value pairs.
    ///
    /// key:
    /// - requirement is ASCII and cannot contain non-ASCII characters
    /// - Cannot contain invisible characters and spaces
    /// - does't need to start with `x-amz-meta-`
    /// - ignoring case
    ///
    pub fn metadata(mut self, metadata: HashMap<String, String>) -> Self {
        self.metadata = metadata;
        self
    }

    /// Returns the metadata header of this [`ObjectArgs`].
    pub(crate) fn get_metadata_header(&self) -> Result<HeaderMap> {
        let mut meta_header: HeaderMap = HeaderMap::new();
        for (key, value) in &self.metadata {
            let key = HeaderName::from_bytes(format!("x-amz-meta-{}", key).as_bytes())?;
            meta_header.insert(key, value.parse()?);
        }
        Ok(meta_header)
    }
}

impl<S1, S2> From<(S1, S2)> for ObjectArgs
where
    S1: Into<String>,
    S2: Into<String>,
{
    fn from((b, k): (S1, S2)) -> Self {
        Self::new(b, k)
    }
}
