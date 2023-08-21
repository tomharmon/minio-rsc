use std::pin::Pin;
use std::str::FromStr;
use std::sync::Arc;

use crate::errors::{Error, Result, ValueError};
use crate::executor::BaseExecutor;
use crate::provider::Provider;
use crate::signer::{sha256_hash, SignerV4};
use crate::time::aws_format_time;
use crate::utils::{check_bucket_name, urlencode, EMPTY_CONTENT_SHA256};
use crate::Credentials;
use async_mutex::Mutex;
use bytes::Bytes;
use chrono::{DateTime, Utc};
use futures_core::Stream;
use futures_util::stream;
use futures_util::{self, StreamExt, TryStreamExt};
use hyper::{header, header::HeaderValue, HeaderMap};
use hyper::{Body, Method, Uri};
use regex::Regex;
use reqwest::Response;

/// A `Builder` can be used to create a [`Minio`] with custom configuration.
pub struct Builder {
    host: Option<String>,
    // access_key: Option<String>,
    // secret_key: Option<String>,
    // session_token: Option<String>,
    region: String,
    agent: String,
    secure: bool,
    provider: Option<Box<Mutex<dyn Provider>>>,
    client: Option<reqwest::Client>,
}

impl Builder {
    pub fn new() -> Self {
        Builder {
            host: None,
            secure: true,
            region: "us-east-1".to_string(),
            agent: "MinIO (Linux; x86_64) minio-rs".to_string(),
            provider: None,
            client: None,
        }
    }

    /// Set hostname of a S3 service. `[http(s)://]hostname`
    pub fn host<T: Into<String>>(mut self, host: T) -> Self {
        self.host = Some(host.into());
        self
    }

    /// Set region name of buckets in S3 service.
    ///
    /// Default: `us-east-1`
    pub fn region<T: Into<String>>(mut self, region: T) -> Self {
        self.region = region.into();
        self
    }

    /// Set agent header for minio client.
    ///
    /// Default: `MinIO (Linux; x86_64) minio-rs`
    pub fn agent<T: Into<String>>(mut self, agent: T) -> Self {
        self.agent = agent.into();
        self
    }

    /// Set flag to indicate to use secure (TLS) connection to S3 service or not.
    ///
    /// Default: false.
    ///
    /// If host start with http or https. This setting will be ignored.
    pub fn secure(mut self, secure: bool) -> Self {
        self.secure = secure;
        self
    }

    /// Set credentials provider of your account in S3 service.
    ///
    /// Required.
    pub fn provider<P>(mut self, provider: P) -> Self
    where
        P: Provider + 'static,
    {
        self.provider = Some(Box::new(Mutex::new(provider)));
        self
    }

    pub fn build(self) -> std::result::Result<Minio, ValueError> {
        let host = self.host.ok_or("Miss host")?;
        let vaild_rg = Regex::new(r"^(http(s)?://)?[A-Za-z0-9_\-.]+(:\d+)?$").unwrap();
        if !vaild_rg.is_match(&host) {
            return Err("Invalid hostname".into());
        }
        let provider = if let Some(provier) = self.provider {
            provier
        } else {
            return Err(ValueError::from("Miss provide"));
        };
        let (host, secure) = if host.starts_with("https://") {
            (host[8..].to_owned(), true)
        } else if host.starts_with("http://") {
            (host[7..].to_owned(), false)
        } else {
            (host, self.secure)
        };

        let agent: HeaderValue = self
            .agent
            .parse()
            .map_err(|_| ValueError::from("Invalid agent"))?;

        let client2 = if let Some(client) = self.client {
            client
        } else {
            let mut headers = header::HeaderMap::new();
            let host = host.parse().map_err(|_| ValueError::from("Invalid host"))?;
            headers.insert(header::HOST, host);
            headers.insert(header::USER_AGENT, agent.clone());
            reqwest::Client::builder()
                .default_headers(headers)
                .https_only(secure)
                .max_tls_version(reqwest::tls::Version::TLS_1_2)
                .build()
                .unwrap()
        };
        Ok(Minio {
            inner: Arc::new(MinioRef {
                host: format!("http{}://{}", if self.secure { "s" } else { "" }, host),
                secure,
                client2,
                region: self.region,
                agent,
                provider,
            }),
        })
    }
}

