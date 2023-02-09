use chrono::{DateTime, Utc};
use hyper::{header::IntoHeaderName, HeaderMap, Method};

use crate::{client::Minio, errors::Result, types::QueryMap};

#[derive(Clone)]
pub struct PresignedExecutor<'a> {
    region: String,
    bucket_name: String,
    object_name: String,
    version_id: Option<String>,
    expires: usize,
    request_date: Option<DateTime<Utc>>,
    headers: Option<HeaderMap>,
    querys: QueryMap,
    client: &'a Minio,
}

impl<'a> PresignedExecutor<'a> {
    pub fn new<T1: Into<String>, T2: Into<String>>(
        client: &'a Minio,
        bucket_name: T1,
        object_name: T2,
    ) -> Self {
        return Self {
            // method,
            region: client.region().to_string(),
            bucket_name: bucket_name.into(),
            object_name: object_name.into(),
            expires: 604800,
            request_date: None,
            headers: None,
            client,
            querys: QueryMap::new(),
            version_id: None,
        };
    }

    pub fn bucket_name<T: Into<String>>(&mut self, name: T) -> &mut Self {
        self.bucket_name = name.into();
        self
    }

    pub fn object_name<T: Into<String>>(mut self, name: T) -> Self {
        self.object_name = name.into();
        self
    }

    pub fn region<T: Into<String>>(mut self, region: T) -> Self {
        self.region = region.into();
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

    async fn generate(self, method: Method) -> Result<String> {
        self.client
            ._get_presigned_url(
                method,
                self.bucket_name,
                self.object_name,
                self.expires,
                self.headers,
                self.request_date,
                self.version_id,
                Some(self.querys),
            )
            .await
    }

    /// Get presigned URL of an object to download its data.
    pub async fn get(self) -> Result<String> {
        self.generate(Method::GET).await
    }

    /**
    Get presigned URL of an object to upload data.
    # Example
    ``` rust
    # use minio_rsc::Minio;
    # async fn example(minio: Minio){
    let upload_object_url :String = minio.presigned_object("bucket", "file.txt")
        .version_id("version_id")
        .expires(24*3600)
        .put()
        .await.unwrap();
    # }
    ```
     */
    pub async fn put(self) -> Result<String> {
        self.generate(Method::PUT).await
    }
}
