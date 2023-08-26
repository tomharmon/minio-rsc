use crate::errors::{Error, Result, S3Error, ValueError, XmlError};
use crate::types::args::{
    BaseArgs, CopySource, ListMultipartUploadsArgs, MultipartUploadArgs, ObjectArgs,
};
use crate::types::response::{
    CompleteMultipartUploadResult, CopyPartResult, InitiateMultipartUploadResult,
    ListMultipartUploadsResult, ListPartsResult,
};
use crate::types::Part;
use crate::Minio;
use bytes::Bytes;
use hyper::{header, HeaderMap, Method};

/// Operating multiUpload
impl Minio {
    /// Aborts a multipart upload.
    pub async fn abort_multipart_upload(
        &self,
        multipart_upload: &MultipartUploadArgs,
    ) -> Result<()> {
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
            Ok(())
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
        parts: Vec<Part>,
        extra_header: Option<&HeaderMap>,
    ) -> Result<CompleteMultipartUploadResult> {
        let mut body = "<CompleteMultipartUpload>".to_string();
        for i in parts {
            body += &i.to_tag();
        }
        body += "</CompleteMultipartUpload>";
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
            .body(body)
            .send_text_ok()
            .await?
            .as_str()
            .try_into()
            .map_err(|e: XmlError| e.into())
    }

    /// This action initiates a multipart upload and returns an MultipartUploadArgs.
    pub async fn create_multipart_upload(&self, args: ObjectArgs) -> Result<MultipartUploadArgs> {
        let result: Result<InitiateMultipartUploadResult> = self
            .executor(Method::POST)
            .bucket_name(args.bucket_name.as_str())
            .object_name(args.object_name.as_str())
            .query_string("uploads")
            .apply(|e| {
                if let Some(bucket) = &args.expected_bucket_owner {
                    e.header("x-amz-expected-bucket-owner", &bucket)
                } else {
                    e
                }
            })
            .header(
                header::CONTENT_TYPE,
                &args
                    .content_type
                    .map_or("binary/octet-stream".to_string(), |f| f),
            )
            .headers_merge2(args.ssec_headers.as_ref())
            .send_text_ok()
            .await?
            .as_str()
            .try_into()
            .map_err(|e: XmlError| e.into());
        let mut result: MultipartUploadArgs = result?.into();
        result.set_ssec_header(args.ssec_headers.to_owned());
        result.set_bucket_owner(args.expected_bucket_owner);
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
        args: &MultipartUploadArgs,
        part_number: usize,
        body: Bytes,
    ) -> Result<Part> {
        if part_number < 1 || part_number > 10000 {
            return Err(ValueError::from(
                "part_number is a positive integer between 1 and 10,000.",
            ))?;
        }
        let res = self
            .executor(Method::PUT)
            .bucket_name(args.bucket())
            .object_name(args.key())
            .query("uploadId", args.upload_id())
            .query("partNumber", part_number.to_string())
            .apply(|e| {
                if let Some(bucket) = args.bucket_owner() {
                    e.header("x-amz-expected-bucket-owner", &bucket)
                } else {
                    e
                }
            })
            .headers_merge2(args.ssec_header())
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
            .headers_merge(&copy_source.extra_headers())
            .send_text_ok()
            .await?;
        let result: CopyPartResult = res.as_str().try_into()?;
        Ok(Part::new(result.e_tag, part_number))
    }
}
