use std::borrow::Borrow;
use std::collections::HashMap;
use std::hash::Hash;
use std::path::Path;

use futures::StreamExt;
use hyper::header::{self, HeaderName};
use hyper::{HeaderMap, Method};
use reqwest::Response;
use tokio::fs::File;
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWriteExt};

use crate::client::Minio;
use crate::errors::Result;
use crate::errors::{S3Error, XmlError};
use crate::sse::{Sse, SseCustomerKey};
use crate::types::response::{InitiateMultipartUploadResult, Tagging};
use crate::types::QueryMap;
use crate::utils::{md5sum_hash, urlencode};

#[derive(Debug, Clone, Default)]
pub struct MetaData {
    inner: HashMap<String, String>,
}

impl MetaData {
    /// key will be converted to lowercase.
    pub fn insert(&mut self, k: String, v: String) -> Option<String> {
        self.inner.insert(k.to_lowercase(), v)
    }

    pub fn gen_header(&self) -> HeaderMap {
        let mut headers = HeaderMap::new();
        for (k, v) in &self.inner {
            let key = format!("x-amz-meta-{}", k);
            let key = HeaderName::from_bytes(key.as_bytes());
            headers.insert(key.unwrap(), v.parse().unwrap());
        }
        headers
    }

    pub fn get<Q: ?Sized>(&self, k: &Q) -> Option<&String>
    where
        String: Borrow<Q>,
        Q: Hash + Eq,
    {
        self.inner.get(k)
    }
}

#[derive(Debug, Clone)]
pub struct Object {
    bucket_name: String,
    object_name: String,
    last_modified: String,
    etag: String,
    content_type: String,
    version_id: String,
    size: usize,
}

impl Object {
    pub fn bucket_name(&self) -> &str {
        self.bucket_name.as_ref()
    }

    pub fn object_name(&self) -> &str {
        self.object_name.as_ref()
    }

    pub fn last_modified(&self) -> &str {
        self.last_modified.as_ref()
    }

    pub fn etag(&self) -> &str {
        self.etag.as_ref()
    }

    pub fn content_type(&self) -> &str {
        self.content_type.as_ref()
    }

    pub fn version_id(&self) -> &str {
        self.version_id.as_ref()
    }

    pub fn size(&self) -> usize {
        self.size
    }
}

pub struct CopySource {
    bucket_name: String,
    object_name: String,
    region: Option<String>,
    version_id: Option<String>,
    ssec: Option<HeaderMap>,
    match_etag: Option<String>,
    not_match_etag: Option<String>,
    modified_since: Option<String>,
    unmodified_since: Option<String>,
}

impl From<Object> for CopySource {
    fn from(obj: Object) -> Self {
        let etag = obj.etag().to_string();
        let mut cs = Self::new(obj.bucket_name, obj.object_name);
        cs.match_etag = Some(etag);
        cs
    }
}

impl CopySource {
    pub fn new<T1: Into<String>, T2: Into<String>>(bucket_name: T1, object_name: T2) -> Self {
        Self {
            bucket_name: bucket_name.into(),
            object_name: object_name.into(),
            region: None,
            version_id: None,
            ssec: None,
            match_etag: None,
            not_match_etag: None,
            modified_since: None,
            unmodified_since: None,
        }
    }

    pub fn version_id<T: Into<String>>(mut self, version_id: T) -> Self {
        self.version_id = Some(version_id.into());
        self
    }

    pub fn ssec(mut self, ssec: &SseCustomerKey) -> Self {
        self.ssec = Some(ssec.copy_headers());
        self
    }

    pub fn gen_copy_header(&self) -> HeaderMap {
        let mut header = HeaderMap::new();
        let mut copy_source =
            urlencode(&format!("/{}/{}", self.bucket_name, self.object_name), true);
        if let Some(version_id) = &self.version_id {
            copy_source = copy_source + "?versionId=" + version_id;
        }
        header.insert("x-amz-copy-source", copy_source.parse().unwrap());
        if let Some(value) = &self.match_etag {
            header.insert("x-amz-copy-source-if-match", value.parse().unwrap());
        }
        if let Some(value) = &self.not_match_etag {
            header.insert("x-amz-copy-source-if-none-match", value.parse().unwrap());
        }
        if let Some(value) = &self.modified_since {
            header.insert(
                "x-amz-copy-source-if-modified-since",
                value.parse().unwrap(),
            );
        }
        if let Some(value) = &self.unmodified_since {
            header.insert(
                "x-amz-copy-source-if-unmodified-since",
                value.parse().unwrap(),
            );
        }
        if let Some(ssec) = &self.ssec {
            for (k, v) in ssec {
                header.insert(k, v.to_owned());
            }
        }

        header
    }
}

