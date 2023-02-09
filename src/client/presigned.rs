use std::str::FromStr;

use crate::errors::{Result, ValueError};

use crate::signer::presign_v4;
use crate::types::args::PresignedArgs;
use crate::types::QueryMap;
use crate::utils::urlencode_binary;
use crate::Minio;
use chrono::{DateTime, Utc};
use hyper::HeaderMap;
use hyper::{Method, Uri};

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
        let credentials = self.fetch_credentials().await;
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

    // /**
    // [PresignedExecutor](crate::executor::PresignedExecutor) for presigned URL of an object.
    // # Example
    // ``` rust
    // # use minio_rsc::Minio;
    // # async fn example(minio: Minio){
    // // Get presigned URL of an object to download its data with expiry time.
    // let get_object_url :String = minio.presigned_object("bucket", "file.txt")
    //     .version_id("version_id")
    //     .expires(24*3600)
    //     .get()
    //     .await.unwrap();
    // // Get presigned URL of an object to upload data with expiry time.
    // let upload_object_url :String = minio.presigned_object("bucket", "file.txt")
    //     .version_id("version_id")
    //     .expires(24*3600)
    //     .put()
    //     .await.unwrap();
    // # }
    // ```
    //  */
    // fn presigned_object<T1: Into<String>, T2: Into<String>>(
    //     &self,
    //     bucket_name: T1,
    //     object_name: T2,
    // ) -> PresignedExecutor {
    //     PresignedExecutor::new(&self, bucket_name, object_name)
    // }

    /**
    Get presigned URL of an object to download its data with expiry time.
    ``` rust
    # use minio_rsc::Minio;
    # use minio_rsc::types::args::PresignedArgs;
    # async fn example(minio: Minio){
    let presigned_get_object: String = minio
        .presigned_get_object(
            PresignedArgs::new("bucket", "file.txt")
                .expires(24*3600)
                .version_id("version_id"),
        )
        .await
        .unwrap();
    # }
    ```
     */
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

    /**
    Get presigned URL of an object to upload data with expiry time.
    ``` rust
    # use minio_rsc::Minio;
    # use minio_rsc::types::args::PresignedArgs;
    # async fn example(minio: Minio){
    let presigned_put_object: String = minio
        .presigned_put_object(
            PresignedArgs::new("bucket", "file.txt")
                .expires(24*3600)
                .version_id("version_id"),
        )
        .await
        .unwrap();
    # }
    ```
     */
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

mod tests {
    use crate::client::Minio;
    use crate::errors::Result;
    use crate::provider::StaticProvider;
    use crate::types::args::PresignedArgs;
    use std::env;
    use tokio;

    #[tokio::main]
    #[test]
    async fn test_presigned() -> Result<()> {
        dotenv::dotenv().ok();

        let provider = StaticProvider::from_env().expect("Fail to load Credentials key");
        let minio = Minio::builder()
            .host(env::var("MINIO_HOST").unwrap())
            .provider(provider)
            .secure(false)
            .build()
            .unwrap();

        let url = minio
            .presigned_get_object(PresignedArgs::new("bucket", "file.txt").expires(24 * 3600))
            .await?;
        println!("{}", url);
        let url = minio
            .presigned_put_object(PresignedArgs::new("bucket", "file.txt").expires(24 * 3600))
            .await?;
        println!("{}", url);
        Ok(())
    }
}
