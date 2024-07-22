use bytes::Bytes;
use hyper::header::{HeaderName, HeaderValue};
use hyper::{HeaderMap, Method};
use reqwest::Response;

use super::{Minio, QueryMap};
use crate::data::Data;
use crate::datatype::{FromXml, ToXml};
use crate::error::{Error, Result, S3Error};
use crate::utils::md5sum_hash;

/// An executor builds the S3 request.
/// ```rust
/// use hyper::Method;
/// use bytes::Bytes;
/// use reqwest::Response;
/// use minio_rsc::Minio;
/// use minio_rsc::error::Result;
///
/// async fn get_object(minio:Minio)-> Result<Response> {
///     let executor = minio.executor(Method::GET);
///     let res: Response = executor
///         .bucket_name("bucket")
///         .object_name("test.txt")
///         .query("versionId", "cdabf31a-9752-4265-b137-6b3961fbaf9b")
///         .send_ok()
///         .await?;
///     Ok(res)
/// }
///
/// async fn put_object(minio:Minio, data:Bytes)-> Result<()> {
///     let executor = minio.executor(Method::PUT);
///     let res: Response = executor
///         .bucket_name("bucket")
///         .object_name("test.txt")
///         .body(data)
///         .send_ok()
///         .await?;
///     Ok(())
/// }
/// ```
pub struct BaseExecutor<'a> {
    method: Method,
    region: String,
    bucket_name: Option<String>,
    object_name: Option<String>,
    body: Data<Error>,
    headers: HeaderMap,
    querys: QueryMap,
    client: &'a Minio,
    build_err: Result<()>,
}

impl<'a> BaseExecutor<'a> {
    pub fn new(method: Method, client: &'a Minio) -> Self {
        return Self {
            method,
            region: client.region().to_string(),
            bucket_name: None,
            object_name: None,
            body: Default::default(),
            headers: HeaderMap::new(),
            client,
            querys: QueryMap::new(),
            build_err: Ok(()),
        };
    }

    /// Set the request method.
    pub fn method(mut self, method: Method) -> Self {
        self.method = method;
        self
    }

    /// Set the bucket name.
    pub fn bucket_name<T: Into<String>>(mut self, name: T) -> Self {
        self.bucket_name = Some(name.into());
        self
    }

    /// Set the object name.
    pub fn object_name<T: Into<String>>(mut self, name: T) -> Self {
        self.object_name = Some(name.into());
        self
    }

    /// Set the region.
    pub fn region<T: Into<String>>(mut self, region: T) -> Self {
        self.region = region.into();
        self
    }

    /// Set the request body.
    pub fn body<B: Into<Data<Error>>>(mut self, body: B) -> Self {
        self.body = body.into();
        self
    }

    /// Set the xml struct to body and set md5 header.
    pub(crate) fn xml<'de, S>(mut self, xml: &'de S) -> Self
    where
        S: ToXml,
    {
        let xml = match xml.to_xml() {
            Ok(xml) => xml,
            Err(e) => {
                self.build_err = Err(e);
                return self;
            }
        };
        let body = Bytes::from(xml);
        let md5 = md5sum_hash(&body);
        self.body(body).header("Content-MD5", md5)
    }

    /// Set the new request header.
    pub fn headers(mut self, header: HeaderMap) -> Self {
        self.headers = header;
        self
    }

    /// Inserts a key-value pair into the request header.
    pub fn header<K, V>(mut self, key: K, value: V) -> Self
    where
        HeaderName: TryFrom<K>,
        <HeaderName as TryFrom<K>>::Error: Into<crate::error::Error>,
        HeaderValue: TryFrom<V>,
        <HeaderValue as TryFrom<V>>::Error: Into<crate::error::Error>,
    {
        let key = <HeaderName as TryFrom<K>>::try_from(key).map_err(Into::into);
        let value = <HeaderValue as TryFrom<V>>::try_from(value).map_err(Into::into);
        match (key, value) {
            (Ok(key), Ok(val)) => {
                self.headers.insert(key, val);
            }
            (Err(e), _) => self.build_err = Err(e),
            (_, Err(e)) => self.build_err = Err(e),
        };
        self
    }

    /// Merge header into request header.
    #[inline]
    pub fn headers_merge(mut self, header: HeaderMap) -> Self {
        self.headers.extend(header);
        self
    }

    /// Merge header into request header.
    #[inline]
    pub fn headers_merge2(self, header: Option<HeaderMap>) -> Self {
        if let Some(header) = header {
            self.headers_merge(header)
        } else {
            self
        }
    }

    /// Set up a new request query.
    pub fn querys(mut self, querys: QueryMap) -> Self {
        self.querys = querys;
        self
    }

    /// Merge querys into request query.
    pub fn querys_merge(mut self, querys: QueryMap) -> Self {
        self.querys.merge(querys);
        self
    }

    /// Inserts a key-value pair into the query map.
    pub fn query<K: Into<String>, V: Into<String>>(mut self, key: K, value: V) -> Self {
        self.querys.insert(key.into(), value.into());
        self
    }

    /// Inserts query_string into the query map.
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

    /// Send an HTTP request to S3 and return a Result<[Response]>.
    ///
    /// note: this is just a response from the s3 service, probably a wrong response.
    pub async fn send(self) -> Result<Response> {
        self.build_err?;
        let query = self.querys.to_query_string();
        self.client
            ._execute(
                self.method,
                &self.region,
                self.bucket_name,
                self.object_name,
                self.body,
                Some(self.headers),
                Some(query),
            )
            .await
    }

    /// Send an HTTP request to S3 and return a Result<[Response]>.
    ///
    /// This checks if the request is a legitimate S3 response.
    pub async fn send_ok(self) -> Result<Response> {
        let res = self.send().await?;
        if res.status().is_success() {
            Ok(res)
        } else {
            let text = res.text().await?;
            let s: S3Error = text.as_str().try_into()?;
            Err(s)?
        }
    }

    /// Send an HTTP request to S3 and return a Result<[String]>.
    ///
    /// This checks if the request is a legitimate S3 response.
    pub async fn send_text_ok(self) -> Result<String> {
        let res = self.send_ok().await?;
        let text = res.text().await?;
        Ok(text)
    }

    /// Send an HTTP request to S3 and conver to xml struct.
    ///
    /// This checks if the request is a legitimate S3 response.
    pub(crate) async fn send_xml_ok<T>(self) -> Result<T>
    where
        T: FromXml,
    {
        self.send_text_ok()
            .await
            .map(T::from_xml)?
            .map_err(Into::into)
    }
}
