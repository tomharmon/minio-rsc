use std::str::FromStr;
use std::sync::Arc;

use crate::data::Data;
use crate::error::{Error, Result, ValueError};
use crate::provider::Provider;
use crate::signer::sign_request_v4;
use crate::utils::{check_bucket_name, urlencode, _VALID_ENDPOINT};
use crate::Credentials;
use hyper::{header, header::HeaderValue, HeaderMap};
use hyper::{Method, Uri};
use reqwest::{Body, Response};

use super::{Bucket, BucketArgs};

/// A `MinioBuilder` can be used to create a [`Minio`] with custom configuration.
pub struct MinioBuilder {
    endpoint: Option<String>,
    // access_key: Option<String>,
    // secret_key: Option<String>,
    // session_token: Option<String>,
    region: String,
    agent: String,
    secure: bool,
    virtual_hosted: bool,
    multi_chunked_encoding: bool,
    provider: Option<Box<dyn Provider>>,
    client: Option<reqwest::Client>,
}

impl MinioBuilder {
    pub fn new() -> Self {
        MinioBuilder {
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

    /// Set custom http [reqwest::Client].
    pub fn client(mut self, client: reqwest::Client) -> Self {
        self.client = Some(client);
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
    pub fn virtual_hosted_style(mut self, virtual_hosted_style: bool) -> Self {
        self.virtual_hosted = virtual_hosted_style;
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
        self.provider = Some(Box::new(provider));
        self
    }

    pub fn build(self) -> std::result::Result<Minio, ValueError> {
        let endpoint = self.endpoint.ok_or("Miss endpoint")?;
        if !_VALID_ENDPOINT.is_match(&endpoint) {
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
/// ## Create Minio client
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
    provider: Box<dyn Provider>,
}

impl Minio {
    /// get a minio [`MinioBuilder`]
    pub fn builder() -> MinioBuilder {
        MinioBuilder::new()
    }

    /// return whether the minio uses mutli chunked encoding.
    pub(crate) fn multi_chunked(&self) -> bool {
        self.inner.multi_chunked
    }

    pub fn region(&self) -> &str {
        self.inner.region.as_ref()
    }

    fn _get_region<T: Into<String>>(&self, bucket_name: Option<T>) -> String {
        self.inner.region.clone()
    }

    #[inline]
    pub(super) async fn fetch_credentials(&self) -> Credentials {
        self.inner.provider.fetch().await
    }

    /// Execute HTTP request.
    async fn _url_open(
        &self,
        method: Method,
        uri: String,
        headers: HeaderMap,
        body: Body,
    ) -> Result<Response> {
        let request = self
            .inner
            .client2
            .request(method, uri)
            .headers(headers)
            .body(body)
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
        let endpoint = self.inner.endpoint.as_str();
        match bucket {
            Some(b) => {
                let mut uri = if self.inner.virtual_hosted {
                    format!("{scheme}://{b}.{endpoint}")
                } else {
                    format!("{scheme}://{endpoint}/{b}",)
                };
                if let Some(key) = key {
                    uri.push('/');
                    uri.push_str(&urlencode(&key, true));
                }
                uri
            }
            None => format!("{scheme}://{endpoint}"),
        }
    }

    pub async fn _execute<B: Into<Data<crate::error::Error>>>(
        &self,
        method: Method,
        region: &str,
        bucket_name: Option<String>,
        object_name: Option<String>,
        data: B,
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
        let mut data = data.into();
        if !self.inner.multi_chunked {
            data = data.convert().await?;
        }
        let mut headers = headers.unwrap_or(HeaderMap::new());
        headers.insert(header::USER_AGENT, self.inner.agent.clone());
        let credentials = self.fetch_credentials().await;
        let uri = Uri::from_str(&uri).map_err(|e| Error::ValueError(e.to_string()))?;
        let (uri, body) = sign_request_v4(
            &method,
            &uri,
            &mut headers,
            region,
            data,
            credentials.access_key(),
            credentials.secret_key(),
        )?;
        self._url_open(method, uri, headers, body).await
    }

    #[inline]
    pub fn executor(&self, method: Method) -> super::BaseExecutor {
        super::BaseExecutor::new(method, self)
    }

    /// Instantiate an [Bucket]
    pub fn bucket<B>(&self, bucket: B) -> Bucket
    where
        B: Into<BucketArgs>,
    {
        Bucket {
            client: self.clone(),
            bucket: bucket.into(),
        }
    }
}
