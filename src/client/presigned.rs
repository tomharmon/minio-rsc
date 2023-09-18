use std::str::FromStr;

use hyper::HeaderMap;
use hyper::{Method, Uri};

use super::{PresignedArgs, QueryMap};
use crate::error::{Result, ValueError};
use crate::signer::presign_v4;
use crate::time::UtcTime;
use crate::utils::urlencode_binary;
use crate::Minio;

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
    async fn _get_presigned_url<T1: Into<String>, T2: Into<String>>(
        &self,
        method: Method,
        bucket_name: T1,
        object_name: T2,
        expires: usize,
        response_headers: Option<HeaderMap>,
        request_date: Option<UtcTime>,
        version_id: Option<String>,
        extra_query_params: Option<QueryMap>,
    ) -> Result<String> {
        if expires < 1 || expires > 604800 {
            return Err(ValueError::from("expires must be between 1 second to 7 days").into());
        }
        let date: UtcTime = request_date.unwrap_or(UtcTime::default());
        let mut query = extra_query_params.unwrap_or(QueryMap::new());
        if let Some(id) = version_id {
            query.insert("versionId".to_string(), id);
        }
        let credentials = self.fetch_credentials().await;
        if let Some(token) = credentials.session_token() {
            query.insert("X-Amz-Security-Token".to_string(), token.to_string());
        }
        if let Some(headers) = response_headers {
            for (name, value) in &headers {
                query.insert(name.to_string(), urlencode_binary(value.as_bytes(), false));
            }
        }
        let uri = self._build_uri(Some(bucket_name.into()), Some(object_name.into()));
        let uri = uri + "?" + &query.to_query_string();
        let uri = Uri::from_str(&uri).map_err(|e| ValueError::new(e.to_string()))?;
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

    /// Get presigned URL of an object to download its data with expiry time.
    /// ## Example
    /// ``` rust
    /// # use minio_rsc::Minio;
    /// # use minio_rsc::client::PresignedArgs;
    /// # async fn example(minio: Minio){
    /// let presigned_get_object: String = minio
    ///     .presigned_get_object(
    ///         PresignedArgs::new("bucket", "file.txt")
    ///             .expires(24*3600)
    ///             .version_id("version_id"),
    ///     )
    ///     .await
    ///     .unwrap();
    /// # }
    /// ```
    pub async fn presigned_get_object(&self, args: PresignedArgs) -> Result<String> {
        self._get_presigned_url(
            Method::GET,
            args.bucket_name,
            args.object_name,
            args.expires,
            args.headers,
            args.request_date,
            args.version_id,
            Some(args.querys),
        )
        .await
    }

    /// Get presigned URL of an object to upload data with expiry time.
    /// ## Example
    /// ``` rust
    /// # use minio_rsc::Minio;
    /// # use minio_rsc::client::PresignedArgs;
    /// # async fn example(minio: Minio){
    /// let presigned_put_object: String = minio
    ///     .presigned_put_object(
    ///         PresignedArgs::new("bucket", "file.txt")
    ///             .expires(24*3600)
    ///             .version_id("version_id"),
    ///     )
    ///     .await
    ///     .unwrap();
    /// # }
    /// ```
    pub async fn presigned_put_object(&self, args: PresignedArgs) -> Result<String> {
        self._get_presigned_url(
            Method::PUT,
            args.bucket_name,
            args.object_name,
            args.expires,
            args.headers,
            args.request_date,
            args.version_id,
            Some(args.querys),
        )
        .await
    }
}
