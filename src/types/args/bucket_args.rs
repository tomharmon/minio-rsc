use hyper::HeaderMap;

use super::ObjectArgs;

/// Custom request parameters for bucket operations.
/// ## parmas
/// - `bucket_name`: The bucket name.
/// - `region`: *Optional*, The bucket region.
/// - `expected_bucket_owner`: *Optional*, The account ID of the expected bucket owner.
/// - `extra_headers`: *Optional*, Extra headers for advanced usage.
///
/// **Note**: Some parameters are only valid in specific methods
#[derive(Debug, Clone)]
pub struct BucketArgs {
    pub(crate) bucket_name: String,
    pub(crate) region: Option<String>,
    pub(crate) expected_bucket_owner: Option<String>,
    pub(crate) extra_headers: Option<HeaderMap>,
}

impl BucketArgs {
    pub fn new<S: Into<String>>(bucket_name: S) -> Self {
        Self {
            bucket_name: bucket_name.into(),
            region: None,
            expected_bucket_owner: None,
            extra_headers: None,
        }
    }

    /// Set object region
    pub fn region(mut self, region: Option<String>) -> Self {
        self.region = region;
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

    /// create [ObjectArgs] from object_name.
    pub fn into_object_args<T: Into<String>>(self, object_name: T) -> ObjectArgs {
        ObjectArgs::new(self.bucket_name, object_name.into())
            .expected_bucket_owner(self.expected_bucket_owner)
            .region(self.region)
            .extra_headers(self.extra_headers)
    }
}

impl<S> From<S> for BucketArgs
where
    S: Into<String>,
{
    fn from(s: S) -> Self {
        Self::new(s)
    }
}

impl From<ObjectArgs> for BucketArgs {
    fn from(obj: ObjectArgs) -> Self {
        Self::new(obj.bucket_name)
            .expected_bucket_owner(obj.expected_bucket_owner)
            .region(obj.region)
            .extra_headers(obj.extra_headers)
    }
}
