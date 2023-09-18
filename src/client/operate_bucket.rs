use bytes::Bytes;
use hyper::Method;
use hyper::{header, HeaderMap};

use super::{BucketArgs, ListObjectsArgs};
use crate::error::{Error, Result, XmlError};
use crate::datatype::{
    Bucket, ListAllMyBucketsResult, ListBucketResult, ObjectLockConfiguration, Owner, Tags,
    VersioningConfiguration,
};
use crate::utils::md5sum_hash;
use crate::Minio;

/// Operating the bucket
impl Minio {
    #[inline]
    pub(crate) fn _bucket_executor(
        &self,
        bucket: BucketArgs,
        method: Method,
    ) -> super::BaseExecutor {
        self.executor(method)
            .bucket_name(&bucket.name)
            .headers_merge2(bucket.extra_headers)
            .apply(|e| {
                if let Some(owner) = &bucket.expected_bucket_owner {
                    e.header("x-amz-expected-bucket-owner", owner)
                } else {
                    e
                }
            })
    }

    /**
    Check if a bucket exists.
    If bucket exists and you have permission to access it, return [Ok(true)], otherwise [Ok(false)]
    ## Example
    ```rust
    use minio_rsc::client::BucketArgs;
    # use minio_rsc::Minio;
    # use minio_rsc::error::Result;
    # async fn example(minio: Minio) -> Result<()>{
    let exists:bool = minio.bucket_exists(BucketArgs::new("bucket")).await?;
    let exists:bool = minio.bucket_exists("bucket").await?;
    # Ok(())
    # }
    ```
     */
    pub async fn bucket_exists<B>(&self, bucket: B) -> Result<bool>
    where
        B: Into<BucketArgs>,
    {
        let bucket: BucketArgs = bucket.into();
        self._bucket_executor(bucket, Method::HEAD)
            .send()
            .await
            .map(|res| res.status().is_success())
    }

    /** List information of all accessible buckets.
    ## Example
    ```rust
    # use minio_rsc::Minio;
    # async fn example(minio: Minio){
    let (buckets, owner) = minio.list_buckets().await.unwrap();
    # }
    ```
     */
    pub async fn list_buckets(&self) -> Result<(Vec<Bucket>, Owner)> {
        let text = self.executor(Method::GET).send_text_ok().await?;
        let res: Result<ListAllMyBucketsResult> =
            text.as_str().try_into().map_err(|e: XmlError| e.into());
        let res = res?;
        return Ok(res.into_part());
    }

    /**
    Lists object information of a bucket.
    ## Example
    ```rust
    use minio_rsc::client::ListObjectsArgs;
    # use minio_rsc::Minio;
    # async fn example(minio: Minio){
    let args = ListObjectsArgs::default().max_keys(10);
    minio.list_objects("bucket", args).await;
    # }
    ```
     */
    pub async fn list_objects<B>(
        &self,
        bucket: B,
        args: ListObjectsArgs,
    ) -> Result<ListBucketResult>
    where
        B: Into<BucketArgs>,
    {
        let bucket: BucketArgs = bucket.into();
        let text = self
            ._bucket_executor(bucket, Method::GET)
            .querys(args.args_query_map())
            .headers_merge2(args.extra_headers)
            .send_text_ok()
            .await?;
        text.as_str().try_into().map_err(|e: XmlError| e.into())
    }

    /**
    Create a bucket with object_lock
    ## params
    - object_lock: prevents objects from being deleted.
    Required to support retention and legal hold.
    Can only be enabled at bucket creation.
    ## Example
    ```rust
    use minio_rsc::client::BucketArgs;
    # use minio_rsc::Minio;
    # async fn example(minio: Minio){
    minio.make_bucket(BucketArgs::new("bucket"), true).await;
    minio.make_bucket("bucket", false).await;
    # }
    ```
     */
    pub async fn make_bucket<B>(&self, bucket: B, object_lock: bool) -> Result<String>
    where
        B: Into<BucketArgs>,
    {
        let bucket: BucketArgs = bucket.into();
        let region = &bucket.region.unwrap_or(self.region().to_string());
        let body = format!("<CreateBucketConfiguration><LocationConstraint>{}</LocationConstraint></CreateBucketConfiguration>",region);
        self.executor(Method::PUT)
            .bucket_name(bucket.name)
            .headers_merge2(bucket.extra_headers)
            .apply(|e| {
                if object_lock {
                    e.header("x-amz-bucket-object-lock-enabled", "true")
                } else {
                    e
                }
            })
            .body(body)
            .send_ok()
            .await
            .map(|res| {
                let location = res.headers().get(header::LOCATION);
                if let Some(loc) = location {
                    if let Ok(loc) = loc.to_str() {
                        return Ok(loc.to_string());
                    }
                }
                Err(res.into())
            })?
    }

