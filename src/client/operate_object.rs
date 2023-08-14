use std::ops::Add;
use std::path::Path;
use std::pin::Pin;

use crate::errors::{Error, Result, S3Error, ValueError, XmlError};
use crate::signer::{MAX_MULTIPART_OBJECT_SIZE, MIN_PART_SIZE};
use crate::types::args::{BaseArgs, CopySource, ObjectArgs};
use crate::types::response::Tags;
use crate::types::{LegalHold, ObjectStat, Retention};
use crate::utils::md5sum_hash;
use crate::Minio;

use bytes::{Bytes, BytesMut};
use futures::{Stream, StreamExt};
use hyper::{header, Method};
use reqwest::Response;

/// Operating the object
impl Minio {
    #[inline]
    fn _object_executor(
        &self,
        method: Method,
        args: &ObjectArgs,
        with_sscs: bool,
        with_content_type: bool,
    ) -> crate::executor::BaseExecutor {
        let is_put = method == Method::PUT;
        self.executor(method)
            .bucket_name(&args.bucket_name)
            .object_name(&args.object_name)
            .headers_merge2(args.extra_headers.as_ref())
            .apply(|e| {
                let e = if let Some(owner) = &args.expected_bucket_owner {
                    e.header("x-amz-expected-bucket-owner", &owner)
                } else {
                    e
                };
                let e = if let Some(version_id) = &args.version_id {
                    e.query("versionId", version_id)
                } else {
                    e
                };
                let e = if with_content_type {
                    if let Some(content_type) = &args.content_type {
                        if is_put {
                            e.query("content-type", content_type)
                        } else {
                            e.query("response-content-type", content_type)
                        }
                    } else {
                        e
                    }
                } else {
                    e
                };
                if with_sscs {
                    e.headers_merge2(args.ssec_headers.as_ref())
                } else {
                    e
                }
            })
    }