/// Simple Storage Service (aka S3) client to perform bucket and object operations.
///
/// You do **not** have to wrap the `Minio` in an [`Rc`] or [`Arc`] to **reuse** it,
/// because it already uses an [`Arc`] internally.
///
/// # Create Minio client
/// ```rust
/// use minio_rsc::{provider::StaticProvider,Minio};
/// let provider = StaticProvider::new("minio-access-key-test", "minio-secret-key-test", None);
/// let minio = Minio::builder()
///     .host("localhost:9022")
///     .provider(provider)
///     .secure(false)
///     .build()
///     .unwrap();
///
/// ```
#[derive(Clone)]
pub struct Minio {
    inner: Arc<MinioRef>,
}

struct MinioRef {
    host: String,
    secure: bool,
    client2: reqwest::Client,
    region: String,
    agent: HeaderValue,
    provider: Box<Mutex<dyn Provider>>,
}

impl Minio {
    /// get a minio [`Builder`]
    pub fn builder() -> Builder {
        Builder::new()
    }

    fn _wrap_headers(
        &self,
        headers: &mut HeaderMap,
        content_sha256: &str,
        date: DateTime<Utc>,
        content_length: usize,
    ) -> Result<()> {
        let i = if self.inner.secure { 8 } else { 7 };
        headers.insert(header::HOST, self.inner.host[i..].parse()?);
        headers.insert(header::USER_AGENT, self.inner.agent.clone());
        if content_length > 0 {
            headers.insert(header::CONTENT_LENGTH, content_length.to_string().parse()?);
        };
        headers.insert("x-amz-content-sha256", content_sha256.parse()?);
        headers.insert("x-amz-date", aws_format_time(&date).parse()?);
        Ok(())
    }

    pub fn region(&self) -> &str {
        self.inner.region.as_ref()
    }

    fn _get_region<T: Into<String>>(&self, bucket_name: Option<T>) -> String {
        self.inner.region.clone()
    }

    #[inline]
    pub(super) async fn fetch_credentials(&self) -> Credentials {
        self.inner.provider.lock().await.fetct().await
    }

    /// Execute HTTP request.
    async fn _url_open(
        &self,
        method: Method,
        uri: &str,
        region: &str,
        body: Data,
        headers: Option<HeaderMap>,
    ) -> Result<Response> {
        // build header
        let mut headers = headers.unwrap_or(HeaderMap::new());

        let mut _hash = Default::default();
        let (content_sha256, content_length) = match &body {
            Data::Empty => (EMPTY_CONTENT_SHA256, 0),
            Data::Bytes(body) => {
                let length = body.len();
                _hash = sha256_hash(&body);
                (_hash.as_str(), length)
            }
            Data::Stream(_, _) => ("STREAMING-AWS4-HMAC-SHA256-PAYLOAD", 0),
        };

        let date: DateTime<Utc> = Utc::now();

        self._wrap_headers(&mut headers, content_sha256, date, content_length)?;
        match &body {
            Data::Stream(_, len) => {
                // headers.insert(header::TRANSFER_ENCODING, "identity".parse()?);
                headers.insert(header::CONTENT_ENCODING, "aws-chunked".parse()?);
                headers.insert("x-amz-decoded-content-length", len.to_string().parse()?);
            }
            _ => {}
        };

        // add authorization header
        let credentials = self.fetch_credentials().await;
        let mut singer = SignerV4::sign_v4_authorization(
            &method,
            &Uri::from_str(&uri).map_err(|e| Error::ValueError(e.to_string()))?,
            region,
            "s3",
            &headers,
            credentials.access_key(),
            credentials.secret_key(),
            &content_sha256,
            &date,
        );
        headers.insert(header::AUTHORIZATION, singer.auth_header().parse()?);

        let _body = match body {
            Data::Empty => Body::empty(),
            Data::Bytes(b) => Body::from(b),
            Data::Stream(s, _) => Body::wrap_stream(
                s.chain(stream::iter(vec![Ok(Bytes::new())]))
                    .map_ok(move |f| singer.sign_next_chunk(f))
                    .flat_map(|f| {
                        stream::iter(match f {
                            Ok(d) => d.into_iter().map(|f| Ok(f)).collect(),
                            Err(e) => vec![Err(e)],
                        })
                    }),
            ),
        };
        // build and send request
        let request = self
            .inner
            .client2
            .request(method, uri)
            .headers(headers)
            .body(_body)
            .send()
            .await?;

        Ok(request)
    }