    /**
    Remove an **empty** bucket.
    If the operation succeeds, return [Ok] otherwise [Error]
    ## Example
    ```rust
    use minio_rsc::client::BucketArgs;
    # use minio_rsc::Minio;
    # async fn example(minio: Minio){
    minio.remove_bucket(BucketArgs::new("bucket")).await;
    minio.remove_bucket("bucket").await;
    # }
    ```
     */
    pub async fn remove_bucket<B>(&self, bucket: B) -> Result<()>
    where
        B: Into<BucketArgs>,
    {
        let bucket: BucketArgs = bucket.into();
        self._bucket_executor(bucket, Method::DELETE)
            .send_ok()
            .await
            .map(|_| ())
    }

    /**
    Get Option<[Tags]> of a bucket.
    Note: return [None] if bucket had not set tagging or delete tagging.
    ## Example
    ```rust
    use minio_rsc::client::BucketArgs;
    use minio_rsc::types::Tags;
    # use minio_rsc::{Minio, error::Result};
    # async fn example(minio: Minio) -> Result<()> {
    let tags:Option<Tags> = minio.get_bucket_tags(BucketArgs::new("bucket")).await?;
    let tags:Option<Tags> = minio.get_bucket_tags("bucket").await?;
    # Ok(())}
    ```
     */
    pub async fn get_bucket_tags<B>(&self, bucket: B) -> Result<Option<Tags>>
    where
        B: Into<BucketArgs>,
    {
        let bucket: BucketArgs = bucket.into();
        let res = self
            ._bucket_executor(bucket, Method::GET)
            .query("tagging", "")
            .send_text_ok()
            .await;
        match res {
            Ok(text) => text
                .as_str()
                .try_into()
                .map(Some)
                .map_err(|e: XmlError| e.into()),
            Err(Error::S3Error(s)) if s.code == "NoSuchTagSet" => Ok(None),
            Err(err) => Err(err),
        }
    }

    /**
    Set tags of a bucket.
    ## Example
    ```rust
    use minio_rsc::client::BucketArgs;
    use minio_rsc::types::Tags;
    # use minio_rsc::{Minio, error::Result};
    # async fn example(minio: Minio) -> Result<()> {
    let mut tags = Tags::new();
    tags.insert("key1".to_string(), "value1".to_string());
    tags.insert("key2".to_string(), "value2".to_string());
    tags.insert("key3".to_string(), "value3".to_string());
    minio.set_bucket_tags(BucketArgs::new("bucket"), tags).await?;
        let mut tags:Tags = minio.get_bucket_tags(BucketArgs::new("bucket")).await?.unwrap_or(Tags::new());
    tags.insert("key4".to_string(), "value4".to_string());
    minio.set_bucket_tags("bucket", tags).await?;
    # Ok(())}
    ```
     */
    pub async fn set_bucket_tags<B>(&self, bucket: B, tags: Tags) -> Result<()>
    where
        B: Into<BucketArgs>,
    {
        let bucket: BucketArgs = bucket.into();
        let body = Bytes::from(tags.to_xml());
        let md5 = md5sum_hash(&body);
        let mut headers = HeaderMap::new();
        headers.insert("Content-MD5", md5.parse()?);
        self._bucket_executor(bucket, Method::PUT)
            .query("tagging", "")
            .body(body)
            .send_ok()
            .await?;
        Ok(())
    }

    /**
    Delete tags of a bucket.
    ## Example
    ```rust
    use minio_rsc::types::args::BucketArgs;
    use minio_rsc::types::Tags;
    # use minio_rsc::{Minio, error::Result};
    # async fn example(minio: Minio) -> Result<()> {
    minio.delete_bucket_tags(BucketArgs::new("bucket")).await?;
    minio.delete_bucket_tags("bucket").await?;
    # Ok(())}
    ```
    */
    pub async fn delete_bucket_tags<B>(&self, bucket: B) -> Result<()>
    where
        B: Into<BucketArgs>,
    {
        let bucket: BucketArgs = bucket.into();
        self._bucket_executor(bucket, Method::DELETE)
            .query("tagging", "")
            .send_ok()
            .await?;
        Ok(())
    }