#[derive(Clone)]
pub struct ObjectExecutor<'a> {
    // object
    offset: usize,
    length: usize,
    version_id: Option<String>,
    content_type: Option<String>,
    ssec_headers: HeaderMap,
    object_name: String,
    // bucket
    bucket_name: String,
    region: String,
    expected_bucket_owner: Option<String>,
    // base
    body: Option<Vec<u8>>,
    headers: HeaderMap,
    querys: QueryMap,
    client: &'a Minio,
}

impl<'a> ObjectExecutor<'a> {
    pub fn new<T1: Into<String>, T2: Into<String>>(
        client: &'a Minio,
        bucket_name: T1,
        object_name: T2,
    ) -> Self {
        return Self {
            // object,
            offset: 0,
            length: 0,
            version_id: None,
            content_type: None,
            ssec_headers: HeaderMap::new(),
            object_name: object_name.into(),
            // bucket
            bucket_name: bucket_name.into(),
            region: client.region().to_string(),
            expected_bucket_owner: None,
            // base
            body: None,
            headers: HeaderMap::new(),
            querys: QueryMap::new(),
            client,
        };
    }

    pub fn offset(mut self, offset: usize) -> Self {
        self.offset = offset;
        self
    }

    pub fn length(mut self, length: usize) -> Self {
        self.length = length;
        self
    }

    pub fn version_id<S: Into<String>>(mut self, version_id: S) -> Self {
        self.version_id = Some(version_id.into());
        self
    }

    pub fn content_type<S: Into<String>>(mut self, content_type: S) -> Self {
        self.content_type = Some(content_type.into());
        self
    }

    pub fn expected_bucket_owner<S: Into<String>>(mut self, expected_bucket_owner: S) -> Self {
        self.expected_bucket_owner = Some(expected_bucket_owner.into());
        self
    }

    pub fn request_headers(mut self, request_headers: HeaderMap) -> Self {
        self.headers = request_headers;
        self
    }

    pub fn extra_query_params(mut self, extra_query_params: QueryMap) -> Self {
        self.querys = extra_query_params;
        self
    }

    pub fn ssec<T>(mut self, ssec: &T) -> Self
    where
        T: Sse,
    {
        self.ssec_headers = ssec.headers();
        self
    }

    #[inline]
    async fn _send(mut self, method: Method) -> Result<Response> {
        if let Some(version_id) = &self.version_id {
            self.querys.insert("versionId", version_id);
        };
        if let Some(owner) = self.expected_bucket_owner.clone() {
            self.headers
                .insert("x-amz-expected-bucket-owner", owner.parse()?);
        }
        let query = self.querys.into();
        self.client
            ._execute(
                method,
                &self.region,
                Some(self.bucket_name),
                Some(self.object_name),
                self.body,
                Some(self.headers),
                Some(query),
            )
            .await
    }

