use std::collections::HashMap;
use std::ops::Add;
use std::path::Path;
use std::pin::Pin;

use crate::errors::{Error, Result, S3Error, ValueError, XmlError};
use crate::signer::{MAX_MULTIPART_OBJECT_SIZE, MIN_PART_SIZE};
use crate::types::args::SelectRequest;
use crate::types::args::{BaseArgs, CopySource, ObjectArgs};
use crate::types::response::SelectObjectReader;
use crate::types::{LegalHold, ObjectStat, Retention, Tags};
use crate::utils::md5sum_hash;
use crate::Minio;

use bytes::{Bytes, BytesMut};
use futures::{Stream, StreamExt};
use hyper::{header, HeaderMap, Method};
use reqwest::Response;

/// Operating the object
impl Minio {
    #[inline]
    fn _object_executor(
        &self,
        method: Method,
        args: ObjectArgs,
        with_sscs: bool,
        with_content_type: bool,
    ) -> Result<super::BaseExecutor> {
        let is_put = method == Method::PUT;
        let metadata_header = if is_put {
            args.get_metadata_header()?
        } else {
            HeaderMap::new()
        };
        let executor = self
            .executor(method)
            .bucket_name(&args.bucket_name)
            .object_name(&args.object_name)
            .headers_merge2(args.extra_headers)
            .apply(|mut e| {
                if let Some(owner) = &args.expected_bucket_owner {
                    e = e.header("x-amz-expected-bucket-owner", owner)
                }
                if let Some(version_id) = &args.version_id {
                    e = e.query("versionId", version_id)
                }
                if is_put {
                    e = e.headers_merge(metadata_header);
                }
                if with_content_type {
                    if let Some(content_type) = &args.content_type {
                        if is_put {
                            e = e.header(header::CONTENT_TYPE, content_type);
                        } else {
                            e = e.header("response-content-type", content_type);
                        }
                    }
                };
                if with_sscs {
                    e = e.headers_merge2(args.ssec_headers);
                }
                e
            });
        Ok(executor)
    }

