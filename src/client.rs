use std::io::Cursor;
use std::str::FromStr;
use std::sync::Arc;

use crate::errors::{Result, S3Error, ValueError, XmlError};
use crate::executor::{BaseExecutor, GetObjectExecutor};
use crate::provider::{Provider, StaticProvider};
use crate::signer::{presign_v4, sha256_hash, sign_v4_authorization};
use crate::time::aws_format_time;
use crate::types::args::BaseArgs;
use crate::types::args::ListObjectsArgs;
use crate::types::response::{ListAllMyBucketsResult, ListBucketResult};
use crate::types::Region;
use crate::utils::{check_bucket_name, is_urlencoded, urlencode, EMPTY_CONTENT_SHA256};
use chrono::{DateTime, Utc};
use hyper::client::HttpConnector;
use hyper::{header, header::HeaderValue, HeaderMap};
use hyper::{Body, Method, Uri};
use quick_xml::events::BytesText;
use quick_xml::Writer;
use regex::Regex;
use reqwest::Response;
use tokio::io::{AsyncRead, AsyncReadExt};
use tokio::sync::Mutex;
type Client = hyper::client::Client<HttpConnector, Body>;

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
            let host = if host.starts_with("https://") || host.starts_with("http://") {
                host
            } else {
                format!("http{}://{}", if self.secure { "s" } else { "" }, host)
            };

            let agent: HeaderValue = self
                .agent
                .parse()
                .map_err(|_| ValueError::from("invalid agent"))?;

            let client2 = if let Some(client) = self.client {
                client
            } else {
                let mut headers = header::HeaderMap::new();
                let i = if host.starts_with("https://") { 8 } else { 7 };
                let host = host[i..]
                    .parse()
                    .map_err(|_| ValueError::from("invalid host"))?;
                headers.insert(header::HOST, host);
                headers.insert(header::USER_AGENT, agent.clone());
                reqwest::Client::builder()
                    .default_headers(headers)
                    .build()
                    .unwrap()
            };
            Ok(Minio {
                inner: Arc::new(MinioRef {
                    host,
                    client: Client::new(),
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
    pub host: String,
    pub client: Client,
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
            (Some(b), Some(mut o)) => {
                if !is_urlencoded(&o) {
                    o = urlencode(&o).replace("%2F", "/");
                }
                format!("{}/{}/{}", self.inner.host, b, o)
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
    /// Create a bucket
    pub async fn make_bucket<T1: Into<String>>(&self, bucket_name: T1) -> Result<bool> {
        let mut writer = Writer::new(Cursor::new(Vec::new()));
        writer
            .create_element("CreateBucketConfiguration")
            .write_inner_content(|writer| {
                writer
                    .create_element("LocationConstraint")
                    .write_text_content(BytesText::new("us-east-1"))
                    .unwrap();
                Ok(())
            })
            .unwrap();
        let res = self
            .executor(Method::PUT)
            .bucket_name(bucket_name)
            .body(writer.into_inner().into_inner())
            .send()
            .await?;
        if res.status().is_success() {
            Ok(true)
        } else {
            let text = res.text().await.unwrap();
            let s: S3Error = text.as_str().try_into()?;
            Err(s)?
        }
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

    /// Check if a bucket exists.
    pub async fn bucket_exists<T1: Into<String>>(&self, bucket_name: T1) -> Result<bool> {
        let res = self
            .executor(Method::HEAD)
            .bucket_name(bucket_name)
            .send()
            .await?;
        Ok(res.status().is_success())
    }

    /// Remove an **empty** bucket.
    pub async fn remove_bucket<T1: Into<String>>(&self, bucket_name: T1) -> Result<bool> {
        let res = self
            .executor(Method::DELETE)
            .bucket_name(bucket_name)
            .send()
            .await?;
        response_is_ok(res).await?;
        Ok(true)
    }

    /// Lists object information of a bucket.
    ///
    /// # Example
    /// ```
    /// use minio_rsc::types::args::ListObjectsArgs;
    /// # use minio_rsc::Minio;
    ///
    /// # async fn example(minio: Minio){
    /// let args = ListObjectsArgs::default()
    ///     .max_keys(10)
    ///     .start_after("key1.txt");
    /// minio.list_objects("bucket", args).await;
    /// # }
    /// ```
    pub async fn list_objects(
        &self,
        bucket_name: &str,
        list_objects_args: ListObjectsArgs,
    ) -> Result<ListBucketResult> {
        let res = self
            .executor(Method::GET)
            .bucket_name(bucket_name)
            .querys(list_objects_args.extra_query_map())
            .send()
            .await?;
        let text = response_ok_text(res).await?;
        println!("text {}", &text);

        text.as_str().try_into().map_err(|e: XmlError| e.into())
    }

    pub async fn bucket_location<T1: Into<String>>(&self, bucket_name: T1) -> Result<Region> {
        let res = self
            .executor(Method::GET)
            .bucket_name(bucket_name)
            .query("location", "")
            // .query_params("location=")
            .send()
            .await?;
        let text = response_ok_text(res).await?;
        text.as_str().try_into().map_err(|x: XmlError| x.into())
    }
}

async fn response_is_ok(res: Response) -> Result<Response> {
    if res.status().is_success() {
        Ok(res)
    } else {
        let text = res.text().await.unwrap();
        let s: S3Error = text.as_str().try_into()?;
        Err(s)?
    }
}

async fn response_ok_text(res: Response) -> Result<String> {
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
    pub async fn presigned_get_object<T1: Into<String>, T2: Into<String>>(
        &self,
        bucket_name: T1,
        object_name: T2,
    ) {
        let uri = self._build_uri(Some(bucket_name.into()), Some(object_name.into()));
        let date: DateTime<Utc> = Utc::now();
        let credentials = self.inner.provider.lock().await.fetct().await;

        let r = presign_v4(
            &Method::GET,
            &Uri::from_str(&uri).unwrap(),
            self.region(),
            credentials.access_key(),
            credentials.secret_key(),
            &date,
            604800,
        );
        println!("==== {:?}", r)
    }
}
#[cfg(test)]
mod tests {
    use std::env;

    use crate::client::Minio;
    use crate::executor::Executor;
    use crate::provider::StaticProvider;
    use crate::types::{args::ListObjectsArgs, Region};
    use crate::Credentials;
    use tokio;
    use url::Url;

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

        let s = minio
            .get_object("file", "/test/ss1001.txt")
            .length(2)
            .offset(5)
            .write_to("test/ss.txt")
            .await
            .unwrap();
        // .text()
        // .await;
        // println!("{:?}", s);
        // let s = minio.bucket_location("file").await;

        // println!("bucket lists {:?}", s);
        // println!("bucket lists {:?}", minio.list_buckets().await);
        // // minio.bucket_exists("bucket_name").await;
        // assert!(minio.bucket_exists("file").await.unwrap());
        // assert!(!minio.bucket_exists("no-file").await.unwrap());
        // assert_eq!(
        //     minio.bucket_location("file").await.unwrap(),
        //     Region::from("us-east-1")
        // );

        let ss: Vec<(String, Option<String>)> = vec![
            ("key".to_string(), Some("2".to_string())),
            ("key2".to_string(), None),
            ("key2".to_string(), Some("".to_string())),
        ];

        for (k, v) in ss {
            println!("ss{}={:?}", k, v);
        }
        // println!("ss{}",ss);

        assert!(minio.make_bucket("bucket-test1").await.is_ok());
        assert!(minio.make_bucket("bucket-test2").await.is_ok());
        println!("bucket lists {:?}", minio.list_buckets().await);
        assert!(minio.remove_bucket("bucket-test2").await.is_ok());
        assert!(minio.bucket_exists("bucket-test1").await.unwrap());
        assert!(!minio.bucket_exists("bucket-test2").await.unwrap());
        assert!(minio.remove_bucket("bucket-test1").await.is_ok());

        let args = ListObjectsArgs::default()
            .max_keys(10)
            .start_after("test1004.txt");
        println!("{:?}", minio.list_objects("file", args).await);

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
        minio.presigned_get_object("file", "test/我的1/ew.txt").await;
    }
}
