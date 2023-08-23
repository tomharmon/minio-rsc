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
use bytes::{Bytes, BytesMut};
use chrono::{DateTime, Utc};
use futures_core::Stream;
use futures_util::{stream, StreamExt, TryStreamExt};
use hyper::{header, header::HeaderValue, HeaderMap};
use hyper::{Body, Method, Uri};
use regex::Regex;
use reqwest::Response;

/// A `Builder` can be used to create a [`Minio`] with custom configuration.
pub struct Builder {
    endpoint: Option<String>,
    // access_key: Option<String>,
    // secret_key: Option<String>,
    // session_token: Option<String>,
    region: String,
    agent: String,
    secure: bool,
    virtual_hosted: bool,
    multi_chunked_encoding: bool,
    provider: Option<Box<Mutex<dyn Provider>>>,
    client: Option<reqwest::Client>,
}

impl Builder {
    pub fn new() -> Self {
        Builder {
            endpoint: None,
            secure: true,
            virtual_hosted: false,
            multi_chunked_encoding: true,
            region: "us-east-1".to_string(),
            agent: "MinIO (Linux; x86_64) minio-rs".to_string(),
            provider: None,
            client: None,
        }
    }

    /// Set hostname of a S3 service.
    #[deprecated(note = "Please use the `endpoint` instead")]
    pub fn host<T: Into<String>>(mut self, host: T) -> Self {
        let host: String = host.into();
        if host.starts_with("http://") {
            self.secure = false;
            self.endpoint = Some(host[7..].into());
        } else if host.starts_with("https://") {
            self.secure = true;
            self.endpoint = Some(host[8..].into());
        } else {
            self.endpoint = Some(host);
        }
        self
    }

