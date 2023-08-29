use hyper::HeaderMap;

use super::super::QueryMap;

use super::BaseArgs;

/// Custom listobjects request parameters Bucket
/// ## parmas
/// - prefix: Limits the response to keys that begin with the specified prefix.
/// - delimiter: A delimiter is a character you use to group keys.
/// - continuation_token: ContinuationToken indicates Amazon S3 that the list is being continued on this bucket with a token.
/// - max_keys: Sets the maximum number of keys returned in the response. Default 1000
/// - encoding_type:Encoding type used by Amazon S3 to encode object keys in the response.Valid Values: `url`
/// - expected_bucket_owner: The account ID of the expected bucket owner.
pub struct ListObjectsArgs {
    pub(crate) bucket_name: String,
    pub(crate) continuation_token: Option<String>,
    pub(crate) delimiter: Option<String>,
    pub(crate) use_encoding_type: bool,
    pub(crate) fetch_owner: bool,
    pub(crate) start_after: Option<String>,
    pub(crate) max_keys: usize,
    pub(crate) prefix: Option<String>,
    pub(crate) expected_bucket_owner: Option<String>,
    pub(crate) extra_headers: Option<HeaderMap>,
}

impl ListObjectsArgs {
    pub(crate) fn default() -> Self {
        Self {
            bucket_name: "".to_string(),
            continuation_token: None,
            delimiter: None,
            fetch_owner: false,
            max_keys: 1000,
            prefix: None,
            start_after: None,
            use_encoding_type: false,
            expected_bucket_owner: None,
            extra_headers: None,
        }
    }

    pub fn new<S: Into<String>>(bucket_name: S) -> Self {
        Self {
            bucket_name: bucket_name.into(),
            continuation_token: None,
            delimiter: None,
            fetch_owner: false,
            max_keys: 1000,
            prefix: None,
            start_after: None,
            use_encoding_type: false,
            expected_bucket_owner: None,
            extra_headers: None,
        }
    }

    pub fn continuation_token<T: Into<String>>(mut self, token: T) -> Self {
        self.continuation_token = Some(token.into());
        self
    }

    pub fn delimiter<T: Into<String>>(mut self, delimiter: T) -> Self {
        self.delimiter = Some(delimiter.into());
        self
    }

    pub fn use_encoding_type(mut self, use_encoding_type: bool) -> Self {
        self.use_encoding_type = use_encoding_type;
        self
    }

    pub fn fetch_owner(mut self, fetch_owner: bool) -> Self {
        self.fetch_owner = fetch_owner;
        self
    }

    pub fn start_after<T: Into<String>>(mut self, start_after: T) -> Self {
        self.start_after = Some(start_after.into());
        self
    }

    pub fn max_keys(mut self, max_keys: usize) -> Self {
        self.max_keys = max_keys;
        if self.max_keys > 1000 {
            self.max_keys = 1000;
        }
        self
    }

    pub fn prefix<T: Into<String>>(mut self, prefix: T) -> Self {
        self.prefix = Some(prefix.into());
        self
    }

    pub fn expected_bucket_owner<T: Into<String>>(mut self, expected_bucket_owner: T) -> Self {
        self.expected_bucket_owner = Some(expected_bucket_owner.into());
        self
    }
}

impl<S> From<S> for ListObjectsArgs
where
    S: Into<String>,
{
    fn from(s: S) -> Self {
        Self::new(s)
    }
}

impl BaseArgs for ListObjectsArgs {
    fn extra_query_map(&self) -> QueryMap {
        let mut querys: QueryMap = QueryMap::default();
        querys.insert("list-type".to_string(), "2".to_string());

        if self.use_encoding_type {
            querys.insert("encoding-type".to_string(), "url".to_string());
        }
        if let Some(delimiter) = &self.delimiter {
            querys.insert("delimiter".to_string(), delimiter.clone());
        }
        if let Some(token) = &self.continuation_token {
            querys.insert("continuation-token".to_string(), token.clone());
        }
        if self.fetch_owner {
            querys.insert("fetch-owner".to_string(), "true".to_string());
        }
        if let Some(prefix) = &self.prefix {
            querys.insert("prefix".to_string(), prefix.clone());
        }
        if let Some(start_after) = &self.start_after {
            querys.insert("start-after".to_string(), start_after.clone());
        }
        querys.insert("max-keys".to_string(), format!("{}", self.max_keys));
        return querys;
    }
}
