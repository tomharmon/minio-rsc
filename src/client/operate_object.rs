use std::collections::HashMap;
use std::ops::Add;
use std::path::Path;
use std::pin::Pin;

use bytes::{Bytes, BytesMut};
use futures::{Stream, StreamExt};
use hyper::{header, HeaderMap, Method};
use reqwest::Response;

use super::{BucketArgs, CopySource, KeyArgs, ObjectStat, SelectObjectReader, Tags};
use crate::datatype::{AccessControlPolicy, LegalHold, Retention};
use crate::datatype::{LegalHoldStatus, SelectRequest};
use crate::error::{Error, Result, S3Error, ValueError};
use crate::signer::{MAX_MULTIPART_OBJECT_SIZE, MIN_PART_SIZE};
use crate::Minio;

/// Operating the object
impl Minio {
    #[inline]
    fn _object_executor(
        &self,
        method: Method,
        bucket: BucketArgs,
        key: KeyArgs,
        with_sscs: bool,
        with_content_type: bool,
    ) -> Result<super::BaseExecutor> {
        let is_put = method == Method::PUT;
        let metadata_header = if is_put {
            key.get_metadata_header()?
        } else {
            HeaderMap::new()
        };
        let executor = self
            ._bucket_executor(bucket, method)
            .object_name(key.name)
            .headers_merge2(key.extra_headers)
            .apply(|mut e| {
                if let Some(version_id) = key.version_id {
                    e = e.query("versionId", version_id)
                }
                if is_put {
                    e = e.headers_merge(metadata_header);
                }
                if with_content_type {
                    if let Some(content_type) = key.content_type {
                        if is_put {
                            e = e.header(header::CONTENT_TYPE, content_type);
                        } else {
                            e = e.header("response-content-type", content_type);
                        }
                    }
                };
                if with_sscs {
                    e = e.headers_merge2(key.ssec_headers);
                }
                e
            });
        Ok(executor)
    }

    /// Creates a copy of an object that is already stored in Minio.
    /// ## Exapmle
    /// ``` rust
    /// # use minio_rsc::Minio;
    /// use minio_rsc::error::Result;
    /// use minio_rsc::client::{CopySource, KeyArgs};
    ///
    /// # async fn example(minio: Minio)->Result<()>{
    /// let src = CopySource::new("bucket","key1");
    /// let response = minio.copy_object("bucket", "det", src).await?;
    /// // modify content-type
    /// let dst = KeyArgs::new("key2").content_type(Some("image/jpeg".to_string()));
    /// let src = CopySource::new("bucket","key1").metadata_replace(true);
    /// let response = minio.copy_object("bucket", dst, src).await?;
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    pub async fn copy_object<B, K>(&self, bucket: B, key: K, src: CopySource) -> Result<()>
    where
        B: Into<BucketArgs>,
        K: Into<KeyArgs>,
    {
        self._object_executor(Method::PUT, bucket.into(), key.into(), true, true)?
            .headers_merge(src.args_headers())
            .send_ok()
            .await
            .map(|_| ())
    }