    /// Set endpoint of a S3 service. `hostname`
    pub fn endpoint<T: Into<String>>(mut self, endpoint: T) -> Self {
        self.endpoint = Some(endpoint.into());
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
    /// Default: `true`.
    pub fn secure(mut self, secure: bool) -> Self {
        self.secure = secure;
        self
    }

    /// Set flag to indicate to use Virtual-hosted–style or not.
    ///
    /// In a virtual-hosted–style URI, the bucket name is part of the domain name in the URL.
    /// like `https://bucket-name.s3.region-code.amazonaws.com`
    ///
    /// Default: `false`.
    ///
    /// **Note**: If the endpoint is an IP address, setting Virtual-hosted–style true will cause an error.
    pub fn virtual_hosted(mut self, virtual_hosted: bool) -> Self {
        self.virtual_hosted = virtual_hosted;
        self
    }

    /// Set flag to indicate to use multi_chunked_encoding or not.
    ///
    /// Default: `true`.
    pub fn multi_chunked_encoding(mut self, multi_chunked_encoding: bool) -> Self {
        self.multi_chunked_encoding = multi_chunked_encoding;
        self
    }

    /// Set credentials provider of your account in S3 service.
    ///
    /// **Required**.
    pub fn provider<P>(mut self, provider: P) -> Self
    where
        P: Provider + 'static,
    {
        self.provider = Some(Box::new(Mutex::new(provider)));
        self
    }

    pub fn build(self) -> std::result::Result<Minio, ValueError> {
        let endpoint = self.endpoint.ok_or("Miss endpoint")?;
        let vaild_rg = Regex::new(r"^[A-Za-z0-9_\-.]+(:\d+)?$").unwrap();
        if !vaild_rg.is_match(&endpoint) {
            return Err("Invalid endpoint".into());
        }
        let provider = self.provider.ok_or("Miss provide")?;

        let agent: HeaderValue = self
            .agent
            .parse()
            .map_err(|_| ValueError::from("Invalid agent"))?;

        let client2 = self.client.unwrap_or_else(|| {
            let mut headers = header::HeaderMap::new();
            headers.insert(header::USER_AGENT, agent.clone());
            reqwest::Client::builder()
                .default_headers(headers)
                .https_only(self.secure)
                .max_tls_version(reqwest::tls::Version::TLS_1_2)
                .build()
                .unwrap()
        });
        Ok(Minio {
            inner: Arc::new(MinioRef {
                endpoint,
                secure: self.secure,
                client2,
                virtual_hosted: self.virtual_hosted,
                multi_chunked: self.multi_chunked_encoding,
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
    endpoint: String,
    virtual_hosted: bool,
    multi_chunked: bool,
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

    /// return whether the minio uses mutli chunked encoding.
    pub(crate) fn multi_chunked(&self) -> bool {
        self.inner.multi_chunked
    }

    fn _wrap_headers(
        &self,
        headers: &mut HeaderMap,
        host: &str,
        content_sha256: &str,
        date: DateTime<Utc>,
        content_length: usize,
    ) -> Result<()> {
        headers.insert(header::HOST, host.parse()?);
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
        let uri_sign = Uri::from_str(&uri).map_err(|e| Error::ValueError(e.to_string()))?;
        let hosts = uri_sign.authority().unwrap().host();
        self._wrap_headers(&mut headers, hosts, content_sha256, date, content_length)?;
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
        let signer = SignerV4::sign_v4_authorization(
            &method,
            &uri_sign,
            region,
            "s3",
            &headers,
            credentials.access_key(),
            credentials.secret_key(),
            &content_sha256,
            &date,
        );
        headers.insert(header::AUTHORIZATION, signer.auth_header().parse()?);

        let _body = body.into_body(signer);

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

    #[inline]
    pub(super) fn scheme(&self) -> &str {
        if self.inner.secure {
            "https"
        } else {
            "http"
        }
    }

    /// build uri for bucket/key
    ///
    /// uriencode(key)
    pub(super) fn _build_uri(&self, bucket: Option<String>, key: Option<String>) -> String {
        let scheme = self.scheme();
        let key = key.map(|k| urlencode(&k, true));
        let endpoint = self.inner.endpoint.as_str();
        if self.inner.virtual_hosted {
            match (bucket, key) {
                (Some(b), Some(k)) => {
                    format!("{}://{}.{}/{}", scheme, b, endpoint, k)
                }
                (Some(b), None) => {
                    format!("{}://{}.{}", scheme, b, endpoint)
                }
                _ => {
                    format!("{}://{}", scheme, endpoint)
                }
            }
        } else {
            match (bucket, key) {
                (Some(b), Some(k)) => {
                    format!("{}://{}/{}/{}", scheme, endpoint, b, k)
                }
                (Some(b), None) => {
                    format!("{}://{}/{}", scheme, endpoint, b)
                }
                _ => {
                    format!("{}://{}", scheme, endpoint)
                }
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
        let mut body = body.into();
        if !self.inner.multi_chunked {
            body = body.convert().await?;
        }
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

impl Data {
    // convert Stream Data into Bytes Data
    async fn convert(self) -> Result<Self> {
        Ok(match self {
            Data::Stream(mut s, l) => {
                let mut buf = BytesMut::with_capacity(l);
                while let Some(data) = s.next().await {
                    let data = data?;
                    buf.extend_from_slice(&data);
                }
                Data::Bytes(buf.freeze())
            }
            _ => self,
        })
    }

    fn into_body(self, mut signer: SignerV4) -> Body {
        match self {
            Data::Empty => Body::empty(),
            Data::Bytes(b) => Body::from(b),
            Data::Stream(s, _) => Body::wrap_stream(
                s.chain(stream::iter(vec![Ok(Bytes::new())]))
                    .map_ok(move |f| signer.sign_next_chunk(f))
                    .flat_map(|f| {
                        stream::iter(match f {
                            Ok(d) => d.into_iter().map(|f| Ok(f)).collect(),
                            Err(e) => vec![Err(e)],
                        })
                    }),
            ),
        }
    }
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
