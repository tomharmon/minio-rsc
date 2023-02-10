use crate::errors::{Error, Result, XmlError};

use crate::types::args::{BaseArgs, BucketArgs, ListObjectsArgs};
use crate::types::response::{Buckets, ListAllMyBucketsResult, ListBucketResult};
use crate::types::{Bucket, Owner};
use crate::Minio;
use hyper::header;
use hyper::Method;

/// Operating the bucket
impl Minio {
    /// Check if a bucket exists.
    pub async fn bucket_exists<B: Into<BucketArgs>>(&self, args: B) -> Result<bool> {
        let args: BucketArgs = args.into();
        self.executor(Method::HEAD)
            .bucket_name(args.bucket_name)
            .headers_merge2(args.extra_headers.as_ref())
            .apply(|e| {
                if let Some(owner) = args.expected_bucket_owner {
                    e.header("x-amz-expected-bucket-owner", &owner)
                } else {
                    e
                }
            })
            .send_ok()
            .await?;
        Ok(true)
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
    pub async fn make_bucket<B: Into<BucketArgs>>(&self, args: B) -> Result<String> {
        let args: BucketArgs = args.into();
        let region = &args.region.unwrap_or(self.region().to_string());
        let body = format!("<CreateBucketConfiguration><LocationConstraint>{}</LocationConstraint></CreateBucketConfiguration>",region);
        let res = self
            .executor(Method::PUT)
            .bucket_name(args.bucket_name)
            .headers_merge2(args.extra_headers.as_ref())
            .body(body.as_bytes().to_vec())
            .send_ok()
            .await?;
        let location = res.headers().get(header::LOCATION);
        if let Some(loc) = location {
            if let Ok(loc) = loc.to_str() {
                return Ok(loc.to_string());
            }
        }
        Err(Error::HttpError)
    }

    /// Remove an **empty** bucket.
    pub async fn remove_bucket<B: Into<BucketArgs>>(&self, args: B) -> Result<bool> {
        let args: BucketArgs = args.into();
        self.executor(Method::DELETE)
            .bucket_name(args.bucket_name)
            .headers_merge2(args.extra_headers.as_ref())
            .apply(|e| {
                if let Some(owner) = args.expected_bucket_owner {
                    e.header("x-amz-expected-bucket-owner", &owner)
                } else {
                    e
                }
            })
            .send_ok()
            .await?;
        Ok(true)
    }
}
