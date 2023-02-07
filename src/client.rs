use std::str::FromStr;
use std::sync::Arc;

use crate::errors::{Result, S3Error, ValueError, XmlError};
use crate::executor::ObjectExecutor;
use crate::executor::{BaseExecutor, BucketExecutor, GetObjectExecutor, PresignedExecutor};
use crate::provider::{Provider, StaticProvider};
use crate::signer::{presign_v4, sha256_hash, sign_v4_authorization};
use crate::time::aws_format_time;
use crate::types::response::ListAllMyBucketsResult;
use crate::types::QueryMap;
use crate::utils::{check_bucket_name, urlencode, urlencode_binary, EMPTY_CONTENT_SHA256};
use chrono::{DateTime, Utc};
use hyper::{header, header::HeaderValue, HeaderMap};
use hyper::{Body, Method, Uri};
use regex::Regex;
use reqwest::Response;
use tokio::io::{AsyncRead, AsyncReadExt};
use tokio::sync::Mutex;

/// Minio client builder
pub struct Builder {
    host: Option<String>,
    access_key: Option<String>,
    secret_key: Option<String>,
    session_token: Option<String>,
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
            access_key: None,
            secret_key: None,
            session_token: None,
            secure: true,
            region: "us-east-1".to_string(),
            agent: "MinIO (Linux; x86_64) minio-rs/0.1.0".to_string(),
            provider: None,
            client: None,
        }
    }

    pub fn host<T: Into<String>>(mut self, host: T) -> Self {
        self.host = Some(host.into());
        self
    }

    pub fn access_key<T: Into<String>>(mut self, access_key: T) -> Self {
        self.access_key = Some(access_key.into());
        self
    }

    pub fn secret_key<T: Into<String>>(mut self, secret_key: T) -> Self {
        self.secret_key = Some(secret_key.into());
        self
    }

    pub fn session_token<T: Into<String>>(mut self, session_token: T) -> Self {
        self.session_token = Some(session_token.into());
        self
    }

    pub fn region<T: Into<String>>(mut self, region: T) -> Self {
        self.region = region.into();
        self
    }

    pub fn agent<T: Into<String>>(mut self, agent: T) -> Self {
        self.agent = agent.into();
        self
    }

    pub fn secure(mut self, secure: bool) -> Self {
        self.secure = secure;
        self
    }

    pub fn provider<P>(mut self, provider: P) -> Self
    where
        P: Provider + 'static,
    {
        self.provider = Some(Box::new(Mutex::new(provider)));
        self
    }

    pub fn builder(self) -> std::result::Result<Minio, ValueError> {
        if let Some(host) = self.host {
            let vaild_rg = Regex::new(r"^(http(s)?://)?(www\.)?[a-zA-Z0-9][-a-zA-Z0-9]{0,62}(\.[a-zA-Z0-9][-a-zA-Z0-9]{0,62})?(:\d+)*(/\w+\.\w+)*$").unwrap();
            if !vaild_rg.is_match(&host) {
                return Err("Invalid hostname".into());
            }
            let provider = if let Some(provier) = self.provider {
                provier
            } else {
                if let Some(ak) = self.access_key {
                    if let Some(sk) = self.secret_key {
                        let prod = StaticProvider::new(ak, sk, self.session_token);
                        Box::new(Mutex::new(prod))
                    } else {
                        Err(ValueError::from("miss secret_key"))?
                    }
                } else {
                    Err(ValueError::from("miss access_key"))?
                }
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

#[derive(Clone)]
pub struct Minio {
    inner: Arc<MinioRef>,
}

pub struct MinioRef {
    host: String,
    secure: bool,
    client2: reqwest::Client,
    region: String,
    agent: HeaderValue,
    provider: Box<Mutex<dyn Provider>>,
}

impl Minio {
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
        &self.inner.region
    }

    fn _get_region<T: Into<String>>(&self, bucket_name: Option<T>) -> String {
        self.inner.region.clone()
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
        let credentials = self.inner.provider.lock().await.fetct().await;
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
    fn _build_uri(&self, bucket_name: Option<String>, object_name: Option<String>) -> String {
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

/// Operating the bucket
impl Minio {
    pub fn bucket<T1: Into<String>>(&self, bucket_name: T1) -> BucketExecutor {
        return BucketExecutor::new(self, bucket_name);
    }

    /// List information of all accessible buckets.
    ///
    /// return Result<[`ListAllMyBucketsResult`](crate::types::response::ListAllMyBucketsResult)>
    ///
    pub async fn list_buckets(&self) -> Result<ListAllMyBucketsResult> {
        let res = self.executor(Method::GET).send().await?;
        let text = response_ok_text(res).await?;
        text.as_str().try_into().map_err(|e: XmlError| e.into())
    }
}

pub async fn response_is_ok(res: Response) -> Result<Response> {
    if res.status().is_success() {
        Ok(res)
    } else {
        let text = res.text().await.unwrap();
        let s: S3Error = text.as_str().try_into()?;
        Err(s)?
    }
}

pub async fn response_ok_text(res: Response) -> Result<String> {
    let success = res.status().is_success();
    let text = res.text().await.unwrap();
    if success {
        Ok(text)
    } else {
        let s: S3Error = text.as_str().try_into()?;
        Err(s)?
    }
}

/// Operating object
impl Minio {
    pub fn object<T1: Into<String>, T2: Into<String>>(
        &self,
        bucket_name: T1,
        object_name: T2,
    ) -> ObjectExecutor {
        ObjectExecutor::new(self, bucket_name, object_name)
    }

    /**
    Get data of an object. Returned [GetObjectExecutor](crate::executor::GetObjectExecutor)

    # Example
    ``` rust
    # use minio_rsc::Minio;
    # async fn example(minio: Minio){
    let response = minio.get_object("bucket", "file.txt")
        .offset(3)
        .length(10)
        .version_id("version_id")
        .send()
        .await;
     let result = minio.get_object("bucket", "file.txt")
        .version_id("version_id")
        .write_to("test/file.txt")
        .await;
    # }
    ```
    */
    pub fn get_object<T1: Into<String>, T2: Into<String>>(
        &self,
        bucket_name: T1,
        object_name: T2,
    ) -> GetObjectExecutor {
        return GetObjectExecutor::new(self, bucket_name, object_name);
    }

    pub fn stat_object<T1: Into<String>, T2: Into<String>>(
        &self,
        bucket_name: T1,
        object_name: T2,
    ) -> GetObjectExecutor {
        return GetObjectExecutor::new(self, bucket_name, object_name);
    }

    pub async fn put_object<
        T1: Into<String>,
        T2: Into<String>,
        D: AsyncRead + std::marker::Unpin,
    >(
        &self,
        bucket_name: T1,
        object_name: T2,
        mut data: D,
    ) {
        let mut buf = Vec::new();
        data.read_to_end(&mut buf).await.unwrap();

        let res = self
            .executor(Method::PUT)
            .bucket_name(bucket_name)
            .object_name(object_name)
            .body(buf)
            .header(header::CONTENT_TYPE, "application/octet-stream")
            .send()
            .await
            .unwrap();
    }

    /// Remove an object.
    ///
    pub async fn remove_object<T1: Into<String>, T2: Into<String>>(
        &self,
        bucket_name: T1,
        object_name: T2,
        version_id: Option<String>,
    ) -> Result<bool> {
        let mut ext = self
            .executor(Method::DELETE)
            .bucket_name(bucket_name)
            .object_name(object_name);
        if let Some(v) = version_id {
            ext = ext.query("versionId", v)
        }
        let res = ext.send().await?;
        response_is_ok(res).await?;
        Ok(true)
    }
}

/// Operating presigned
impl Minio {
    /// Get presigned URL of an object for HTTP method, expiry time and custom request parameters.
    /// # param
    /// - method: HTTP method.
    /// - bucket_name: Name of the bucket.
    /// - object_name: Object name in the bucket.
    /// - expires: Expiry in seconds. between 1, 604800
    /// - response_headers Optional response_headers argument to specify response fields like date, size, type of file, data about server, etc.
    /// - request_date: Optional request_date argument to specify a different request date. Default is current date.
    /// - version_id: Version ID of the object.
    /// - extra_query_params: Extra query parameters for advanced usage.
    pub async fn _get_presigned_url<T1: Into<String>, T2: Into<String>>(
        &self,
        method: Method,
        bucket_name: T1,
        object_name: T2,
        expires: usize,
        response_headers: Option<HeaderMap>,
        request_date: Option<DateTime<Utc>>,
        version_id: Option<String>,
        extra_query_params: Option<QueryMap>,
    ) -> Result<String> {
        if expires < 1 || expires > 604800 {
            return Err(ValueError::from("expires must be between 1 second to 7 days").into());
        }
        let date: DateTime<Utc> = request_date.unwrap_or(Utc::now());
        let mut query = extra_query_params.unwrap_or(QueryMap::new());
        if let Some(id) = version_id {
            query.insert("versionId", id);
        }
        let credentials = self.inner.provider.lock().await.fetct().await;
        if let Some(token) = credentials.session_token() {
            query.insert("X-Amz-Security-Token", token);
        }
        if let Some(headers) = response_headers {
            for (name, value) in &headers {
                query.insert(name.to_string(), urlencode_binary(value.as_bytes(), false));
            }
        }
        let uri = self._build_uri(Some(bucket_name.into()), Some(object_name.into()));
        let uri = uri + "?" + &query.to_query_string();
        let uri = Uri::from_str(&uri).map_err(|e| ValueError::from(e))?;
        let r = presign_v4(
            &method,
            &uri,
            self.region(),
            credentials.access_key(),
            credentials.secret_key(),
            &date,
            expires,
        );
        Ok(r)
    }

    /**
    [PresignedExecutor](crate::executor::PresignedExecutor) for presigned URL of an object.
    # Example
    ``` rust
    # use minio_rsc::Minio;
    # async fn example(minio: Minio){
    // Get presigned URL of an object to download its data with expiry time.
    let get_object_url :String = minio.presigned_object("bucket", "file.txt")
        .version_id("version_id")
        .expires(24*3600)
        .get()
        .await.unwrap();
    // Get presigned URL of an object to upload data with expiry time.
    let upload_object_url :String = minio.presigned_object("bucket", "file.txt")
        .version_id("version_id")
        .expires(24*3600)
        .put()
        .await.unwrap();
    # }
    ```
     */
    pub fn presigned_object<T1: Into<String>, T2: Into<String>>(
        &self,
        bucket_name: T1,
        object_name: T2,
    ) -> PresignedExecutor {
        PresignedExecutor::new(&self, bucket_name, object_name)
    }
}

#[cfg(test)]
mod tests {
    use std::env;

    use crate::client::Minio;
    use crate::provider::StaticProvider;
    use crate::types::args::ListObjectsArgs;
    use hyper::Method;
    use tokio;

    #[tokio::main]
    #[test]
    async fn it_works() {
        dotenv::dotenv().ok();

        let provider = StaticProvider::from_env().expect("Fail to load Credentials key");
        let minio = Minio::builder()
            .host(env::var("MINIO_HOST").unwrap())
            .provider(provider)
            .secure(false)
            .builder()
            .unwrap();

        assert!(minio.bucket("bucket-test1").make().await.is_ok());
        assert!(minio.bucket("bucket-test2").make().await.is_ok());
        println!("bucket lists {:?}", minio.list_buckets().await);
        assert!(minio.bucket("bucket-test2").remove().await.is_ok());
        assert!(minio.bucket("bucket-test1").exists().await.unwrap());
        assert!(!minio.bucket("bucket-test2").exists().await.unwrap());
        assert!(minio.bucket("bucket-test1").remove().await.is_ok());

        let args = ListObjectsArgs::default()
            .max_keys(10)
            .start_after("test1004.txt");
        println!("list {:?}", minio.bucket("file").list_object(args).await);

        // // minio.make_bucket("file12").await;
        let mut count = 0u32;

        // println!("{:?}", "ss");

        // Infinite loop
        loop {
            count += 1;

            let mut file = tokio::fs::File::open("test/test.txt").await.unwrap();
            let mut s = minio
                .put_object("file", format!("/test/我的{}/ew.txt", count), file)
                .await;
            let s = minio
                .remove_object("file", format!("/test/ss{}.txt", count), None)
                .await;

            if count >= 2 {
                println!("OK, that's enough");

                // Exit this loop
                break;
            }
        }
        minio.remove_object("file", "/test1.txt", None).await;
        let s = minio
            .remove_object(
                "file",
                "/test2.txt",
                Some("dfbd25b3-abec-4184-a4e8-5a35a5c1174d".to_string()),
            )
            .await;
        println!("{:?}", s);
        // minio
        //     .presigned_get_object("file", "test/我的1/ew.txt")
        //     .await;

        assert!(minio
            .presigned_object("file", "/test/12/ew.txt")
            .put()
            .await
            .is_ok());
        println!(
            "== {:?}",
            minio
                .executor(Method::GET)
                .bucket_name("file")
                .query("tagging", "")
                .query("notification", "")
                .send()
                .await
                .unwrap()
                .text()
                .await
        );
    }
}