    /// Downloads data of an object to file.
    /// # Exapmle
    /// ``` rust
    /// # use minio_rsc::Minio;
    /// # use minio_rsc::error::Result;
    /// # async fn example(minio: Minio)->Result<()>{
    /// let response = minio.fget_object("bucket", "file.txt", "local_file.txt").await?;
    /// # Ok(())
    /// # }
    /// ```
    #[cfg(feature = "fs-tokio")]
    pub async fn fget_object<B, K, P>(&self, bucket: B, key: K, path: P) -> Result<()>
    where
        B: Into<BucketArgs>,
        K: Into<KeyArgs>,
        P: AsRef<Path>,
    {
        use tokio::{fs::File, io::AsyncWriteExt};

        let res = self.get_object(bucket, key).await?;
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

    /// Get [reqwest::Response] of an object.
    /// ## Exapmle
    /// ``` rust
    /// use reqwest::Response;
    /// # use minio_rsc::Minio;
    /// # use minio_rsc::client::KeyArgs;
    /// # use minio_rsc::error::Result;
    /// # async fn example(minio: Minio)->Result<()>{
    /// let response: Response = minio.get_object("bucket", "file.txt").await?;
    /// let key = KeyArgs::new("file.txt").version_id(Some("cdabf31a-9752-4265-b137-6b3961fbaf9b".to_string()));
    /// let response: Response = minio.get_object("bucket", key).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn get_object<B, K>(&self, bucket: B, key: K) -> Result<Response>
    where
        B: Into<BucketArgs>,
        K: Into<KeyArgs>,
    {
        let bucket: BucketArgs = bucket.into();
        let key: KeyArgs = key.into();
        let range = key.range();
        self._object_executor(Method::GET, bucket, key, true, true)?
            .apply(|e| {
                if let Some(range) = range {
                    e.header(header::RANGE, &range)
                } else {
                    e
                }
            })
            .send_ok()
            .await
    }

    /// Get torrent files from a bucket.
    pub async fn get_object_torrent<B, K>(&self, bucket: B, key: K) -> Result<Response>
    where
        B: Into<BucketArgs>,
        K: Into<KeyArgs>,
    {
        let bucket: BucketArgs = bucket.into();
        let key: KeyArgs = key.into();
        self._object_executor(Method::GET, bucket, key, true, true)?
            .query("torrent", "")
            .send_ok()
            .await
    }

    /// Uploads data to an object in a bucket.
    /// ## Exapmle
    /// ``` rust
    /// use reqwest::Response;
    /// use std::collections::HashMap;
    /// use minio_rsc::client::KeyArgs;
    /// # use minio_rsc::Minio;
    /// # use minio_rsc::error::Result;
    ///
    /// # async fn example(minio: Minio)->Result<()>{
    /// let data = "hello minio";
    /// minio.put_object("bucket", "file.txt", data.into()).await?;
    ///
    /// let metadata: HashMap<String, String> = [("filename".to_owned(), "file.txt".to_owned())].into();
    /// let key = KeyArgs::new("file.txt")
    ///             .content_type(Some("text/plain".to_string()))
    ///             .metadata(metadata);
    /// minio.put_object("bucket", key, data.into()).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn put_object<B, K>(&self, bucket: B, key: K, data: Bytes) -> Result<()>
    where
        B: Into<BucketArgs>,
        K: Into<KeyArgs>,
    {
        let bucket: BucketArgs = bucket.into();
        let key: KeyArgs = key.into();
        self._object_executor(Method::PUT, bucket, key, true, true)?
            .body(data)
            .send_ok()
            .await?;
        Ok(())
    }

    /// Upload large payload in an efficient manner easily.
    ///
    /// - len: total byte length of stream.
    /// If set None, the data will be transmitted through `multipart_upload`.
    /// otherwise the data will be transmitted in multiple chunks through an HTTP request.
    pub async fn put_object_stream<B, K>(
        &self,
        bucket: B,
        key: K,
        mut stream: Pin<Box<dyn Stream<Item = Result<Bytes>> + Sync + Send>>,
        len: Option<usize>,
    ) -> Result<()>
    where
        B: Into<BucketArgs>,
        K: Into<KeyArgs>,
    {
        let bucket: BucketArgs = bucket.into();
        let key: KeyArgs = key.into();
        if let Some(len) = len {
            if len >= MAX_MULTIPART_OBJECT_SIZE {
                return Err(ValueError::from("max object size is 5TiB").into());
            }
            if self.multi_chunked() || len < MIN_PART_SIZE {
                self._object_executor(Method::PUT, bucket, key, true, true)?
                    .body((stream, len))
                    .send_ok()
                    .await?;
                return Ok(());
            }
        }
        let mpu_args = self.create_multipart_upload(bucket, key).await?;

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

    /// Uploads data from a file to an object in a bucket.
    /// ## Exapmle
    /// ``` rust
    /// # use minio_rsc::Minio;
    /// # use minio_rsc::error::Result;
    /// # async fn example(minio: Minio)->Result<()>{
    /// minio.fput_object("bucket", "file.txt","localfile.txt").await?;
    /// # Ok(())
    /// # }
    /// ```
    #[cfg(feature = "fs-tokio")]
    pub async fn fput_object<B, K, P>(&self, bucket: B, key: K, path: P) -> Result<()>
    where
        B: Into<BucketArgs>,
        K: Into<KeyArgs>,
        P: AsRef<Path>,
    {
        use crate::signer::RECOMMEND_CHUNK_SIZE;
        use async_stream::stream;
        use tokio::io::AsyncReadExt;

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
        self.put_object_stream(bucket, key, stm, Some(len)).await
    }

    /// Remove an object.
    /// ## Exapmle
    /// ``` rust
    /// # use minio_rsc::Minio;
    /// # use minio_rsc::error::Result;
    /// # async fn example(minio: Minio)->Result<()>{
    /// let response = minio.remove_object("bucket", "file.txt").await?;
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    pub async fn remove_object<B, K>(&self, bucket: B, key: K) -> Result<()>
    where
        B: Into<BucketArgs>,
        K: Into<KeyArgs>,
    {
        self._object_executor(Method::DELETE, bucket.into(), key.into(), true, false)?
            .send_ok()
            .await
            .map(|_| ())
    }

    /// Get object information.
    ///
    /// return Ok(Some([ObjectStat])) if object exists and you have READ access to the object, otherwise return Ok([None])
    /// ## Exapmle
    /// ``` rust
    /// # use minio_rsc::Minio;
    /// # use minio_rsc::error::Result;
    /// # async fn example(minio: Minio)->Result<()>{
    /// let object_stat = minio.stat_object("bucket", "file.txt").await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn stat_object<B, K>(&self, bucket: B, key: K) -> Result<Option<ObjectStat>>
    where
        B: Into<BucketArgs>,
        K: Into<KeyArgs>,
    {
        let bucket: BucketArgs = bucket.into();
        let key: KeyArgs = key.into();
        let bucket_name = bucket.name.clone();
        let object_name = key.name.clone();
        let res = self
            ._object_executor(Method::HEAD, bucket, key, true, false)?
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

    /// Get the access control list (ACL) of an object.
    pub async fn get_object_acl<B, K>(&self, bucket: B, key: K) -> Result<AccessControlPolicy>
    where
        B: Into<BucketArgs>,
        K: Into<KeyArgs>,
    {
        let bucket: BucketArgs = bucket.into();
        let key: KeyArgs = key.into();
        self._object_executor(Method::GET, bucket, key, false, false)?
            .query("acl", "")
            .send_xml_ok()
            .await
    }

    /// Returns true if legal hold is enabled on an object.
    pub async fn is_object_legal_hold_enabled<B, K>(&self, bucket: B, key: K) -> Result<bool>
    where
        B: Into<BucketArgs>,
        K: Into<KeyArgs>,
    {
        let bucket: BucketArgs = bucket.into();
        let key: KeyArgs = key.into();
        let result = self
            ._object_executor(Method::GET, bucket, key, false, false)?
            .query("legal-hold", "")
            .send_xml_ok::<LegalHold>()
            .await;
        match result {
            Ok(l) => Ok(l.status == LegalHoldStatus::ON),
            // Ok(Err(err)) => Err(err.into()),
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
    pub async fn enable_object_legal_hold_enabled<B, K>(&self, bucket: B, key: K) -> Result<()>
    where
        B: Into<BucketArgs>,
        K: Into<KeyArgs>,
    {
        let bucket: BucketArgs = bucket.into();
        let key: KeyArgs = key.into();
        let legal_hold: LegalHold = LegalHold {
            status: LegalHoldStatus::ON,
        };
        self._object_executor(Method::PUT, bucket, key, false, false)?
            .query("legal-hold", "")
            .xml(&legal_hold)
            .send_ok()
            .await
            .map(|_| ())
    }

    /// Disables legal hold on an object.
    pub async fn disable_object_legal_hold_enabled<B, K>(&self, bucket: B, key: K) -> Result<()>
    where
        B: Into<BucketArgs>,
        K: Into<KeyArgs>,
    {
        let bucket: BucketArgs = bucket.into();
        let key: KeyArgs = key.into();
        let legal_hold: LegalHold = LegalHold {
            status: LegalHoldStatus::OFF,
        };
        self._object_executor(Method::PUT, bucket, key, false, false)?
            .query("legal-hold", "")
            .xml(&legal_hold)
            .send_ok()
            .await
            .map(|_| ())
    }

    /// Get [Tags] of an object.
    /// ## Example
    /// ```rust
    /// # use minio_rsc::Minio;
    /// # use minio_rsc::error::Result;
    /// use minio_rsc::client::Tags;
    ///
    /// # async fn example(minio: Minio)->Result<()>{
    /// let tags: Tags = minio.get_object_tags("bucket", "file.txt").await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn get_object_tags<B, K>(&self, bucket: B, key: K) -> Result<Tags>
    where
        B: Into<BucketArgs>,
        K: Into<KeyArgs>,
    {
        let bucket: BucketArgs = bucket.into();
        let key: KeyArgs = key.into();
        self._object_executor(Method::GET, bucket, key, false, false)?
            .query("tagging", "")
            .send_xml_ok()
            .await
    }

    /// Set [Tags] of an object.
    /// ## Example
    /// ```rust
    /// # use minio_rsc::Minio;
    /// # use minio_rsc::error::Result;
    /// use minio_rsc::client::Tags;
    ///
    /// # async fn example(minio: Minio)->Result<()>{
    /// let mut tags: Tags = Tags::new();
    /// tags.insert("key1", "value1")
    ///     .insert("key2", "value2");
    /// minio.set_object_tags("bucket", "file.txt", tags).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn set_object_tags<B, K, T>(&self, bucket: B, key: K, tags: T) -> Result<()>
    where
        B: Into<BucketArgs>,
        K: Into<KeyArgs>,
        T: Into<Tags>,
    {
        let bucket: BucketArgs = bucket.into();
        let key: KeyArgs = key.into();
        self._object_executor(Method::PUT, bucket, key, false, false)?
            .query("tagging", "")
            .xml(&tags.into())
            .send_ok()
            .await
            .map(|_| ())
    }

    /// Delete tags of an object.
    /// ## Example
    /// ```rust
    /// # use minio_rsc::Minio;
    /// # use minio_rsc::error::Result;
    /// # async fn example(minio: Minio)->Result<()>{
    /// minio.del_object_tags("bucket", "file.txt").await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn del_object_tags<B, K>(&self, bucket: B, key: K) -> Result<()>
    where
        B: Into<BucketArgs>,
        K: Into<KeyArgs>,
    {
        let bucket: BucketArgs = bucket.into();
        let key: KeyArgs = key.into();
        self._object_executor(Method::DELETE, bucket, key, false, false)?
            .query("tagging", "")
            .send_ok()
            .await
            .map(|_| ())
    }

    /// Get [Retention] of an object.
    pub async fn get_object_retention<B, K>(&self, bucket: B, key: K) -> Result<Retention>
    where
        B: Into<BucketArgs>,
        K: Into<KeyArgs>,
    {
        let bucket: BucketArgs = bucket.into();
        let key: KeyArgs = key.into();
        self._object_executor(Method::GET, bucket, key, false, false)?
            .query("retention", "")
            .send_xml_ok()
            .await
    }

    /// Set [Retention] of an object.
    pub async fn set_object_retention<B, K>(
        &self,
        bucket: B,
        key: K,
        retention: Retention,
    ) -> Result<()>
    where
        B: Into<BucketArgs>,
        K: Into<KeyArgs>,
    {
        let bucket: BucketArgs = bucket.into();
        let key: KeyArgs = key.into();
        self._object_executor(Method::PUT, bucket, key, false, false)?
            .query("retention", "")
            .xml(&retention)
            .send_ok()
            .await
            .map(|_| ())
    }

    /// Filters the contents of an object based on a simple structured query language (SQL) statement.
    /// ## Example
    /// ```rust
    /// # use minio_rsc::Minio;
    /// # use minio_rsc::error::Result;
    /// use minio_rsc::datatype::{SelectRequest,InputSerialization,CsvInput,CompressionType,JsonOutput};
    ///     # async fn example(client:Minio) -> Result<()>{
    /// let input_serialization = InputSerialization::new(CsvInput::default(), CompressionType::NONE);
    /// let output_serialization = JsonOutput::default().into();
    /// let req = SelectRequest::new(
    ///     "Select * from s3object where s3object._1>100".to_owned(),
    ///     input_serialization,
    ///     output_serialization,
    ///     true,
    ///     None,
    ///     None);
    /// let reader = client.select_object_content("bucket", "example.csv", req).await?;
    /// let data = reader.read_all().await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn select_object_content<B, K>(
        &self,
        bucket: B,
        key: K,
        request: SelectRequest,
    ) -> Result<SelectObjectReader>
    where
        B: Into<BucketArgs>,
        K: Into<KeyArgs>,
    {
        let bucket: BucketArgs = bucket.into();
        let key: KeyArgs = key.into();
        self._object_executor(Method::POST, bucket, key, true, false)?
            .query_string("select&select-type=2")
            .xml(&request)
            .send_ok()
            .await
            .map(|res| SelectObjectReader::new(res, request.output_serialization))
    }
}
