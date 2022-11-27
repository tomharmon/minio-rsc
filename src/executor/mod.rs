use std::pin::Pin;

use futures::Future;
use hyper::header::IntoHeaderName;
use hyper::{HeaderMap, Method};
use reqwest::Response;
mod get_object_executor;
use crate::client::Minio;
use crate::{errors::Result, types::QueryMap};
pub use get_object_executor::*;

pub trait Executor<'a> {
    type Data;
    fn send(self) -> Pin<Box<dyn Future<Output = Result<Self::Data>> + 'a>>;
}

#[derive(Clone)]
pub struct BaseExecutor<'a> {
    method: Method,
    region: String,
    bucket_name: Option<String>,
    object_name: Option<String>,
    body: Option<Vec<u8>>,
    headers: Option<HeaderMap>,
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
            headers: None,
            client,
            querys: QueryMap::new(),
        };
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

    pub fn body(mut self, body: Vec<u8>) -> Self {
        self.body = Some(body);
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

    pub fn query_string<T: Into<String>>(mut self, query_params: T) -> Self {
        let ss: String = query_params.into();
        for s in ss.split("&").filter(|x| !x.is_empty()) {
            let index = s.find("=");
            if let Some(i) = index {
                self.querys.insert(&s[0..i], &s[i + 1..]);
            } else {
                self.querys.insert(s, "");
            }
        }
        self
    }

    pub async fn send(self) -> Result<Response> {
        let query = self.querys.into();
        self.client
            ._execute(
                self.method,
                &self.region,
                self.bucket_name,
                self.object_name,
                self.body,
                self.headers,
                Some(query),
            )
            .await
    }
}