    /**
    Creates a copy of an object that is already stored in Minio.
    # Exapmle
    ``` rust
    # use minio_rsc::Minio;
    use minio_rsc::types::args::ObjectArgs;
    use minio_rsc::errors::Result;
    use minio_rsc::types::args::CopySource;

    # async fn example(minio: Minio)->Result<()>{
    let src = CopySource::new("bucket","key1");
    let dst = ObjectArgs::new("bucket","key2");
    let response = minio.copy_object(dst, src).await?;
    // modify content-type
    let dst = ObjectArgs::new("bucket","key2").content_type(Some("image/jpeg".to_string()));
    let src = CopySource::from(dst.clone()).metadata_replace(true);
    let response = minio.copy_object(dst, src).await?;
    # Ok(())
    # }
    ```
    */
    pub async fn copy_object<B: Into<ObjectArgs>>(&self, dst: B, src: CopySource) -> Result<()> {
        let dst: ObjectArgs = dst.into();
        self._object_executor(Method::PUT, dst, true, true)?
            .headers_merge(src.args_headers())
            .send_ok()
            .await?;
        Ok(())
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
    pub async fn fget_object<B: Into<ObjectArgs>, P>(&self, args: B, path: P) -> Result<()>
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
            Ok(())
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
            ._object_executor(Method::GET, args, true, true)?
            .apply(|e| {
                if let Some(range) = range {
                    e.header(header::RANGE, &range)
                } else {
                    e
                }
            })
            .send_ok()
            .await?)
    }

    pub async fn put_object<B: Into<ObjectArgs>>(&self, args: B, data: Bytes) -> Result<()> {
        let args: ObjectArgs = args.into();
        self._object_executor(Method::PUT, args, true, true)?
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
            if self.multi_chunked() || len < MIN_PART_SIZE {
                let args: ObjectArgs = args.into();
                self._object_executor(Method::PUT, args, true, true)?
                    .body((stream, len))
                    .send_ok()
                    .await?;
                return Ok(());
            }
        }
        let mpu_args = self.create_multipart_upload(args.into()).await?;

        let mut parts = Vec::new();
        let mut current = BytesMut::with_capacity(MIN_PART_SIZE);
        while let Some(piece) = stream.next().await {
            if current.len() >= MIN_PART_SIZE {
                let part = match self
                    .upload_part(&mpu_args, parts.len().add(1), current.freeze())
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
                current = BytesMut::with_capacity(MIN_PART_SIZE);
                parts.push(part);
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
                .upload_part(&mpu_args, parts.len().add(1), current.freeze())
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
        use crate::signer::RECOMMEND_CHUNK_SIZE;
        use async_stream::stream;
        use tokio::io::AsyncReadExt;

        let args: ObjectArgs = args.into();
        let mut file = tokio::fs::File::open(path).await?;
        let meta = file.metadata().await?;
        let len = meta.len() as usize;
        let stm = Box::pin(stream! {
            loop  {
                let mut buf = BytesMut::with_capacity(RECOMMEND_CHUNK_SIZE);
                let size = file.read_buf(&mut buf).await;
                yield match size {
                    Ok(d) if d > 0 => Ok(buf.freeze()),
                    Ok(_) => break,
                    Err(e) => Err(e.into())
                }
            }
        });
        self.put_object_stream(args, stm, Some(len)).await
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
    pub async fn remove_object<B: Into<ObjectArgs>>(&self, args: B) -> Result<()> {
        let args: ObjectArgs = args.into();
        self._object_executor(Method::DELETE, args, true, false)?
            .send_ok()
            .await?;
        Ok(())
    }

    /**
    Get object information.

    return Ok([Some]) if object exists and you have READ access to the object, otherwise return Ok([None])
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
            ._object_executor(Method::HEAD, args, true, false)?
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
        let mut metadata = HashMap::new();
        res_header.into_iter().for_each(|(k, v)| {
            let key = k.as_str();
            if key.starts_with("x-amz-meta-") {
                if let Ok(value) = String::from_utf8(v.as_bytes().to_vec()) {
                    metadata.insert(key[11..].to_string(), value.to_owned());
                }
            }
        });
        Ok(Some(ObjectStat {
            bucket_name,
            object_name,
            last_modified,
            etag,
            content_type,
            version_id,
            size,
            metadata,
        }))
    }

    ///Returns true if legal hold is enabled on an object.
    pub async fn is_object_legal_hold_enabled<B: Into<ObjectArgs>>(&self, args: B) -> Result<bool> {
        let args: ObjectArgs = args.into();
        let result: Result<String> = self
            ._object_executor(Method::GET, args, false, false)?
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
        self._object_executor(Method::PUT, args, false, false)?
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
        self._object_executor(Method::PUT, args, false, false)?
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
        self._object_executor(Method::GET, args, false, false)?
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
        self._object_executor(Method::PUT, args, false, false)?
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
        self._object_executor(Method::DELETE, args, false, false)?
            .query("tagging", "")
            .send_ok()
            .await
            .map(|_| true)
    }

    /// Get retention of a object.
    pub async fn get_object_retention<B: Into<ObjectArgs>>(&self, args: B) -> Result<Retention> {
        let args: ObjectArgs = args.into();
        self._object_executor(Method::GET, args, false, false)?
            .query("retention", "")
            .send_text_ok()
            .await?
            .as_str()
            .try_into()
            .map_err(|e: XmlError| e.into())
    }

    /// Set retention of a object.
    pub async fn set_object_retention<B: Into<ObjectArgs>>(
        &self,
        args: B,
        retention: Retention,
    ) -> Result<()> {
        let args: ObjectArgs = args.into();
        let body = Bytes::from(retention.to_xml());
        let md5 = md5sum_hash(&body);
        self._object_executor(Method::PUT, args, false, false)?
            .query("retention", "")
            .header("Content-MD5", &md5)
            .body(body)
            .send_ok()
            .await
            .map(|_| ())
    }

    /// Filters the contents of an object based on a simple structured query language (SQL) statement.
    /// ## Example
    /// ```rust
    /// # use minio_rsc::Minio;
    /// # use minio_rsc::errors::Result;
    /// use minio_rsc::types::args::{SelectRequest,InputSerialization,CsvInput,CompressionType,JsonOutput};
    ///
    /// # async fn example(client:Minio) -> Result<()>{
    /// let input_serialization = InputSerialization::new(CsvInput::default(), CompressionType::NONE);
    /// let output_serialization = JsonOutput::default().into();
    /// let req = SelectRequest::new(
    ///     "Select * from s3object where s3object._1>100".to_owned(),
    ///     input_serialization,
    ///     output_serialization,
    ///     true,
    ///     None,
    ///     None);
    /// let reader = client.select_object_content(("bucket", "example.csv"), req).await?;
    /// let data = reader.read_all().await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn select_object_content<B: Into<ObjectArgs>>(
        &self,
        args: B,
        request: SelectRequest,
    ) -> Result<SelectObjectReader> {
        let args: ObjectArgs = args.into();
        let body = request.to_xml();
        let res = self
            ._object_executor(Method::POST, args, true, false)?
            .query_string("select&select-type=2")
            .body(body)
            .send_ok()
            .await?;
        Ok(SelectObjectReader::new(res))
    }
}