    /// build uri for bucket/key
    ///
    /// uriencode(key)
    pub(super) fn _build_uri(&self, bucket: Option<String>, key: Option<String>) -> String {
        match (bucket, key) {
            (Some(b), Some(k)) => {
                format!("{}/{}/{}", self.inner.host, b, urlencode(&k, true))
            }
            (Some(b), None) => {
                format!("{}/{}/", self.inner.host, b)
            }
            _ => {
                format!("{}/", self.inner.host)
            }
        }
    }

    pub async fn _execute<B: Into<Data>>(
        &self,
        method: Method,
        region: &str,
        bucket_name: Option<String>,
        object_name: Option<String>,
        body: B,
        headers: Option<HeaderMap>,
        query_params: Option<String>,
    ) -> Result<Response> {
        // check bucket_name
        if let Some(bucket_name) = &bucket_name {
            check_bucket_name(bucket_name)?;
        }
        // check object_name
        if let Some(object_name) = &object_name {
            if object_name.is_empty() {
                Err(ValueError::from("Object name cannot be empty."))?
            }
            if bucket_name.is_none() {
                Err(ValueError::from("Miss bucket name."))?
            }
        }
        // build uri
        let uri = self._build_uri(bucket_name, object_name);

        // add query to uri
        let uri = if let Some(query) = query_params {
            format!("{}?{}", uri, query)
        } else {
            uri
        };
        let body = body.into();
        self._url_open(method, &uri, region, body, headers).await
    }

    #[inline]
    pub fn executor(&self, method: Method) -> BaseExecutor {
        BaseExecutor::new(method, self)
    }
}

/// Payload for http request
pub enum Data {
    Empty,
    /// Transferring Payload in a Single Chunk
    Bytes(Bytes),
    /// Transferring Payload in Multiple Chunks, `usize` the total byte length of the stream.
    Stream(Pin<Box<dyn Stream<Item = Result<Bytes>> + Send>>, usize),
}

impl Default for Data {
    fn default() -> Self {
        Self::Empty
    }
}

impl From<Option<Bytes>> for Data {
    fn from(value: Option<Bytes>) -> Self {
        match value {
            Some(v) => Self::Bytes(v),
            None => Self::Empty,
        }
    }
}

impl From<Bytes> for Data {
    fn from(value: Bytes) -> Self {
        Self::Bytes(value)
    }
}

impl From<String> for Data {
    fn from(value: String) -> Self {
        Self::Bytes(value.into())
    }
}

impl From<&'static str> for Data {
    fn from(value: &'static str) -> Self {
        Self::Bytes(value.into())
    }
}

impl From<Vec<u8>> for Data {
    fn from(value: Vec<u8>) -> Self {
        Self::Bytes(value.into())
    }
}

impl From<(Pin<Box<dyn Stream<Item = Result<Bytes>> + Send>>, usize)> for Data {
    fn from(value: (Pin<Box<dyn Stream<Item = Result<Bytes>> + Send>>, usize)) -> Self {
        Self::Stream(value.0, value.1)
    }
}
