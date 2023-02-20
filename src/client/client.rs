use std::str::FromStr;
use std::sync::Arc;

use crate::errors::{Result, ValueError};
use crate::executor::BaseExecutor;
use crate::provider::Provider;
use crate::signer::{sha256_hash, sign_v4_authorization};
use crate::time::aws_format_time;
use crate::utils::{check_bucket_name, urlencode, EMPTY_CONTENT_SHA256};
use crate::Credentials;
use chrono::{DateTime, Utc};
use hyper::{header, header::HeaderValue, HeaderMap};
use hyper::{Body, Method, Uri};
use regex::Regex;
use reqwest::Response;
use tokio::sync::Mutex;

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

    // Set credentials provider of your account in S3 service.
    pub fn provider<P>(mut self, provider: P) -> Self
    where
        P: Provider + 'static,
    {
        self.provider = Some(Box::new(Mutex::new(provider)));
        self
    }

    pub fn build(self) -> std::result::Result<Minio, ValueError> {
        if let Some(host) = self.host {
            let vaild_rg = Regex::new(r"^(http(s)?://)?(www\.)?[a-zA-Z0-9][-a-zA-Z0-9]{0,62}(\.[a-zA-Z0-9][-a-zA-Z0-9]{0,62})?(:\d+)*(/\w+\.\w+)*$").unwrap();
            if !vaild_rg.is_match(&host) {
                return Err("Invalid hostname".into());
            }
            let provider = if let Some(provier) = self.provider {
                provier
            } else {
                return Err(ValueError::from("miss provide"));
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
                .map_err(|_| ValueError::from("invalid agent"))?;

            let client2 = if let Some(client) = self.client {
                client
            } else {
                let mut headers = header::HeaderMap::new();
                let host = host.parse().map_err(|_| ValueError::from("invalid host"))?;
                headers.insert(header::HOST, host);
                headers.insert(header::USER_AGENT, agent.clone());
                reqwest::Client::builder()
                    .default_headers(headers)
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
        } else {
            Err("miss host".into())
        }
    }
}

/// Simple Storage Service (aka S3) client to perform bucket and object operations.
///
/// You do **not** have to wrap the `Minio` in an [`Rc`] or [`Arc`] to **reuse** it,
/// because it already uses an [`Arc`] internally.
///
/// # Create Minio client
/** ```rust
let provider = StaticProvider::new("minio-access-key-test", "minio-secret-key-test", None);
let minio = Minio::builder()
    .host("localhost:9022")
    .provider(provider)
    .secure(false)
    .build()
    .unwrap();
*/
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
    ) {
        headers.insert(header::HOST, self.inner.host[16..].parse().unwrap());
        headers.insert(header::USER_AGENT, self.inner.agent.clone());
        if content_length > 0 {
            headers.insert(
                header::CONTENT_LENGTH,
                content_length.to_string().parse().unwrap(),
            );
        };
        headers.insert("x-amz-content-sha256", content_sha256.parse().unwrap());
        headers.insert("x-amz-date", aws_format_time(&date).parse().unwrap());
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
        body: Option<Vec<u8>>,
        headers: Option<HeaderMap>,
    ) -> Result<Response> {
        // build header
        let mut headers = headers.unwrap_or(HeaderMap::new());

        let (_body, content_sha256, content_length) = if let Some(body) = body {
            let length = body.len();
            let hash = sha256_hash(&body);
            (Body::from(body), hash, length)
        } else {
            (Body::empty(), EMPTY_CONTENT_SHA256.to_string(), 0)
        };

        let date: DateTime<Utc> = Utc::now();

        self._wrap_headers(&mut headers, &content_sha256, date, content_length);

        // add authorization header
        let credentials = self.fetch_credentials().await;
        let authorization = sign_v4_authorization(
            &method,
            &Uri::from_str(&uri).unwrap(),
            region,
            "s3",
            &headers,
            credentials.access_key(),
            credentials.secret_key(),
            &content_sha256,
            &date,
        );
        headers.insert(header::AUTHORIZATION, authorization.parse().unwrap());

        // build and send request
        let request = self
            .inner
            .client2
            .request(method, uri)
            .headers(headers)
            .body(_body)
            .send()
            .await
            .unwrap();

        Ok(request)
    }

    /// build uri for bucket_name/object_name
    /// uriencode object_name
    pub(super) fn _build_uri(
        &self,
        bucket_name: Option<String>,
        object_name: Option<String>,
    ) -> String {
        match (bucket_name, object_name) {
            (Some(b), Some(o)) => {
                format!("{}/{}/{}", self.inner.host, b, urlencode(&o, true))
            }
            (Some(b), None) => {
                format!("{}/{}/", self.inner.host, b)
            }
            _ => {
                format!("{}/", self.inner.host)
            }
        }
    }

    pub async fn _execute(
        &self,
        method: Method,
        region: &str,
        bucket_name: Option<String>,
        object_name: Option<String>,
        body: Option<Vec<u8>>,
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
        Ok(self._url_open(method, &uri, region, body, headers).await?)
    }

    pub fn executor(&self, method: Method) -> BaseExecutor {
        BaseExecutor::new(method, self)
    }
}

// impl Minio {
//     pub fn bucket<T1: Into<String>>(&self, bucket_name: T1) -> BucketExecutor {
//         return BucketExecutor::new(self, bucket_name);
//     }
// }

// /// Operating object
// impl Minio {
//     /// ObjectExecutor. Returned [ObjectExecutor](crate::executor::ObjectExecutor)
//     pub fn object<T1: Into<String>, T2: Into<String>>(
//         &self,
//         bucket_name: T1,
//         object_name: T2,
//     ) -> ObjectExecutor {
//         ObjectExecutor::new(self, bucket_name, object_name)
//     }
// }
