use crate::errors::{Error, Result, S3Error, ValueError, XmlError};
use crate::sse::{Sse, SseCustomerKey};
use crate::types::args::{BaseArgs, CopySource, ListMultipartUploadsArgs, MultipartUploadArgs};
use crate::types::response::{
    CompleteMultipartUploadResult, InitiateMultipartUploadResult, ListMultipartUploadsResult,
    ListPartsResult,
};
use crate::types::{CompleteMultipartUpload, Part};
use crate::Minio;
use hyper::{header, HeaderMap, Method};

/// Operating multiUpload
impl Minio {
    /// Aborts a multipart upload.
    pub async fn abort_multipart_upload(
        &self,
        multipart_upload: &MultipartUploadArgs,
    ) -> Result<bool> {
        let res = self
            .executor(Method::DELETE)
            .bucket_name(multipart_upload.bucket())
            .object_name(multipart_upload.key())
            .query("uploadId", multipart_upload.upload_id())
            .apply(|e| {
                if let Some(bucket) = multipart_upload.bucket_owner() {
                    e.header("x-amz-expected-bucket-owner", &bucket)
                } else {
                    e
                }
            })
            .send()
            .await?;
        if res.status() == 204 {
            Ok(true)
        } else {
            let text = res.text().await?;
            let s: S3Error = text.as_str().try_into()?;
            Err(s)?
        }
    }

    /// Completes a multipart upload by assembling previously uploaded parts.
    pub async fn complete_multipart_upload(
        &self,
        multipart_upload: &MultipartUploadArgs,
        parts: CompleteMultipartUpload,
        extra_header: Option<&HeaderMap>,
    ) -> Result<CompleteMultipartUploadResult> {
        self.executor(Method::POST)
            .bucket_name(multipart_upload.bucket())
            .object_name(multipart_upload.key())
            .query("uploadId", multipart_upload.upload_id())
            .apply(|e| {
                if let Some(bucket) = multipart_upload.bucket_owner() {
                    e.header("x-amz-expected-bucket-owner", &bucket)
                } else {
                    e
                }
            })
            .headers_merge2(extra_header)
            .headers_merge2(multipart_upload.ssec_header())
            .body(parts.to_xml().as_bytes().to_vec())
            .send_text_ok()
            .await?
            .as_str()
            .try_into()
            .map_err(|e: XmlError| e.into())
    }

    /// This action initiates a multipart upload and returns an MultipartUploadArgs.
    pub async fn create_multipart_upload<T1: Into<String>, T2: Into<String>>(
        &self,
        bucket_name: T1,
        object_name: T2,
        content_type: Option<&str>,
        ssec: Option<SseCustomerKey>,
        bucket_owner: Option<String>,
    ) -> Result<MultipartUploadArgs> {
        let ssec_header = &ssec.map_or(HeaderMap::new(), |f| f.headers());
        let result: Result<InitiateMultipartUploadResult> = self
            .executor(Method::POST)
            .bucket_name(bucket_name)
            .object_name(object_name)
            .query_string("uploads")
            .apply(|e| {
                if let Some(bucket) = &bucket_owner {
                    e.header("x-amz-expected-bucket-owner", &bucket)
                } else {
                    e
                }
            })
            .header(
                header::CONTENT_TYPE,
                content_type.map_or("binary/octet-stream", |f| f),
            )
            .headers_merge(&ssec_header)
            .send_text_ok()
            .await?
            .as_str()
            .try_into()
            .map_err(|e: XmlError| e.into());
        let mut result: MultipartUploadArgs = result?.into();
        result.set_ssec_header(Some(ssec_header.to_owned()));
        result.set_bucket_owner(bucket_owner);
        Ok(result)
    }

    /// lists in-progress multipart uploads.
    pub async fn list_multipart_uploads(
        &self,
        args: ListMultipartUploadsArgs,
    ) -> Result<ListMultipartUploadsResult> {
        self.executor(Method::GET)
            .bucket_name(args.bucket_name())
            .querys(args.extra_query_map())
            .headers(args.extra_headers())
            .send_text_ok()
            .await?
            .as_str()
            .try_into()
            .map_err(|e: XmlError| e.into())
    }

