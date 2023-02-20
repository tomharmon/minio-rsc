use hyper::HeaderMap;

struct MakeBucketArgs {
    bucket_name: String,
    region: Option<String>,
    object_lock: bool,
    extra_headers: Option<HeaderMap>,
}

impl MakeBucketArgs {
    /// Creates a new [`MakeBucketArgs`].
    pub fn new(bucket_name: String) -> Self {
        Self {
            bucket_name,
            region: None,
            object_lock: false,
            extra_headers: None,
        }
    }

    fn set_object_lock(&mut self, lock: bool) {
        self.object_lock = lock;
    }

    fn bucket_name(&self) -> &str {
        self.bucket_name.as_ref()
    }

    fn region(&self) -> Option<&String> {
        self.region.as_ref()
    }

    fn object_lock(&self) -> bool {
        self.object_lock
    }

    fn extra_headers(&self) -> Option<&HeaderMap> {
        self.extra_headers.as_ref()
    }
}

impl<S> From<S> for MakeBucketArgs
where
    S: Into<String>,
{
    fn from(s: S) -> Self {
        Self::new(s.into())
    }
}
