use hyper::header::IntoHeaderName;
use hyper::{HeaderMap, Method};
use reqwest::Response;
// mod bucket_executor;
// mod object_executor;
use crate::client::{Data, Minio};
use crate::errors::S3Error;
use crate::{errors::Result, types::QueryMap};
// pub use bucket_executor::*;
// pub use object_executor::*;

pub struct BaseExecutor<'a> {
    method: Method,
    region: String,
    bucket_name: Option<String>,
    object_name: Option<String>,
    body: Option<Data>,
    headers: HeaderMap,
    querys: QueryMap,
    client: &'a Minio,
}

impl<'a> BaseExecutor<'a> {
    pub fn new(method: Method, client: &'a Minio) -> Self {
        return Self {
            method,
            region: client.region().to_string(),
            bucket_name: None,
            object_name: None,
            body: None,
            headers: HeaderMap::new(),
            client,
            querys: QueryMap::new(),
        };
    }

    pub fn method(mut self, method: Method) -> Self {
        self.method = method;
        self
    }

    pub fn bucket_name<T: Into<String>>(mut self, name: T) -> Self {
        self.bucket_name = Some(name.into());
        self
    }

    pub fn object_name<T: Into<String>>(mut self, name: T) -> Self {
        self.object_name = Some(name.into());
        self
    }

    pub fn region<T: Into<String>>(mut self, region: T) -> Self {
        self.region = region.into();
        self
    }

    pub fn body<B: Into<Data>>(mut self, body: B) -> Self {
        self.body = Some(body.into());
        self
    }

    pub fn headers(mut self, header: HeaderMap) -> Self {
        self.headers = header;
        self
    }

    pub fn header<K>(mut self, key: K, value: &str) -> Self
    where
        K: IntoHeaderName,
    {
        if let Ok(value) = value.parse() {
            self.headers.insert(key, value);
        }
        self
    }

    pub fn headers_merge(mut self, header: &HeaderMap) -> Self {
        for (k, v) in header {
            self.headers.insert(k, v.to_owned());
        }
        self
    }

    pub fn headers_merge2(self, header: Option<&HeaderMap>) -> Self {
        if let Some(header) = header {
            self.headers_merge(header)
        } else {
            self
        }
    }

    pub fn querys(mut self, querys: QueryMap) -> Self {
        self.querys = querys;
        self
    }

    pub fn querys_merge(mut self, querys: QueryMap) -> Self {
        self.querys.merge(querys);
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

    pub async fn send(self) -> Result<Response> {
        let query = self.querys.into();
        self.client
            ._execute(
                self.method,
                &self.region,
                self.bucket_name,
                self.object_name,
                self.body.unwrap_or_default(),
                Some(self.headers),
                Some(query),
            )
            .await
    }

    pub async fn send_ok(self) -> Result<Response> {
        let res = self.send().await?;
        if res.status().is_success() {
            Ok(res)
        } else {
            let text = res.text().await.unwrap();
            let s: S3Error = text.as_str().try_into()?;
            Err(s)?
        }
    }

    pub async fn send_text_ok(self) -> Result<String> {
        let res = self.send().await?;
        let success = res.status().is_success();
        let text = res.text().await.unwrap();
        if success {
            Ok(text)
        } else {
            let s: S3Error = text.as_str().try_into()?;
            Err(s)?
        }
    }
}