    /// Lists the parts that have been uploaded for a specific multipart upload.
    pub async fn list_parts(
        &self,
        multipart_upload: &MultipartUploadArgs,
        max_parts: Option<usize>,
        part_number_marker: Option<usize>,
    ) -> Result<ListPartsResult> {
        self.executor(Method::GET)
            .bucket_name(multipart_upload.bucket())
            .object_name(multipart_upload.key())
            .query("uploadId", multipart_upload.upload_id())
            .query("max-parts", max_parts.unwrap_or(1000).to_string())
            .apply(|e| {
                let e = if let Some(n) = part_number_marker {
                    e.query("part-number-marker", n.to_string())
                } else {
                    e
                };
                if let Some(bucket) = multipart_upload.bucket_owner() {
                    e.header("x-amz-expected-bucket-owner", &bucket)
                } else {
                    e
                }
            })
            .headers_merge2(multipart_upload.ssec_header())
            .send_text_ok()
            .await?
            .as_str()
            .try_into()
            .map_err(|e: XmlError| e.into())
    }

    /// Uploads a part in a multipart upload.
    pub async fn upload_part(
        &self,
        multipart_upload: &MultipartUploadArgs,
        part_number: usize,
        body: Vec<u8>,
    ) -> Result<Part> {
        if part_number < 1 || part_number > 10000 {
            return Err(ValueError::from(
                "part_number is a positive integer between 1 and 10,000.",
            ))?;
        }
        let res = self
            .executor(Method::PUT)
            .bucket_name(multipart_upload.bucket())
            .object_name(multipart_upload.key())
            .query("uploadId", multipart_upload.upload_id())
            .query("partNumber", part_number.to_string())
            .apply(|e| {
                if let Some(bucket) = multipart_upload.bucket_owner() {
                    e.header("x-amz-expected-bucket-owner", &bucket)
                } else {
                    e
                }
            })
            .headers_merge2(multipart_upload.ssec_header())
            .body(body)
            .send()
            .await?;
        if res.status() == 200 {
            if let Some(s) = res
                .headers()
                .get(header::ETAG)
                .map(|x| x.to_str().unwrap_or(""))
            {
                Ok(Part::new(s.to_string(), part_number))
            } else {
                Err(Error::HttpError)
            }
        } else {
            let text = res.text().await?;
            let s: S3Error = text.as_str().try_into()?;
            Err(s)?
        }
    }

    /// Uploads a part by copying data from an existing object as data source.
    pub async fn upload_part_copy(
        &self,
        multipart_upload: &MultipartUploadArgs,
        part_number: usize,
        copy_source: CopySource,
    ) -> Result<String> {
        if part_number < 1 || part_number > 10000 {
            return Err(ValueError::from(
                "part_number is a positive integer between 1 and 10,000.",
            ))?;
        }
        let res = self
            .executor(Method::PUT)
            .bucket_name(multipart_upload.bucket())
            .object_name(multipart_upload.key())
            .query("uploadId", multipart_upload.upload_id())
            .query("partNumber", part_number.to_string())
            .apply(|e| {
                if let Some(bucket) = multipart_upload.bucket_owner() {
                    e.header("x-amz-expected-bucket-owner", &bucket)
                } else {
                    e
                }
            })
            .headers_merge2(multipart_upload.ssec_header())
            .headers_merge(&copy_source.extra_headers())
            .send()
            .await?;
        if res.status() == 200 {
            if let Some(s) = res
                .headers()
                .get(header::ETAG)
                .map(|x| x.to_str().unwrap_or(""))
            {
                Ok(s.to_string())
            } else {
                Err(Error::HttpError)
            }
        } else {
            let text = res.text().await?;
            let s: S3Error = text.as_str().try_into()?;
            Err(s)?
        }
    }
}

mod tests {
    use crate::client::Minio;
    use crate::errors::Result;
    use crate::provider::StaticProvider;
    use crate::types::args::ListMultipartUploadsArgs;
    use std::env;
    use tokio;

    #[tokio::main]
    #[test]
    async fn test_multi_upload() -> Result<()> {
        dotenv::dotenv().ok();

        let provider = StaticProvider::from_env().expect("Fail to load Credentials key");
        let minio = Minio::builder()
            .host(env::var("MINIO_HOST").unwrap())
            .provider(provider)
            .secure(false)
            .builder()
            .unwrap();

        let multipart_upload = minio
            .create_multipart_upload("file", "/test/1.txt", Some("text/plain"), None, None)
            .await?;
        let part1 = minio
            .upload_part(
                &multipart_upload,
                1,
                "test_multi_upload".as_bytes().to_vec(),
            )
            .await?;

        minio
            .complete_multipart_upload(&multipart_upload, vec![part1].into(), None)
            .await?;
        minio
            .list_multipart_uploads(ListMultipartUploadsArgs::new("file".to_string()))
            .await?;
        Ok(())
    }
}
