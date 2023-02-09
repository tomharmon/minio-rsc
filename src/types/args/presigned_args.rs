use chrono::{DateTime, Utc};
use hyper::{header::IntoHeaderName, HeaderMap};

use crate::types::QueryMap;

/// Custom request parameters for presigned URL
/// ## param 
/// - bucket_name: Name of the bucket.
/// - object_name: Object name in the bucket.
/// - expires: Expiry in seconds; defaults to 7 days.
/// - headers: Optional response_headers argument to specify response fields like date, size, type of file, data about server, etc.
/// - request_date: Optional request_date argument to specify a different request date. Default is current date.
/// - version_id: Version ID of the object.
/// - querys: Extra query parameters for advanced usage.
#[derive(Clone)]
pub struct PresignedArgs {
    pub(crate) region: Option<String>,
    pub(crate) bucket_name: String,
    pub(crate) object_name: String,
    pub(crate) version_id: Option<String>,
    pub(crate) expires: usize,
    pub(crate) request_date: Option<DateTime<Utc>>,
    pub(crate) headers: Option<HeaderMap>,
    pub(crate) querys: QueryMap,
}

impl PresignedArgs {
    pub fn new<T1: Into<String>, T2: Into<String>>(bucket_name: T1, object_name: T2) -> Self {
        Self {
            region: None,
            bucket_name: bucket_name.into(),
            object_name: object_name.into(),
            version_id: None,
            expires: 604800,
            request_date: None,
            headers: None,
            querys: QueryMap::new(),
        }
    }

    pub fn region<T: Into<String>>(mut self, region: T) -> Self {
        self.region = Some(region.into());
        self
    }

    pub fn version_id<T: Into<String>>(mut self, version_id: T) -> Self {
        self.version_id = Some(version_id.into());
        self
    }

    pub fn regirequest_date(mut self, request_date: DateTime<Utc>) -> Self {
        self.request_date = Some(request_date);
        self
    }

    pub fn expires(mut self, expires: usize) -> Self {
        self.expires = expires;
        self
    }

    pub fn headers(mut self, header: HeaderMap) -> Self {
        self.headers = Some(header);
        self
    }

    pub fn header<K>(mut self, key: K, value: &str) -> Self
    where
        K: IntoHeaderName,
    {
        let mut headers = self.headers.unwrap_or(HeaderMap::new());
        if let Ok(value) = value.parse() {
            headers.insert(key, value);
        }
        self.headers = Some(headers);
        self
    }

    pub fn querys(mut self, querys: QueryMap) -> Self {
        self.querys = querys;
        self
    }

    pub fn query<K: Into<String>, V: Into<String>>(mut self, key: K, value: V) -> Self {
        self.querys.insert(key.into(), value.into());
        self
    }

    pub fn query_string(mut self, query_str: &str) -> Self {
        self.querys.merge_str(query_str);
        self
    }

    pub fn apply<F>(self, apply: F) -> Self
    where
        F: FnOnce(Self) -> Self,
    {
        apply(self)
    }
}