    /**
    Get versioning configuration of a bucket.
    ## Example
    ```rust
    use minio_rsc::client::BucketArgs;
    # use minio_rsc::{Minio, error::Result};
    # async fn example(minio: Minio) -> Result<()> {
    let versing = minio.get_bucket_versioning("bucket").await?;
    # Ok(())}
    ```
    */
    pub async fn get_bucket_versioning<B>(&self, bucket: B) -> Result<VersioningConfiguration>
    where
        B: Into<BucketArgs>,
    {
        let bucket: BucketArgs = bucket.into();
        self._bucket_executor(bucket, Method::GET)
            .query_string("versioning")
            .send_text_ok()
            .await?
            .as_str()
            .try_into()
            .map_err(|e: XmlError| e.into())
    }

    /**
    Get versioning configuration of a bucket.
    ## Example
    ```rust
    use minio_rsc::client::BucketArgs;
    use minio_rsc::types::VersioningConfiguration;
    # use minio_rsc::{Minio, error::Result};
    # async fn example(minio: Minio) -> Result<()> {
    let versing = VersioningConfiguration::new(true, None);
    minio.set_bucket_versioning("bucket", versing).await?;
    # Ok(())}
    ```
    */
    pub async fn set_bucket_versioning<B>(
        &self,
        bucket: B,
        versioning: VersioningConfiguration,
    ) -> Result<bool>
    where
        B: Into<BucketArgs>,
    {
        let bucket: BucketArgs = bucket.into();
        let body = crate::xml::ser::to_string(&versioning)
            .map(Bytes::from)
            .map_err(XmlError::from)?;
        let md5 = md5sum_hash(&body);
        self._bucket_executor(bucket, Method::PUT)
            .query_string("versioning")
            .header("Content-MD5", &md5)
            .body(body)
            .send_ok()
            .await
            .map(|_| true)
    }

    /**
    Get object-lock configuration in a bucket.
    ## Example
    ```rust
    use minio_rsc::client::BucketArgs;
    # use minio_rsc::{Minio, error::Result};
    # async fn example(minio: Minio) -> Result<()> {
    let config = minio.get_object_lock_config("bucket").await?;
    # Ok(())}
    ```
    */
    pub async fn get_object_lock_config<B>(&self, bucket: B) -> Result<ObjectLockConfiguration>
    where
        B: Into<BucketArgs>,
    {
        let bucket: BucketArgs = bucket.into();
        self._bucket_executor(bucket, Method::GET)
            .query_string("object-lock")
            .send_text_ok()
            .await?
            .as_str()
            .try_into()
            .map_err(|e: XmlError| e.into())
    }

    /**
    Get object-lock configuration in a bucket.
    ## Example
    ```rust
    use minio_rsc::client::BucketArgs;
    use minio_rsc::types::ObjectLockConfiguration;
    # use minio_rsc::{Minio, error::Result};
    # async fn example(minio: Minio) -> Result<()> {
    let mut conf = ObjectLockConfiguration::new(1, true, true);
    minio.set_object_lock_config("bucket", conf).await?;
    # Ok(())}
    ```
    */
    pub async fn set_object_lock_config<B>(
        &self,
        bucket: B,
        config: ObjectLockConfiguration,
    ) -> Result<()>
    where
        B: Into<BucketArgs>,
    {
        let bucket: BucketArgs = bucket.into();
        let body = Bytes::from(config.to_xml());
        let md5 = md5sum_hash(&body);
        self._bucket_executor(bucket, Method::PUT)
            .query_string("object-lock")
            .header("Content-MD5", &md5)
            .body(body)
            .send_ok()
            .await
            .map(|_| ())
    }

    /**
    Delete object-lock configuration in a bucket.
    ## Example
    ```rust
    use minio_rsc::client::BucketArgs;
    # use minio_rsc::{Minio, error::Result};
    # async fn example(minio: Minio) -> Result<()> {
    minio.delete_object_lock_config("bucket").await?;
    # Ok(())}
    ```
    */
    pub async fn delete_object_lock_config<B>(&self, bucket: B) -> Result<()>
    where
        B: Into<BucketArgs>,
    {
        let config = ObjectLockConfiguration::default();
        self.set_object_lock_config(bucket, config).await
    }
}
