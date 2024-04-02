use bytes::Bytes;
use hyper::{header, HeaderMap, Method};

use super::args::MultipartUploadTask;
use super::{BucketArgs, CopySource, KeyArgs, ListMultipartUploadsArgs};
use crate::datatype::Part;
use crate::datatype::{
    CompleteMultipartUpload, CompleteMultipartUploadResult, CopyPartResult,
    InitiateMultipartUploadResult, ListMultipartUploadsResult, ListPartsResult,
};
use crate::error::{Result, S3Error, ValueError};
use crate::signer::{MAX_MULTIPART_COUNT, MAX_PART_SIZE};
use crate::Minio;

/// Operating multiUpload
impl Minio {
    /// Aborts a multipart upload.
    pub async fn abort_multipart_upload(&self, task: &MultipartUploadTask) -> Result<()> {
        let res = self
            .executor(Method::DELETE)
            .bucket_name(task.bucket())
            .object_name(task.key())
            .query("uploadId", task.upload_id())
            .apply(|e| {
                if let Some(bucket) = task.bucket_owner() {
                    e.header("x-amz-expected-bucket-owner", bucket)
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
        task: &MultipartUploadTask,
        parts: Vec<Part>,
        extra_header: Option<HeaderMap>,
    ) -> Result<CompleteMultipartUploadResult> {
        let body = CompleteMultipartUpload { parts };
        self.executor(Method::POST)
            .bucket_name(task.bucket())
            .object_name(task.key())
            .query("uploadId", task.upload_id())
            .apply(|e| {
                if let Some(bucket) = task.bucket_owner() {
                    e.header("x-amz-expected-bucket-owner", bucket)
                } else {
                    e
                }
            })
            .headers_merge2(extra_header)
            .headers_merge2(task.ssec_header().cloned())
            .xml(&body)
            .send_xml_ok()
            .await
    }

    /// This action initiates a multipart upload and returns an MultipartUploadArgs.
    pub async fn create_multipart_upload<B, K>(
        &self,
        bucket: B,
        key: K,
    ) -> Result<MultipartUploadTask>
    where
        B: Into<BucketArgs>,
        K: Into<KeyArgs>,
    {
        let bucket: BucketArgs = bucket.into();
        let key: KeyArgs = key.into();
        let metadata_header: HeaderMap = key.get_metadata_header()?;
        let expected_bucket_owner = bucket.expected_bucket_owner.clone();
        let mut result: MultipartUploadTask = self
            ._bucket_executor(bucket, Method::POST)
            .object_name(key.name.as_str())
            .query_string("uploads")
            .header(
                header::CONTENT_TYPE,
                &key.content_type
                    .map_or("binary/octet-stream".to_string(), |f| f),
            )
            .headers_merge(metadata_header)
            .headers_merge2(key.extra_headers)
            .headers_merge2(key.ssec_headers.clone())
            .send_xml_ok::<InitiateMultipartUploadResult>()
            .await
            .map(Into::into)?;
        result.set_ssec_header(key.ssec_headers);
        result.set_bucket_owner(expected_bucket_owner);
        Ok(result)
    }

    /// lists in-progress multipart uploads.
    pub async fn list_multipart_uploads(
        &self,
        args: ListMultipartUploadsArgs,
    ) -> Result<ListMultipartUploadsResult> {
        self.executor(Method::GET)
            .bucket_name(args.bucket_name())
            .querys(args.args_query_map())
            .headers(args.args_headers())
            .send_xml_ok()
            .await
    }

    /// Lists the parts that have been uploaded for a specific multipart upload.
    pub async fn list_parts(
        &self,
        task: &MultipartUploadTask,
        max_parts: Option<usize>,
        part_number_marker: Option<usize>,
    ) -> Result<ListPartsResult> {
        self.executor(Method::GET)
            .bucket_name(task.bucket())
            .object_name(task.key())
            .query("uploadId", task.upload_id())
            .query("max-parts", max_parts.unwrap_or(1000).to_string())
            .apply(|e| {
                let e = if let Some(n) = part_number_marker {
                    e.query("part-number-marker", n.to_string())
                } else {
                    e
                };
                if let Some(bucket) = task.bucket_owner() {
                    e.header("x-amz-expected-bucket-owner", bucket)
                } else {
                    e
                }
            })
            .headers_merge2(task.ssec_header().cloned())
            .send_xml_ok()
            .await
    }

    /// Uploads a part in a multipart upload.
    pub async fn upload_part(
        &self,
        task: &MultipartUploadTask,
        part_number: usize,
        body: Bytes,
    ) -> Result<Part> {
        if part_number < 1 || part_number > MAX_MULTIPART_COUNT {
            return Err(ValueError::from(
                "part_number is a positive integer between 1 and 10,000.",
            ))?;
        }
        if body.len() > MAX_PART_SIZE {
            return Err(ValueError::from("part size must be less then 5GiB."))?;
        }
        let res = self
            .executor(Method::PUT)
            .bucket_name(task.bucket())
            .object_name(task.key())
            .query("uploadId", task.upload_id())
            .query("partNumber", part_number.to_string())
            .apply(|e| {
                if let Some(bucket) = task.bucket_owner() {
                    e.header("x-amz-expected-bucket-owner", bucket)
                } else {
                    e
                }
            })
            .headers_merge2(task.ssec_header().cloned())
            .body(body)
            .send()
            .await?;
        if res.status() == 200 {
            if let Some(s) = res
                .headers()
                .get(header::ETAG)
                .map(|x| x.to_str().unwrap_or(""))
            {
                Ok(Part {
                    e_tag: s.to_string(),
                    part_number,
                })
            } else {
                Err(res.into())
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
        task: &MultipartUploadTask,
        part_number: usize,
        copy_source: CopySource,
    ) -> Result<Part> {
        if part_number < 1 || part_number > MAX_MULTIPART_COUNT {
            return Err(ValueError::from(
                "part_number is a positive integer between 1 and 10,000.",
            ))?;
        }
        self.executor(Method::PUT)
            .bucket_name(task.bucket())
            .object_name(task.key())
            .query("uploadId", task.upload_id())
            .query("partNumber", part_number.to_string())
            .apply(|e| {
                if let Some(bucket) = task.bucket_owner() {
                    e.header("x-amz-expected-bucket-owner", bucket)
                } else {
                    e
                }
            })
            .headers_merge2(task.ssec_header().cloned())
            .headers_merge(copy_source.args_headers())
            .send_xml_ok()
            .await
            .map(|CopyPartResult { e_tag }| Part { e_tag, part_number })
    }
}
