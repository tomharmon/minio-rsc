use std::path::Path;

use crate::errors::S3Error;
use crate::{errors::Result, types::args::CopySource};

use crate::types::args::{BaseArgs, ObjectArgs};
use crate::types::ObjectStat;
use crate::Minio;
use futures::StreamExt;
use hyper::{header, Method};
use reqwest::Response;
use tokio::fs::File;
use tokio::io::AsyncWriteExt;

/// Operating the object
impl Minio {
    pub async fn copy_object<B: Into<ObjectArgs>>(&self, dst: B, src: CopySource) -> Result<bool> {
        let dst: ObjectArgs = dst.into();
        self.executor(Method::PUT)
            .bucket_name(dst.bucket_name)
            .object_name(dst.object_name)
            .apply(|e| {
                let e = if let Some(owner) = dst.expected_bucket_owner {
                    e.header("x-amz-expected-bucket-owner", &owner)
                } else {
                    e
                };
                if let Some(version_id) = dst.version_id {
                    e.query("versionId", version_id)
                } else {
                    e
                }
            })
            .header(
                header::CONTENT_TYPE,
                dst.content_type
                    .as_ref()
                    .map_or("binary/octet-stream", |f| f),
            )
            .headers_merge2(dst.ssec_headers.as_ref())
            .headers_merge(&src.extra_headers())
            .send()
            .await?;
        // Ok(true);
        todo!()
    }

    /**
    Downloads data of an object to file.
    # Exapmle
    ``` rust
    # use minio_rsc::Minio;
    # use minio_rsc::types::args::ObjectArgs;
    # use minio_rsc::errors::Result;
    # async fn example(minio: Minio)->Result<()>{
    let response = minio.fget_object(
            ObjectArgs::new("bucket", "file.txt")
                .version_id(Some("cdabf31a-9752-4265-b137-6b3961fbaf9b".to_string())),
            "file.txt"
        ).await?;
    # Ok(())
    # }
    ```
    */
    pub async fn fget_object<B: Into<ObjectArgs>, P>(&self, args: B, path: P) -> Result<bool>
    where
        P: AsRef<Path>,
    {
        let res = self.get_object(args).await?;
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
            Ok(true)
        }
    }

    pub async fn get_object<B: Into<ObjectArgs>>(&self, args: B) -> Result<Response> {
        let args: ObjectArgs = args.into();
        let range = args.range();
        Ok(self
            .executor(Method::GET)
            .bucket_name(args.bucket_name)
            .object_name(args.object_name)
            .headers_merge2(args.extra_headers.as_ref())
            .apply(|e| {
                let e = if let Some(owner) = args.expected_bucket_owner {
                    e.header("x-amz-expected-bucket-owner", &owner)
                } else {
                    e
                };
                let e = if let Some(version_id) = args.version_id {
                    e.query("versionId", version_id)
                } else {
                    e
                };
                let e = if let Some(content_type) = args.content_type {
                    e.query("response-content-type", content_type)
                } else {
                    e
                };
                if let Some(range) = range {
                    e.header(header::RANGE, &range)
                } else {
                    e
                }
            })
            .headers_merge2(args.ssec_headers.as_ref())
            .send()
            .await?)
    }

    pub async fn put_object<B: Into<ObjectArgs>>(&self, args: B, data: Vec<u8>) -> Result<bool> {
        let args: ObjectArgs = args.into();
        let args: ObjectArgs = args.into();
        let range = args.range();
        self.executor(Method::PUT)
            .bucket_name(args.bucket_name)
            .object_name(args.object_name)
            .headers_merge2(args.extra_headers.as_ref())
            .apply(|e| {
                let e = if let Some(owner) = args.expected_bucket_owner {
                    e.header("x-amz-expected-bucket-owner", &owner)
                } else {
                    e
                };
                let e = if let Some(version_id) = args.version_id {
                    e.query("versionId", version_id)
                } else {
                    e
                };
                let e = if let Some(content_type) = args.content_type {
                    e.query("response-content-type", content_type)
                } else {
                    e
                };
                if let Some(range) = range {
                    e.header(header::RANGE, &range)
                } else {
                    e
                }
            })
            .headers_merge2(args.ssec_headers.as_ref())
            .body(data)
            .send_ok()
            .await?;
        Ok(true)
    }

    /**
    Remove an object.
    # Exapmle
    ``` rust
    # use minio_rsc::Minio;
    # use minio_rsc::types::args::ObjectArgs;
    # use minio_rsc::errors::Result;
    # async fn example(minio: Minio)->Result<()>{
    let response = minio.remove_object(
            ObjectArgs::new("bucket", "file.txt")
                .version_id(Some("cdabf31a-9752-4265-b137-6b3961fbaf9b".to_string())),
        ).await?;
    # Ok(())
    # }
    ```
    */
    pub async fn remove_object<B: Into<ObjectArgs>>(&self, args: B) -> Result<bool> {
        let args: ObjectArgs = args.into();
        self.executor(Method::DELETE)
            .bucket_name(args.bucket_name)
            .object_name(args.object_name)
            .headers_merge2(args.extra_headers.as_ref())
            .apply(|e| {
                let e = if let Some(owner) = args.expected_bucket_owner {
                    e.header("x-amz-expected-bucket-owner", &owner)
                } else {
                    e
                };
                if let Some(version_id) = args.version_id {
                    e.query("versionId", version_id)
                } else {
                    e
                }
            })
            .headers_merge2(args.ssec_headers.as_ref())
            .send()
            .await?;
        Ok(true)
    }

    /**
    Get object information.

    return Ok(None) if object not found
    # Exapmle
    ``` rust
    # use minio_rsc::Minio;
    # use minio_rsc::types::args::ObjectArgs;
    # use minio_rsc::errors::Result;
    # async fn example(minio: Minio)->Result<()>{
    let object_stat = minio.stat_object(
            ObjectArgs::new("bucket", "file.txt")
                .version_id(Some("cdabf31a-9752-4265-b137-6b3961fbaf9b".to_string())),
        ).await?;
    # Ok(())
    # }
    ```
    */
    pub async fn stat_object<B: Into<ObjectArgs>>(&self, args: B) -> Result<Option<ObjectStat>> {
        let args: ObjectArgs = args.into();
        let bucket_name = args.bucket_name.clone();
        let object_name = args.object_name.clone();
        let res = self
            .executor(Method::HEAD)
            .bucket_name(args.bucket_name)
            .object_name(args.object_name)
            .headers_merge2(args.extra_headers.as_ref())
            .apply(|e| {
                let e = if let Some(owner) = args.expected_bucket_owner {
                    e.header("x-amz-expected-bucket-owner", &owner)
                } else {
                    e
                };
                if let Some(version_id) = args.version_id {
                    e.query("versionId", version_id)
                } else {
                    e
                }
            })
            .headers_merge2(args.ssec_headers.as_ref())
            .send()
            .await?;
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
        Ok(Some(ObjectStat {
            bucket_name,
            object_name,
            last_modified,
            etag,
            content_type,
            version_id,
            size,
        }))
    }
}
