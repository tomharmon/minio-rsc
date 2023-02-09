use hyper::{HeaderMap, Method};
use reqwest::Response;

use crate::client::Minio;
use crate::errors::{S3Error, XmlError};
use crate::types::args::{BaseArgs, ListObjectsArgs};
use crate::types::response::{ListBucketResult, Tagging};
use crate::types::Region;
use crate::utils::md5sum_hash;
use crate::{errors::Result, types::QueryMap};

use super::ObjectExecutor;

#[derive(Clone)]
pub struct BucketExecutor<'a> {
    // bucket
    bucket_name: String,
    region: String,
    expected_bucket_owner: Option<String>,
    // base
    body: Option<Vec<u8>>,
    headers: Option<HeaderMap>,
    querys: QueryMap,
    client: &'a Minio,
}

impl<'a> BucketExecutor<'a> {
    pub fn new<T: Into<String>>(client: &'a Minio, bucket_name: T) -> Self {
        return Self {
            region: client.region().to_string(),
            bucket_name: bucket_name.into(),
            body: None,
            headers: None,
            client,
            querys: QueryMap::new(),
            expected_bucket_owner: None,
        };
    }

    pub fn region<T: Into<String>>(mut self, region: T) -> Self {
        self.region = region.into();
        self
    }

    pub fn expected_bucket_owner(mut self, expected_bucket_owner: Option<String>) -> Self {
        self.expected_bucket_owner = expected_bucket_owner;
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

    pub fn querys(mut self, querys: QueryMap) -> Self {
        self.querys = querys;
        self
    }

    async fn _send(self, method: Method) -> Result<Response> {
        let header = if let Some(owner) = self.expected_bucket_owner.clone() {
            let mut header = self.headers.unwrap_or(HeaderMap::new());
            header.insert("x-amz-expected-bucket-owner", owner.parse()?);
            Some(header)
        } else {
            self.headers
        };
        let query = self.querys.into();
        self.client
            ._execute(
                method,
                &self.region,
                Some(self.bucket_name),
                None,
                self.body,
                header,
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

impl<'a> BucketExecutor<'a> {
    /// Create a bucket
    pub async fn make(self) -> Result<bool> {
        self._send_text(Method::PUT).await?;
        Ok(true)
    }

    /// Check if a bucket exists.
    pub async fn exists(self) -> Result<bool> {
        let res = self._send(Method::HEAD).await?;
        Ok(res.status().is_success())
    }

    /// Remove an **empty** bucket.
    pub async fn remove(self) -> Result<bool> {
        self._send_text(Method::DELETE).await?;
        Ok(true)
    }

    pub async fn location(self) -> Result<Region> {
        let text = self
            .querys(QueryMap::from_str("location"))
            ._send_text(Method::GET)
            .await?;
        text.as_str().try_into().map_err(|x: XmlError| x.into())
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
    pub async fn list_object(self, list_objects_args: ListObjectsArgs) -> Result<ListBucketResult> {
        let text = self
            .querys(list_objects_args.extra_query_map())
            .headers(list_objects_args.extra_headers())
            ._send_text(Method::GET)
            .await?;
        text.as_str().try_into().map_err(|e: XmlError| e.into())
    }

    pub fn object<T: Into<String>>(self, object_name: T) -> ObjectExecutor<'a> {
        ObjectExecutor::new(self.client, self.bucket_name, object_name.into())
    }
    
}

impl<'a> BucketExecutor<'a> {
    pub async fn tags_get(self) -> Result<Tagging> {
        let text = self
            .querys(QueryMap::from_str("tagging"))
            ._send_text(Method::GET)
            .await?;
        text.as_str().try_into().map_err(|e: XmlError| e.into())
    }

    pub async fn tags_delete(self) -> Result<bool> {
        let res = self
            .querys(QueryMap::from_str("tagging"))
            ._send(Method::DELETE)
            .await?;
        Ok(res.status().is_success())
    }

    pub async fn tags_set(self, tagging: Tagging) -> Result<()> {
        let body = tagging.clone().to_xml()?;
        let md5 = md5sum_hash(body.as_ref());
        let mut headers = HeaderMap::new();
        headers.insert("Content-MD5", md5.parse()?);
        self.querys(QueryMap::from_str("tagging"))
            .headers(headers)
            .body(body)
            ._send_text(Method::PUT)
            .await?;
        Ok(())
    }
}

// impl<'a> BucketExecutor<'a> {
//     pub async fn version_get(self) -> Result<String> {
//         let text = self
//             .querys(QueryMap::from_str("versioning"))
//             ._send_text(Method::GET)
//             .await?;
//         Ok(text)
//     }

//     pub async fn version_set(self, tagging: Tagging) -> Result<()> {
//         let body = tagging.to_xml()?;
//         let md5 = md5sum_hash(body.as_ref());
//         let mut headers = HeaderMap::new();
//         headers.insert("Content-MD5", md5.parse()?);
//         self.querys(QueryMap::from_str("versioning"))
//             .headers(headers)
//             .body(body)
//             ._send_text(Method::PUT)
//             .await?;
//         Ok(())
//     }
// }

mod tests {
    use crate::client::Minio;
    use crate::provider::StaticProvider;
    use crate::types::args::ListObjectsArgs;
    use crate::types::response::Tagging;
    use std::env;
    use tokio;

    #[tokio::main]
    #[test]
    async fn test_bucket() {
        dotenv::dotenv().ok();
        let provider = StaticProvider::from_env().expect("Fail to load Credentials key");
        let minio = Minio::builder()
            .host(env::var("MINIO_HOST").unwrap())
            .provider(provider)
            .secure(false)
            .build()
            .unwrap();

        assert!(minio.bucket("bucket-test1").make().await.is_ok());
        assert!(minio.bucket("bucket-test2").make().await.is_ok());
        println!("bucket lists {:?}", minio.list_buckets().await);
        assert!(minio.bucket("bucket-test2").remove().await.is_ok());
        assert!(minio.bucket("bucket-test1").exists().await.unwrap());
        assert!(!minio.bucket("bucket-test2").exists().await.unwrap());

        // test tags
        let bucket1 = minio.bucket("bucket-test1");
        let mut tagging = Tagging::new();
        tagging
            .insert("tag", "value")
            .insert("tag2", "value2")
            .insert("tag3", "value3");
        bucket1.clone().tags_set(tagging).await.unwrap();
        bucket1.clone().tags_get().await.unwrap();
        bucket1.clone().tags_delete().await.unwrap();
        
        bucket1.remove().await.unwrap();
    }
}