    async fn _send_text(self, method: Method) -> Result<String> {
        let res = self._send(method).await?;
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

impl<'a> ObjectExecutor<'a> {
    /**
    Get data of an object. Returned [Result]<[Response]>
    - offset: Start byte position of object data.
    - length: Number of bytes of object data from offset.
    - ssec: Server-side encryption customer key.
    - version_id: Version-ID of the object.
    - extra_query_params: Extra query parameters for advanced usage.
    - request_headers: Any additional headers to be added with GET request.
    - content_type: Sets the Content-Type header of the response.
    # Exapmle
    ``` rust
    # use minio_rsc::Minio;
    # async fn example(minio: Minio){
    let response = minio.object("bucket", "file.txt")
        .offset(3)
        .length(10)
        .version_id("cdabf31a-9752-4265-b137-6b3961fbaf9b")
        .get()
        .await;
    # }
    ```
    */
    pub async fn get(mut self) -> Result<Response> {
        self.ssec_headers.iter().for_each(|(key, val)| {
            self.headers.insert(key, val.clone());
        });
        if let Some(content_type) = &self.content_type {
            self.querys.insert("response-content-type", content_type);
        };
        if self.offset > 0 || self.length > 0 {
            let ranger = if self.length > 0 {
                format!("bytes={}-{}", self.offset, self.offset + self.length - 1)
            } else {
                format!("bytes={}-", self.offset)
            };
            if let Ok(value) = ranger.parse() {
                self.headers.insert(header::RANGE, value);
            }
        }
        return self._send(Method::GET).await;
    }

    /**
    Downloads data of an object to file.
    # Exapmle
    ``` rust
    # use minio_rsc::Minio;
    # async fn example(minio: Minio){
    let response = minio.object("bucket", "file.txt")
        .version_id("cdabf31a-9752-4265-b137-6b3961fbaf9b")
        .write_to("file.txt")
        .await;
    # }
    ```
    */
    pub async fn write_to<P>(self, path: P) -> Result<()>
    where
        P: AsRef<Path>,
    {
        let res = self.get().await?;
        if !res.status().is_success() {
            let text = res.text().await?;
            let s3err: S3Error = text.as_str().try_into()?;
            Err(s3err)?
        } else {
            let mut stream = res.bytes_stream();
            let mut file = File::create(path).await?;
            while let Some(item) = stream.next().await {
                if let Ok(datas) = item {
                    file.write_all(&datas).await?;
                }
            }
            Ok(())
        }
    }

    /// Get object information.
    ///
    /// return Ok(None) if object not found
    pub async fn stat(mut self) -> Result<Option<Object>> {
        self.ssec_headers.iter().for_each(|(key, val)| {
            self.headers.insert(key, val.clone());
        });
        let bucket_name = self.bucket_name.clone();
        let object_name = self.object_name.clone();
        let res = self._send(Method::HEAD).await?;
        if !res.status().is_success() {
            return Ok(None);
        }
        let res_header = res.headers();
        let etag = res_header
            .get(header::ETAG)
            .map(|x| x.to_str().unwrap_or(""))
            .unwrap_or("")
            .replace("\"", "");
        let size: usize = res_header
            .get(header::CONTENT_LENGTH)
            .map(|x| x.to_str().unwrap_or("0").parse().unwrap_or(0))
            .unwrap_or(0);
        let last_modified = res_header
            .get(header::LAST_MODIFIED)
            .map(|x| x.to_str().unwrap_or(""))
            .unwrap_or("")
            .to_owned();
        let content_type = res_header
            .get(header::CONTENT_TYPE)
            .map(|x| x.to_str().unwrap_or(""))
            .unwrap_or("")
            .to_owned();
        let version_id = res_header
            .get("x-amz-version-id")
            .map(|x| x.to_str().unwrap_or(""))
            .unwrap_or("")
            .to_owned();
        Ok(Some(Object {
            bucket_name,
            object_name,
            last_modified,
            etag,
            content_type,
            version_id,
            size,
        }))
    }

    pub async fn copy_from(mut self, copy_source: CopySource) -> Result<()> {
        self.ssec_headers.iter().for_each(|(key, val)| {
            self.headers.insert(key, val.clone());
        });
        copy_source.gen_copy_header().iter().for_each(|(key, val)| {
            self.headers.insert(key, val.clone());
        });
        self._send(Method::PUT).await?;
        Ok(())
    }

    pub async fn put<D: AsyncRead + std::marker::Unpin>(mut self, mut data: D) -> Result<()> {
        let mut buf = Vec::new();
        data.read_to_end(&mut buf).await.unwrap();
        self.body = Some(buf);
        if let Some(content_type) = &self.content_type {
            self.querys.insert("content-type", content_type);
        };
        self._send_text(Method::PUT).await?;
        Ok(())
    }

    /**
    Remove an object.
    # Exapmle
    ``` rust
    # use minio_rsc::Minio;
    # async fn example(minio: Minio){
    let response = minio.object("bucket", "file.txt")
        .version_id("cdabf31a-9752-4265-b137-6b3961fbaf9b")
        .remove()
        .await;
    # }
    ```
    */
    pub async fn remove(self) -> Result<bool> {
        self._send_text(Method::DELETE).await?;
        Ok(true)
    }
}

impl<'a> ObjectExecutor<'a> {
    // async fn _upload_part(
    //     self,
    //     body: Vec<u8>,
    //     upload_id: String,
    //     part_number: usize,
    // ) -> Result<()> {
    //     self.inner
    //         .clone()
    //         .method(Method::PUT)
    //         .query("partNumber", format!("{}", part_number))
    //         .query("uploadId", upload_id)
    //         .body(body)
    //         .send()
    //         .await?;
    //     Ok(())
    // }

    pub async fn _create_multipart_upload(self) -> Result<InitiateMultipartUploadResult> {
        let mut sw = self.clone();
        sw.querys = QueryMap::from_str("uploads");
        sw._send_text(Method::POST)
            .await?
            .as_str()
            .try_into()
            .map_err(|e: XmlError| e.into())
    }
}

impl<'a> ObjectExecutor<'a> {
    pub async fn get_tags(mut self) -> Result<Tagging> {
        self.querys = QueryMap::from_str("tagging");
        let text = self._send_text(Method::GET).await?;
        text.as_str().try_into().map_err(|e: XmlError| e.into())
    }

    pub async fn del_tags(mut self) -> Result<bool> {
        self.querys = QueryMap::from_str("tagging");
        self._send_text(Method::DELETE).await?;
        Ok(true)
    }

    pub async fn set_tags(mut self, tagging: Tagging) -> Result<()> {
        let body = tagging.to_xml()?;
        let md5 = md5sum_hash(body.as_ref());
        self.querys = QueryMap::from_str("tagging");
        if let Ok(value) = md5.parse() {
            self.headers.insert("Content-MD5", value);
        }
        self.body = Some(body);
        self._send_text(Method::PUT).await?;
        Ok(())
    }
}

mod tests {
    use super::MetaData;
    use crate::client::{self, Minio};
    use crate::errors::Result;
    use crate::executor::object_executor::CopySource;
    use crate::executor::Executor;
    use crate::provider::StaticProvider;
    use crate::types::args::ListObjectsArgs;
    use crate::types::response::Tagging;
    use hyper::Method;
    use std::env;
    use tokio;

    #[tokio::main]
    #[test]
    async fn test_object_executor() -> Result<()> {
        dotenv::dotenv().ok();
        let provider = StaticProvider::from_env().expect("Fail to load Credentials key");
        let minio = Minio::builder()
            .host(env::var("MINIO_HOST").unwrap())
            .provider(provider)
            .secure(false)
            .build()?;
        let test_object = minio.object("file", "test.txt");
        if test_object.clone().stat().await.is_ok() {
            test_object.clone().remove().await?;
        }
        test_object
            .clone()
            .content_type("text/plain")
            .put("hello minio".as_bytes())
            .await?;

        let str = test_object
            .clone()
            .offset(6)
            .length(2)
            .get()
            .await?
            .text()
            .await?;
        assert_eq!(str, "mi".to_string());

        let s = minio
            .object("file", "fs111.txt")
            // .content_type("plan/txt")
            .put("111".as_bytes())
            .await;
        println!("{:?}", s);
        let s = minio
            .object("file", "fs1.txt")
            .version_id("16cf7c94-8e1a-4c88-a4e2-3301ce531aa8")
            .copy_from(
                CopySource::new("file", "fs.txt")
                    .version_id("cdabf31a-9752-4265-b137-6b3961fbaf9b"),
            )
            .await;
        println!("{:?}", s);
        let s = minio
            .object("file", "iuii.txt")
            // .version_id("16cf7c94-8e1a-4c88-a4e2-3301ce531aa8")
            ._create_multipart_upload()
            .await;
        println!("{:?}", s);
        let mut s = MetaData::default();
        println!("{:?}", s.gen_header());
        let s = minio
            .executor(Method::GET)
            .bucket_name("file")
            .query("uploads", "")
            .query("delimiter", "/")
            .send()
            .await
            .unwrap()
            .text()
            .await;
        println!("{:?}", s);
        Ok(())
    }
}
