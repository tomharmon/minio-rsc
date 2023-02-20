use crate::errors::{Error, Result, XmlError};

use crate::types::args::{BaseArgs, BucketArgs, ListObjectsArgs};
use crate::types::response::{Buckets, ListAllMyBucketsResult, ListBucketResult, Tags};
use crate::types::{Bucket, ObjectLockConfiguration, Owner, QueryMap, VersioningConfiguration};
use crate::utils::md5sum_hash;
use crate::Minio;
use hyper::Method;
use hyper::{header, HeaderMap};

/// Operating the bucket
impl Minio {
    #[inline]
    fn _bucket_executor(&self, args: BucketArgs, method: Method) -> crate::executor::BaseExecutor {
        self.executor(method)
            .bucket_name(&args.bucket_name)
            .headers_merge2(args.extra_headers.as_ref())
            .apply(|e| {
                if let Some(owner) = &args.expected_bucket_owner {
                    e.header("x-amz-expected-bucket-owner", owner)
                } else {
                    e
                }
            })
    }

    /// Check if a bucket exists.
    pub async fn bucket_exists<B: Into<BucketArgs>>(&self, args: B) -> Result<bool> {
        let args: BucketArgs = args.into();
        self._bucket_executor(args, Method::HEAD)
            .send()
            .await
            .map(|res| res.status().is_success())
    }

    /// List information of all accessible buckets.
    ///
    pub async fn list_buckets(&self) -> Result<(Vec<Bucket>, Owner)> {
        let text = self.executor(Method::GET).send_text_ok().await?;
        let res: Result<ListAllMyBucketsResult> =
            text.as_str().try_into().map_err(|e: XmlError| e.into());
        let res = res?;
        let ListAllMyBucketsResult { buckets, owner } = res;
        let Buckets { bucket } = buckets;
        return Ok((bucket, owner));
    }

    /// Lists object information of a bucket.
    ///
    /// # Example
    /// ```
    /// use minio_rsc::types::args::ListObjectsArgs;
    /// # use minio_rsc::Minio;
    ///
    /// # async fn example(minio: Minio){
    /// let args = ListObjectsArgs::new("bucket")
    ///     .max_keys(10);
    /// minio.list_objects("args").await;
    /// # }
    /// ```
    pub async fn list_objects<L: Into<ListObjectsArgs>>(
        &self,
        args: L,
    ) -> Result<ListBucketResult> {
        let args: ListObjectsArgs = args.into();
        let text = self
            .executor(Method::GET)
            .bucket_name(&args.bucket_name)
            .querys(args.extra_query_map())
            .headers(args.extra_headers())
            .send_text_ok()
            .await?;
        text.as_str().try_into().map_err(|e: XmlError| e.into())
    }

    /// Create a bucket
    pub async fn make_bucket<B: Into<BucketArgs>>(
        &self,
        args: B,
        object_lock: bool,
    ) -> Result<String> {
        let args: BucketArgs = args.into();
        let region = &args.region.unwrap_or(self.region().to_string());
        let body = format!("<CreateBucketConfiguration><LocationConstraint>{}</LocationConstraint></CreateBucketConfiguration>",region);
        self.executor(Method::PUT)
            .bucket_name(args.bucket_name)
            .headers_merge2(args.extra_headers.as_ref())
            .apply(|e| {
                if object_lock {
                    e.header("x-amz-bucket-object-lock-enabled", "true")
                } else {
                    e
                }
            })
            .body(body.as_bytes().to_vec())
            .send_ok()
            .await
            .map(|res| {
                let location = res.headers().get(header::LOCATION);
                if let Some(loc) = location {
                    if let Ok(loc) = loc.to_str() {
                        return Ok(loc.to_string());
                    }
                }
                Err(Error::HttpError)
            })?
    }

    /// Remove an **empty** bucket.
    pub async fn remove_bucket<B: Into<BucketArgs>>(&self, args: B) -> Result<bool> {
        let args: BucketArgs = args.into();
        self._bucket_executor(args, Method::DELETE)
            .send_ok()
            .await
            .map(|_| true)
    }