    pub async fn copy_object<B: Into<ObjectArgs>>(&self, dst: B, src: CopySource) -> Result<bool> {
        let dst: ObjectArgs = dst.into();
        self._object_executor(Method::PUT, &dst, true, false)
            .header(
                header::CONTENT_TYPE,
                dst.content_type
                    .as_ref()
                    .map_or("binary/octet-stream", |f| f),
            )
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
    #[cfg(feature = "fs-tokio")]
    pub async fn fget_object<B: Into<ObjectArgs>, P>(&self, args: B, path: P) -> Result<bool>
    where
        P: AsRef<Path>,
    {
        use tokio::{fs::File, io::AsyncWriteExt};

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

    /**
    Get [reqwest::Response] of an object.
    # Exapmle
    ``` rust
    use reqwest::Response;
    # use minio_rsc::Minio;
    # use minio_rsc::types::args::ObjectArgs;
    # use minio_rsc::errors::Result;
    # async fn example(minio: Minio)->Result<()>{
    let response: Response = minio.get_object(ObjectArgs::new("bucket", "file.txt")).await?;
    let response: Response = minio.get_object(("bucket", "file.txt")).await?;
    let response: Response = minio.get_object(
            ObjectArgs::new("bucket", "file.txt")
                .version_id(Some("cdabf31a-9752-4265-b137-6b3961fbaf9b".to_string()))
            ).await?;
    # Ok(())
    # }
    ```
    */
    pub async fn get_object<B: Into<ObjectArgs>>(&self, args: B) -> Result<Response> {
        let args: ObjectArgs = args.into();
        let range = args.range();
        Ok(self
            ._object_executor(Method::GET, &args, true, true)
            .apply(|e| {
                if let Some(range) = range {
                    e.header(header::RANGE, &range)
                } else {
                    e
                }
            })
            .headers_merge2(args.ssec_headers.as_ref())
            .send_ok()
            .await?)
    }

    pub async fn put_object<B: Into<ObjectArgs>>(&self, args: B, data: Bytes) -> Result<()> {
        let args: ObjectArgs = args.into();
        self._object_executor(Method::PUT, &args, true, true)
            .headers_merge2(args.ssec_headers.as_ref())
            .body(data)
            .send_ok()
            .await?;
        Ok(())
    }

    /**
    Upload large payload in an efficient manner easily.

    - len: total byte length of stream.
    If set None, the data will be transmitted through `multipart_upload`.
    otherwise the data will be transmitted in multiple chunks through an HTTP request.
     */
    pub async fn put_object_stream<B: Into<ObjectArgs>>(
        &self,
        args: B,
        mut stream: Pin<Box<dyn Stream<Item = Result<Bytes>> + Send>>,
        len: Option<usize>,
    ) -> Result<()> {
        if let Some(len) = len {
            if len >= MAX_MULTIPART_OBJECT_SIZE {
                return Err(ValueError::from("max object size is 5TiB").into());
            }
            let args: ObjectArgs = args.into();
            self._object_executor(Method::PUT, &args, true, true)
                .headers_merge2(args.ssec_headers.as_ref())
                .body((stream, len))
                .send_ok()
                .await?;
            return Ok(());
        }
        let mpu_args = self.create_multipart_upload(args.into()).await?;

        let mut parts = Vec::new();
        let mut current = BytesMut::with_capacity(1024 * 1024 * 6);
        while let Some(piece) = stream.next().await {
            if current.len() >= MIN_PART_SIZE {
                let part = match self
                    .upload_part(
                        &mpu_args,
                        parts.len().add(1),
                        Bytes::copy_from_slice(&current),
                    )
                    .await
                {
                    Ok(pce) => pce,
                    Err(e) => {
                        return match self.abort_multipart_upload(&mpu_args).await {
                            Ok(_) => Err(e),
                            Err(err) => Err(err),
                        }
                    }
                };
                parts.push(part);
                current.clear();
            }
            match piece {
                Ok(open_piece) => {
                    current.extend_from_slice(&open_piece);
                }
                Err(e) => {
                    self.abort_multipart_upload(&mpu_args).await?;
                    return Err(e);
                }
            }
        }
        if current.len() != 0 {
            let part = match self
                .upload_part(
                    &mpu_args,
                    parts.len().add(1),
                    Bytes::copy_from_slice(&current),
                )
                .await
            {
                Ok(pce) => pce,
                Err(e) => {
                    return match self.abort_multipart_upload(&mpu_args).await {
                        Ok(_) => Err(e),
                        Err(err) => Err(err),
                    }
                }
            };
            parts.push(part);
            current.clear();
        }

        self.complete_multipart_upload(&mpu_args, parts, None)
            .await
            .map(|_| ())
    }

    /**
    Uploads data from a file to an object in a bucket.
    # Exapmle
    ``` rust
    # use minio_rsc::Minio;
    # use minio_rsc::types::args::ObjectArgs;
    # use minio_rsc::errors::Result;
    # async fn example(minio: Minio)->Result<()>{
    minio.fput_object(ObjectArgs::new("bucket", "file.txt"),"localfile.txt").await?;
    minio.fput_object(("bucket", "file.txt"),"localfile.txt").await?;
    # Ok(())
    # }
    ```
    */
    #[cfg(feature = "fs-tokio")]
    pub async fn fput_object<B: Into<ObjectArgs>, P>(&self, args: B, path: P) -> Result<()>
    where
        P: AsRef<Path>,
    {
        use super::fs_stream::TokioFileStream;

        let args: ObjectArgs = args.into();
        let stream = TokioFileStream::new(path).await?;
        let len = Some(stream.len());
        self.put_object_stream(args, Box::pin(stream), len).await
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
        self._object_executor(Method::DELETE, &args, true, false)
            .send_ok()
            .await?;
        Ok(true)
    }

    /**
    Get object information.

    return [Ok] if object not found
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
            ._object_executor(Method::HEAD, &args, true, false)
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

    ///Returns true if legal hold is enabled on an object.
    pub async fn is_object_legal_hold_enabled<B: Into<ObjectArgs>>(&self, args: B) -> Result<bool> {
        let args: ObjectArgs = args.into();
        let result: Result<String> = self
            ._object_executor(Method::GET, &args, false, false)
            .query("legal-hold", "")
            .send_text_ok()
            .await;
        match result {
            Ok(s) => s
                .as_str()
                .try_into()
                .map_err(|e: XmlError| e.into())
                .map(|res: LegalHold| res.is_enable()),
            Err(Error::S3Error(s)) => {
                if s.code == "NoSuchObjectLockConfiguration" {
                    return Ok(false);
                } else {
                    Err(Error::S3Error(s))
                }
            }
            Err(err) => Err(err),
        }
    }

    /// Enables legal hold on an object.
    pub async fn enable_object_legal_hold_enabled<B: Into<ObjectArgs>>(
        &self,
        args: B,
    ) -> Result<bool> {
        let args: ObjectArgs = args.into();
        let legal_hold: LegalHold = LegalHold::new(true);
        let body = Bytes::from(legal_hold.to_xml());
        let md5 = md5sum_hash(&body);
        self._object_executor(Method::PUT, &args, false, false)
            .query("legal-hold", "")
            .header("Content-MD5", &md5)
            .body(body)
            .send_ok()
            .await
            .map(|_| true)
    }

    /// Disables legal hold on an object.
    pub async fn disable_object_legal_hold_enabled<B: Into<ObjectArgs>>(
        &self,
        args: B,
    ) -> Result<bool> {
        let args: ObjectArgs = args.into();
        let legal_hold: LegalHold = LegalHold::new(false);
        let body = Bytes::from(legal_hold.to_xml());
        let md5 = md5sum_hash(&body);
        self._object_executor(Method::PUT, &args, false, false)
            .query("legal-hold", "")
            .header("Content-MD5", &md5)
            .body(body)
            .send_ok()
            .await
            .map(|_| true)
    }

    /// Get tags of a object.
    pub async fn get_object_tags<B: Into<ObjectArgs>>(&self, args: B) -> Result<Tags> {
        let args: ObjectArgs = args.into();
        self._object_executor(Method::GET, &args, false, false)
            .query("tagging", "")
            .send_text_ok()
            .await?
            .as_str()
            .try_into()
            .map_err(|e: XmlError| e.into())
    }

    /// Set tags of a object.
    pub async fn set_object_tags<B: Into<ObjectArgs>, T: Into<Tags>>(
        &self,
        args: B,
        tags: T,
    ) -> Result<bool> {
        let args: ObjectArgs = args.into();
        let tags: Tags = tags.into();
        let body = Bytes::from(tags.to_xml());
        let md5 = md5sum_hash(&body);
        self._object_executor(Method::PUT, &args, false, false)
            .query("tagging", "")
            .header("Content-MD5", &md5)
            .body(body)
            .send_ok()
            .await
            .map(|_| true)
    }

    /// Delete tags of a object.
    pub async fn delete_object_tags<B: Into<ObjectArgs>>(&self, args: B) -> Result<bool> {
        let args: ObjectArgs = args.into();
        self._object_executor(Method::DELETE, &args, false, false)
            .query("tagging", "")
            .send_ok()
            .await
            .map(|_| true)
    }

    /// Get retention of a object.
    pub async fn get_object_retention<B: Into<ObjectArgs>>(&self, args: B) -> Result<Retention> {
        let args: ObjectArgs = args.into();
        self._object_executor(Method::GET, &args, false, false)
            .query("retention", "")
            .send_text_ok()
            .await?
            .as_str()
            .try_into()
            .map_err(|e: XmlError| e.into())
    }

    /// Set retention of a object.
    pub async fn set_object_retention<B: Into<ObjectArgs>, T: Into<Retention>>(
        &self,
        args: B,
        tags: T,
    ) -> Result<bool> {
        let args: ObjectArgs = args.into();
        let tags: Retention = tags.into();
        let body = Bytes::from(tags.to_xml());
        let md5 = md5sum_hash(&body);
        self._object_executor(Method::PUT, &args, false, false)
            .query("retention", "")
            .header("Content-MD5", &md5)
            .body(body)
            .send_ok()
            .await
            .map(|_| true)
    }
}