    /// Get tags of a bucket.
    ///
    /// return None if bucket had not set tagging
    pub async fn get_bucket_tags<B: Into<BucketArgs>>(&self, args: B) -> Result<Option<Tags>> {
        let args: BucketArgs = args.into();
        let res = self
            ._bucket_executor(args, Method::GET)
            .querys(QueryMap::from_str("tagging"))
            .send_text_ok()
            .await;
        match res {
            Ok(text) => text
                .as_str()
                .try_into()
                .map(|x| Some(x))
                .map_err(|e: XmlError| e.into()),
            Err(Error::S3Error(s)) => {
                if s.code == "NoSuchTagSet" {
                    return Ok(None);
                } else {
                    Err(Error::S3Error(s))
                }
            }
            Err(err) => Err(err),
        }
    }

    /// Set tags of a bucket.
    pub async fn set_bucket_tags<B: Into<BucketArgs>, T: Into<Tags>>(
        &self,
        args: B,
        tags: T,
    ) -> Result<bool> {
        let args: BucketArgs = args.into();
        let tags: Tags = tags.into();
        let body = tags.to_xml();
        let body = body.as_bytes();
        let md5 = md5sum_hash(body);
        let mut headers = HeaderMap::new();
        headers.insert("Content-MD5", md5.parse()?);
        self._bucket_executor(args, Method::PUT)
            .querys(QueryMap::from_str("tagging"))
            .body(body.to_vec())
            .send_ok()
            .await?;
        Ok(true)
    }

    /// Delete tags of a bucket.
    pub async fn delete_bucket_tags<B: Into<BucketArgs>>(&self, args: B) -> Result<bool> {
        let args: BucketArgs = args.into();
        self._bucket_executor(args, Method::DELETE)
            .querys(QueryMap::from_str("tagging"))
            .send_ok()
            .await?;
        Ok(true)
    }

    /// Get versioning configuration of a bucket.
    pub async fn get_bucket_versioning<B: Into<BucketArgs>>(
        &self,
        args: B,
    ) -> Result<VersioningConfiguration> {
        let args: BucketArgs = args.into();
        self._bucket_executor(args, Method::GET)
            .query_string("versioning")
            .send_text_ok()
            .await?
            .as_str()
            .try_into()
            .map_err(|e: XmlError| e.into())
    }

    /// Get versioning configuration of a bucket.
    pub async fn set_bucket_versioning<B: Into<BucketArgs>>(
        &self,
        args: B,
        versioning: VersioningConfiguration,
    ) -> Result<bool> {
        let args: BucketArgs = args.into();
        let body = versioning.to_xml();
        let body = body.as_bytes();
        let md5 = md5sum_hash(body);
        self._bucket_executor(args, Method::PUT)
            .query_string("versioning")
            .header("Content-MD5", &md5)
            .body(body.to_vec())
            .send_ok()
            .await
            .map(|_| true)
    }

    /// Get object-lock configuration in a bucket.
    pub async fn get_object_lock_config<B: Into<BucketArgs>>(
        &self,
        args: B,
    ) -> Result<ObjectLockConfiguration> {
        let args: BucketArgs = args.into();
        self._bucket_executor(args, Method::GET)
            .query_string("object-lock")
            .send_text_ok()
            .await?
            .as_str()
            .try_into()
            .map_err(|e: XmlError| e.into())
    }

    /// Get object-lock configuration in a bucket.
    pub async fn set_object_lock_config<B: Into<BucketArgs>>(
        &self,
        args: B,
        config: ObjectLockConfiguration,
    ) -> Result<bool> {
        let args: BucketArgs = args.into();
        let body = config.to_xml();
        let body = body.as_bytes();
        let md5 = md5sum_hash(body);
        self._bucket_executor(args, Method::PUT)
            .query_string("object-lock")
            .header("Content-MD5", &md5)
            .body(body.to_vec())
            .send_ok()
            .await
            .map(|_| true)
    }

    /// Delete object-lock configuration in a bucket.
    pub async fn delete_object_lock_config<B: Into<BucketArgs>>(&self, args: B) -> Result<bool> {
        let config = ObjectLockConfiguration::new();
        self.set_object_lock_config(args, config).await
    }
}
