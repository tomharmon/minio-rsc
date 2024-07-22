use std::collections::HashMap;

use hyper::{
    header::{HeaderName, IntoHeaderName},
    HeaderMap,
};

use crate::{
    datatype::{
        FromXml, InitiateMultipartUploadResult, ObjectLockConfiguration, RetentionMode, Tagging,
        ToXml,
    },
    error::Result,
    sse::{Sse, SseCustomerKey},
    time::UtcTime,
    utils::urlencode,
};

use super::QueryMap;

/// Custom request parameters for bucket operations.
/// ## parmas
/// - `bucket_name`: The bucket name.
/// - `region`: *Optional*, The bucket region.
/// - `expected_bucket_owner`: *Optional*, The account ID of the expected bucket owner.
/// - `extra_headers`: *Optional*, Extra headers for advanced usage.
///
/// **Note**: Some parameters are only valid in specific methods
#[derive(Debug, Clone)]
pub struct BucketArgs {
    pub(crate) name: String,
    pub(crate) region: Option<String>,
    pub(crate) expected_bucket_owner: Option<String>,
    pub(crate) extra_headers: Option<HeaderMap>,
}

impl BucketArgs {
    pub fn new<S: Into<String>>(bucket_name: S) -> Self {
        Self {
            name: bucket_name.into(),
            region: None,
            expected_bucket_owner: None,
            extra_headers: None,
        }
    }

    /// Set object region
    pub fn region(mut self, region: Option<String>) -> Self {
        self.region = region;
        self
    }

    /// Set the account ID of the expected bucket owner.
    pub fn expected_bucket_owner(mut self, expected_bucket_owner: Option<String>) -> Self {
        self.expected_bucket_owner = expected_bucket_owner;
        self
    }

    /// Set extra headers for advanced usage.
    pub fn extra_headers(mut self, extra_headers: Option<HeaderMap>) -> Self {
        self.extra_headers = extra_headers;
        self
    }
}

impl<S> From<S> for BucketArgs
where
    S: Into<String>,
{
    fn from(s: S) -> Self {
        Self::new(s)
    }
}

/// A source object definition for `copy_object` and `upload_part_copy` method.
#[derive(Debug, Clone)]
pub struct CopySource {
    bucket_name: String,
    object_name: String,
    region: Option<String>,
    offset: usize,
    length: usize,
    version_id: Option<String>,
    metadata_replace: bool,
    ssec: Option<HeaderMap>,
    match_etag: Option<String>,
    not_match_etag: Option<String>,
    modified_since: Option<String>,
    unmodified_since: Option<String>,
}

impl CopySource {
    pub fn new<T1: Into<String>, T2: Into<String>>(bucket_name: T1, object_name: T2) -> Self {
        Self {
            bucket_name: bucket_name.into(),
            object_name: object_name.into(),
            region: None,
            version_id: None,
            metadata_replace: false,
            ssec: None,
            match_etag: None,
            not_match_etag: None,
            modified_since: None,
            unmodified_since: None,
            offset: 0,
            length: 0,
        }
    }

    /// Set object region
    pub fn region(mut self, region: Option<String>) -> Self {
        self.region = region;
        self
    }

    /// Used only in `upload_part_copy` method.
    ///
    /// **Note**: length must be greater than 0, or both length and offset are 0.
    pub fn range(mut self, offset: usize, length: usize) -> Self {
        self.offset = offset;
        self.length = length;
        self
    }

    /// When copying an object, preserve all metadata if set `false` (default) or specify new metadata.
    pub fn metadata_replace(mut self, metadata_replace: bool) -> Self {
        self.metadata_replace = metadata_replace;
        self
    }

    /// Set version-ID of the object
    pub fn version_id<T: Into<String>>(mut self, version_id: T) -> Self {
        self.version_id = Some(version_id.into());
        self
    }

    /// Set server-side encryption customer key
    pub fn ssec(mut self, ssec: &SseCustomerKey) -> Self {
        let mut header = ssec.headers();
        header.extend(ssec.copy_headers());
        self.ssec = Some(header);
        self
    }

    pub fn match_etag(mut self, match_etag: Option<String>) -> Self {
        self.match_etag = match_etag;
        self
    }

    pub fn not_match_etag(mut self, not_match_etag: Option<String>) -> Self {
        self.not_match_etag = not_match_etag;
        self
    }

    pub fn modified_since(mut self, modified_since: Option<String>) -> Self {
        self.modified_since = modified_since;
        self
    }

    pub fn unmodified_since(mut self, unmodified_since: Option<String>) -> Self {
        self.unmodified_since = unmodified_since;
        self
    }

    pub(crate) fn args_headers(&self) -> HeaderMap {
        let mut header = HeaderMap::new();
        let mut copy_source =
            urlencode(&format!("/{}/{}", self.bucket_name, self.object_name), true);
        if let Some(version_id) = &self.version_id {
            copy_source = copy_source + "?versionId=" + version_id;
        }
        header.insert("x-amz-copy-source", copy_source.parse().unwrap());
        if let Some(value) = &self.match_etag {
            header.insert("x-amz-copy-source-if-match", value.parse().unwrap());
        }
        if let Some(value) = &self.not_match_etag {
            header.insert("x-amz-copy-source-if-none-match", value.parse().unwrap());
        }
        if self.metadata_replace {
            header.insert("x-amz-metadata-directive", "REPLACE".parse().unwrap());
        }
        if let Some(value) = &self.modified_since {
            header.insert(
                "x-amz-copy-source-if-modified-since",
                value.parse().unwrap(),
            );
        }
        if let Some(value) = &self.unmodified_since {
            header.insert(
                "x-amz-copy-source-if-unmodified-since",
                value.parse().unwrap(),
            );
        }
        if self.offset > 0 || self.length > 0 {
            let ranger = if self.length > 0 {
                format!("bytes={}-{}", self.offset, self.offset + self.length - 1)
            } else {
                format!("bytes={}-", self.offset)
            };
            if let Ok(value) = ranger.parse() {
                header.insert("x-amz-copy-source-range", value);
            }
        }
        if let Some(ssec) = &self.ssec {
            header.extend(ssec.clone());
            for (k, v) in ssec {
                header.insert(k, v.to_owned());
            }
        }
        header
    }
}

/// Custom request parameters for object operations.
/// ## parmas
/// - `name`: The key of object.
/// - `version_id`: *Optional*, Version-ID of the object.
/// - `content_type`: *Optional*, Content type of the object.
/// - `ssec`: *Optional*, Server-side encryption customer key.
/// - `offset`: *Optional*, Start byte position of object data.
/// - `length`: *Optional*, Number of bytes of object data from offset.
/// - `metadata`: *Optional*, user-defined metadata.
/// - `extra_headers`: *Optional*, Extra headers for advanced usage.
///
/// **Note**: Some parameters are only valid in specific methods
#[derive(Debug, Clone)]
pub struct KeyArgs {
    pub(crate) name: String,
    pub(crate) version_id: Option<String>,
    pub(crate) content_type: Option<String>,
    pub(crate) ssec_headers: Option<HeaderMap>,
    pub(crate) offset: usize,
    pub(crate) length: usize,
    pub(crate) extra_headers: Option<HeaderMap>,
    pub(crate) metadata: HashMap<String, String>,
}

impl KeyArgs {
    pub fn new<S: Into<String>>(name: S) -> Self {
        Self {
            name: name.into(),
            extra_headers: None,
            version_id: None,
            content_type: None,
            ssec_headers: None,
            offset: 0,
            length: 0,
            metadata: Default::default(),
        }
    }

    /// Set version-ID of the object
    pub fn version_id(mut self, version_id: Option<String>) -> Self {
        self.version_id = version_id;
        self
    }

    /// Set content-type of the object
    pub fn content_type(mut self, content_type: Option<String>) -> Self {
        self.content_type = content_type;
        self
    }

    /// Set extra headers for advanced usage.
    pub fn extra_headers(mut self, extra_headers: Option<HeaderMap>) -> Self {
        self.extra_headers = extra_headers;
        self
    }

    /// Set server-side encryption customer key
    pub fn ssec(mut self, ssec: &SseCustomerKey) -> Self {
        self.ssec_headers = Some(ssec.headers());
        self
    }

    /// Returns the range of this [`ObjectArgs`].
    pub(crate) fn range(&self) -> Option<String> {
        if self.offset > 0 || self.length > 0 {
            Some(if self.length > 0 {
                format!("bytes={}-{}", self.offset, self.offset + self.length - 1)
            } else {
                format!("bytes={}-", self.offset)
            })
        } else {
            None
        }
    }

    /// Set start byte position of object data when `download` an object.
    /// Valid in the download operation of the object.
    ///
    /// Default: 0
    pub fn offset(mut self, offset: usize) -> Self {
        self.offset = offset;
        self
    }

    /// Set number of bytes of object data from offset when `download` an object.
    /// If set length 0, it means to the end of the object.
    ///
    /// Default: 0
    pub fn length(mut self, length: usize) -> Self {
        self.length = length;
        self
    }

    /// Set user-defined metadata when `uploading` an object.
    /// Metadata is a set of key-value pairs.
    ///
    /// key:
    /// - requirement is ASCII and cannot contain non-ASCII characters
    /// - Cannot contain invisible characters and spaces
    /// - does't need to start with `x-amz-meta-`
    /// - ignoring case
    ///
    pub fn metadata(mut self, metadata: HashMap<String, String>) -> Self {
        self.metadata = metadata;
        self
    }

    /// Returns the metadata header of this [`ObjectArgs`].
    pub(crate) fn get_metadata_header(&self) -> Result<HeaderMap> {
        let mut meta_header: HeaderMap = HeaderMap::new();
        for (key, value) in &self.metadata {
            let key = HeaderName::from_bytes(format!("x-amz-meta-{}", key).as_bytes())?;
            meta_header.insert(key, value.parse()?);
        }
        Ok(meta_header)
    }
}

impl<S> From<S> for KeyArgs
where
    S: Into<String>,
{
    fn from(name: S) -> Self {
        Self::new(name)
    }
}

/// Custom `list_multipart_uploads` request parameters
#[derive(Debug, Clone)]
pub struct ListMultipartUploadsArgs {
    bucket_name: String,
    delimiter: String,
    encoding_type: String,
    key_marker: Option<String>,
    max_uploads: usize,
    prefix: String,
    upload_id_marker: Option<String>,
    extra_headers: Option<HeaderMap>,
    extra_query_params: Option<String>,
    expected_bucket_owner: Option<String>,
}

impl ListMultipartUploadsArgs {
    pub fn new(bucket_name: String) -> Self {
        Self {
            bucket_name,
            delimiter: "".to_string(),
            encoding_type: "".to_string(),
            max_uploads: 1000,
            prefix: "".to_string(),
            key_marker: None,
            upload_id_marker: None,
            expected_bucket_owner: None,
            extra_query_params: None,
            extra_headers: None,
        }
    }

    pub fn bucket_name(&self) -> &str {
        &self.bucket_name
    }

    pub fn delimiter<T: Into<String>>(mut self, delimiter: T) -> Self {
        self.delimiter = delimiter.into();
        self
    }

    pub fn encoding_type<T: Into<String>>(mut self, encoding_type: T) -> Self {
        self.encoding_type = encoding_type.into();
        self
    }

    pub fn key_marker<T: Into<String>>(mut self, key_marker: T) -> Self {
        self.key_marker = Some(key_marker.into());
        self
    }

    pub fn upload_id_marker<T: Into<String>>(mut self, upload_id_marker: T) -> Self {
        self.upload_id_marker = Some(upload_id_marker.into());
        self
    }

    pub fn max_uploads(mut self, max_uploads: usize) -> Self {
        self.max_uploads = max_uploads;
        if self.max_uploads > 1000 {
            self.max_uploads = 1000;
        }
        self
    }

    pub fn prefix<T: Into<String>>(mut self, prefix: T) -> Self {
        self.prefix = prefix.into();
        self
    }

    pub fn expected_bucket_owner<T: Into<String>>(mut self, expected_bucket_owner: T) -> Self {
        self.expected_bucket_owner = Some(expected_bucket_owner.into());
        self
    }

    /// Set extra query parameters for advanced usage.
    pub fn extra_query_params(mut self, extra_query_params: Option<String>) -> Self {
        self.extra_query_params = extra_query_params;
        self
    }

    /// Set extra headers for advanced usage.
    pub fn extra_headers(mut self, extra_headers: Option<HeaderMap>) -> Self {
        self.extra_headers = extra_headers;
        self
    }

    pub(crate) fn args_query_map(&self) -> QueryMap {
        let mut querys: QueryMap = QueryMap::default();
        querys.insert("uploads".to_string(), "".to_string());
        querys.insert("delimiter".to_string(), self.delimiter.to_string());
        querys.insert("max-uploads".to_string(), self.max_uploads.to_string());
        querys.insert("prefix".to_string(), self.prefix.to_string());
        querys.insert("encoding-type".to_string(), self.encoding_type.to_string());
        if let Some(encoding_type) = &self.key_marker {
            querys.insert("key-marker".to_string(), encoding_type.to_string());
        }
        if let Some(delimiter) = &self.upload_id_marker {
            querys.insert("upload-id-marker".to_string(), delimiter.clone());
        }
        return querys;
    }

    pub(crate) fn args_headers(&self) -> HeaderMap {
        let mut headermap = HeaderMap::new();
        if let Some(owner) = &self.expected_bucket_owner {
            if let Ok(val) = owner.parse() {
                headermap.insert("x-amz-expected-bucket-owner", val);
            }
        }
        headermap
    }
}

pub struct ListObjectVersionsArgs {
    pub delimiter: Option<String>,
    pub encoding_type: Option<String>,
    pub extra_headers: Option<HeaderMap>,
    /// Specifies the key to start with when listing objects in a bucket.
    pub key_marker: Option<String>,
    pub prefix: Option<String>,
    /// Sets the maximum number of keys returned in the response. Default 1,000
    pub max_keys: usize,
    /// Specifies the object version you want to start listing from.
    pub version_id_marker: Option<String>,
}

impl Default for ListObjectVersionsArgs {
    fn default() -> Self {
        Self {
            extra_headers: None,
            delimiter: None,
            encoding_type: None,
            max_keys: 1000,
            prefix: None,
            key_marker: None,
            version_id_marker: None,
        }
    }
}

impl ListObjectVersionsArgs {
    pub(crate) fn args_query_map(&self) -> QueryMap {
        let mut querys: QueryMap = QueryMap::default();
        querys.insert("versions".to_string(), "".to_string());
        if let Some(delimiter) = &self.delimiter {
            querys.insert("delimiter".to_string(), delimiter.clone());
        }
        if let Some(encoding_type) = &self.encoding_type {
            querys.insert("encoding-type".to_string(), encoding_type.clone());
        }
        if let Some(key_marker) = &self.key_marker {
            querys.insert("key-marker".to_string(), key_marker.clone());
        }
        if let Some(prefix) = &self.prefix {
            querys.insert("prefix".to_string(), prefix.clone());
        }
        if let Some(version_id_marker) = &self.version_id_marker {
            querys.insert("version-id-marker".to_string(), version_id_marker.clone());
        }
        querys.insert("max-keys".to_string(), format!("{}", self.max_keys));
        querys
    }
}

/// Custom `list_objects` request parameters
/// ## parmas
/// - prefix: Limits the response to keys that begin with the specified prefix.
/// - delimiter: A delimiter is a character you use to group keys.
/// - continuation_token: ContinuationToken indicates Amazon S3 that the list is being continued on this bucket with a token.
/// - max_keys: Sets the maximum number of keys returned in the response. Default 1000
/// - encoding_type:Encoding type used by Amazon S3 to encode object keys in the response.Valid Values: `url`
#[derive(Debug, Clone)]
pub struct ListObjectsArgs {
    pub(crate) continuation_token: Option<String>,
    pub(crate) delimiter: Option<String>,
    pub(crate) use_encoding_type: bool,
    pub(crate) fetch_owner: bool,
    pub(crate) start_after: Option<String>,
    pub(crate) max_keys: usize,
    pub(crate) prefix: Option<String>,
    pub(crate) extra_headers: Option<HeaderMap>,
}

impl Default for ListObjectsArgs {
    fn default() -> Self {
        Self {
            continuation_token: None,
            delimiter: None,
            fetch_owner: false,
            max_keys: 1000,
            prefix: None,
            start_after: None,
            use_encoding_type: false,
            extra_headers: None,
        }
    }
}

impl ListObjectsArgs {
    pub fn continuation_token<T: Into<String>>(mut self, token: T) -> Self {
        self.continuation_token = Some(token.into());
        self
    }

    pub fn delimiter<T: Into<String>>(mut self, delimiter: T) -> Self {
        self.delimiter = Some(delimiter.into());
        self
    }

    pub fn use_encoding_type(mut self, use_encoding_type: bool) -> Self {
        self.use_encoding_type = use_encoding_type;
        self
    }

    pub fn fetch_owner(mut self, fetch_owner: bool) -> Self {
        self.fetch_owner = fetch_owner;
        self
    }

    pub fn start_after<T: Into<String>>(mut self, start_after: T) -> Self {
        self.start_after = Some(start_after.into());
        self
    }

    pub fn max_keys(mut self, max_keys: usize) -> Self {
        self.max_keys = max_keys;
        if self.max_keys > 1000 {
            self.max_keys = 1000;
        }
        self
    }

    pub fn prefix<T: Into<String>>(mut self, prefix: T) -> Self {
        self.prefix = Some(prefix.into());
        self
    }

    /// Set extra headers for advanced usage.
    pub fn extra_headers(mut self, extra_headers: Option<HeaderMap>) -> Self {
        self.extra_headers = extra_headers;
        self
    }

    pub(crate) fn args_query_map(&self) -> QueryMap {
        let mut querys: QueryMap = QueryMap::default();
        querys.insert("list-type".to_string(), "2".to_string());

        if self.use_encoding_type {
            querys.insert("encoding-type".to_string(), "url".to_string());
        }
        if let Some(delimiter) = &self.delimiter {
            querys.insert("delimiter".to_string(), delimiter.clone());
        }
        if let Some(token) = &self.continuation_token {
            querys.insert("continuation-token".to_string(), token.clone());
        }
        if self.fetch_owner {
            querys.insert("fetch-owner".to_string(), "true".to_string());
        }
        if let Some(prefix) = &self.prefix {
            querys.insert("prefix".to_string(), prefix.clone());
        }
        if let Some(start_after) = &self.start_after {
            querys.insert("start-after".to_string(), start_after.clone());
        }
        querys.insert("max-keys".to_string(), format!("{}", self.max_keys));
        return querys;
    }
}

/// Custom request parameters for multiUpload operations.
///
/// Used in `abort_multipart_upload`, `complete_multipart_upload`, `create_multipart_upload`,
/// `MultipartUploadArgs`, `upload_part`, `upload_part_copy` method.
#[derive(Debug, Clone)]
pub struct MultipartUploadTask {
    bucket: String,
    key: String,
    upload_id: String,
    bucket_owner: Option<String>,
    content_type: Option<String>,
    ssec_header: Option<HeaderMap>,
}

impl From<InitiateMultipartUploadResult> for MultipartUploadTask {
    fn from(i: InitiateMultipartUploadResult) -> Self {
        Self::new(i.bucket, i.key, i.upload_id, None, None, None)
    }
}

impl MultipartUploadTask {
    pub fn new(
        bucket: String,
        key: String,
        upload_id: String,
        bucket_owner: Option<String>,
        content_type: Option<String>,
        ssec_header: Option<HeaderMap>,
    ) -> Self {
        Self {
            bucket,
            key,
            upload_id,
            bucket_owner,
            content_type,
            ssec_header,
        }
    }

    pub fn bucket(&self) -> &str {
        self.bucket.as_ref()
    }

    pub fn key(&self) -> &str {
        self.key.as_ref()
    }

    pub fn upload_id(&self) -> &str {
        self.upload_id.as_ref()
    }

    pub fn content_type(&self) -> Option<&String> {
        self.content_type.as_ref()
    }

    pub fn bucket_owner(&self) -> Option<&String> {
        self.bucket_owner.as_ref()
    }

    pub fn ssec_header(&self) -> Option<&HeaderMap> {
        self.ssec_header.as_ref()
    }

    pub(crate) fn set_ssec(&mut self, ssec: SseCustomerKey) {
        self.ssec_header = Some(ssec.headers());
    }

    pub(crate) fn set_ssec_header(&mut self, ssec_header: Option<HeaderMap>) {
        self.ssec_header = ssec_header;
    }

    pub(crate) fn set_bucket_owner(&mut self, bucket_owner: Option<String>) {
        self.bucket_owner = bucket_owner;
    }
}

/// The container element for Object Lock configuration parameters.\
/// see `put_object_lock_configuration` and `get_object_lock_configuration` API.
///
/// **Note**: both `mode` and `duration` settings will be effective.
#[derive(Debug, Clone, Default)]
pub struct ObjectLockConfig {
    /// Valid Values: GOVERNANCE | COMPLIANCE
    mode: String,
    /// The date on which this Object Lock Retention will expire.
    duration: usize,
    /// Valid Values: Days | Years
    duration_unit: String,
}

impl ObjectLockConfig {
    pub fn new(duration: usize, is_day: bool, is_governance: bool) -> Self {
        let mut obj = Self::default();
        obj.config(duration, is_day, is_governance);
        obj
    }

    /// - is_day: set period `Days` if true, otherwise set mode `Years`
    /// - is_governance: set mode `GOVERNANCE` if true, otherwise set mode `COMPLIANCE`.
    pub fn config(&mut self, duration: usize, is_day: bool, is_governance: bool) {
        self.duration = duration;
        self.duration_unit = (if is_day { "Days" } else { "Years" }).to_string();
        self.mode = (if is_governance {
            "GOVERNANCE"
        } else {
            "COMPLIANCE"
        })
        .to_string();
    }

    /// The date on which this Object Lock Retention will expire.
    pub fn duration(&self) -> usize {
        self.duration
    }

    /// Valid Values: GOVERNANCE | COMPLIANCE
    pub fn mode(&self) -> &str {
        self.mode.as_ref()
    }

    /// period, Valid Values: Days | Years | Empty String
    pub fn period(&self) -> &str {
        self.duration_unit.as_ref()
    }
}

impl ToXml for ObjectLockConfig {
    fn to_xml(&self) -> crate::error::Result<String> {
        let mut result =
            "<ObjectLockConfiguration><ObjectLockEnabled>Enabled</ObjectLockEnabled>".to_string();
        if !self.mode.is_empty() && !self.duration_unit.is_empty() {
            result += "<Rule><DefaultRetention>";
            result += &format!("<Mode>{}</Mode>", self.mode);
            result += &format!(
                "<{}>{}</{}>",
                self.duration_unit, self.duration, self.duration_unit
            );
            result += "</DefaultRetention></Rule>";
        }
        result += "</ObjectLockConfiguration>";
        Ok(result)
    }
}

impl FromXml for ObjectLockConfig {
    fn from_xml(value: String) -> crate::error::Result<Self> {
        let obj = crate::xml::de::from_str::<ObjectLockConfiguration>(&value)?;
        if let Some(rule) = obj.rule {
            let mode = if rule.default_retention.mode == RetentionMode::GOVERNANCE {
                "GOVERNANCE"
            } else {
                "COMPLIANCE"
            };
            if let Some(duration) = rule.default_retention.days {
                Ok(Self {
                    mode: mode.to_owned(),
                    duration,
                    duration_unit: "Days".to_owned(),
                })
            } else if let Some(duration) = rule.default_retention.years {
                Ok(Self {
                    mode: mode.to_owned(),
                    duration,
                    duration_unit: "Years".to_owned(),
                })
            } else {
                Ok(Default::default())
            }
        } else {
            Ok(Default::default())
        }
    }
}

/// Custom request parameters for presigned URL
/// ## param
/// - bucket_name: Name of the bucket.
/// - object_name: Object name in the bucket.
/// - expires: Expiry in seconds; defaults to 7 days.
/// - headers: Optional response_headers argument to specify response fields like date, size, type of file, data about server, etc.
/// - request_date: Optional request_date argument to specify a different request date. Default is current date.
/// - version_id: Version ID of the object.
/// - querys: Extra query parameters for advanced usage.
#[derive(Clone)]
pub struct PresignedArgs {
    pub(crate) region: Option<String>,
    pub(crate) bucket_name: String,
    pub(crate) object_name: String,
    pub(crate) version_id: Option<String>,
    pub(crate) expires: usize,
    pub(crate) request_date: Option<UtcTime>,
    pub(crate) headers: Option<HeaderMap>,
    pub(crate) querys: QueryMap,
}

impl PresignedArgs {
    pub fn new<T1: Into<String>, T2: Into<String>>(bucket_name: T1, object_name: T2) -> Self {
        Self {
            region: None,
            bucket_name: bucket_name.into(),
            object_name: object_name.into(),
            version_id: None,
            expires: 604800,
            request_date: None,
            headers: None,
            querys: QueryMap::new(),
        }
    }

    pub fn region<T: Into<String>>(mut self, region: T) -> Self {
        self.region = Some(region.into());
        self
    }

    pub fn version_id<T: Into<String>>(mut self, version_id: T) -> Self {
        self.version_id = Some(version_id.into());
        self
    }

    pub fn regirequest_date(mut self, request_date: UtcTime) -> Self {
        self.request_date = Some(request_date);
        self
    }

    pub fn expires(mut self, expires: usize) -> Self {
        self.expires = expires;
        self
    }

    pub fn headers(mut self, header: HeaderMap) -> Self {
        self.headers = Some(header);
        self
    }

    pub fn header<K>(mut self, key: K, value: &str) -> Self
    where
        K: IntoHeaderName,
    {
        let mut headers = self.headers.unwrap_or(HeaderMap::new());
        if let Ok(value) = value.parse() {
            headers.insert(key, value);
        }
        self.headers = Some(headers);
        self
    }

    pub fn querys(mut self, querys: QueryMap) -> Self {
        self.querys = querys;
        self
    }

    pub fn query<K: Into<String>, V: Into<String>>(mut self, key: K, value: V) -> Self {
        self.querys.insert(key.into(), value.into());
        self
    }

    pub fn query_string(mut self, query_str: &str) -> Self {
        self.querys.merge_str(query_str);
        self
    }

    pub fn apply<F>(self, apply: F) -> Self
    where
        F: FnOnce(Self) -> Self,
    {
        apply(self)
    }
}

/// Tags
/// - request XML of put_bucket_tags API and put_object_tags API
/// - response XML of set_bucket_tags API and set_object_tags API.
#[derive(Debug, Clone)]
pub struct Tags(HashMap<String, String>);

impl Tags {
    pub fn new() -> Self {
        Self(HashMap::new())
    }

    pub fn to_query(&self) -> String {
        self.0
            .iter()
            .map(|(key, value)| format!("{}={}", urlencode(key, false), urlencode(value, false)))
            .collect::<Vec<String>>()
            .join("&")
    }

    pub fn insert<K: Into<String>, V: Into<String>>(&mut self, key: K, value: V) -> &mut Self {
        self.0.insert(key.into(), value.into());
        self
    }

    pub fn into_map(self) -> HashMap<String, String> {
        self.0
    }
}

impl From<HashMap<String, String>> for Tags {
    fn from(inner: HashMap<String, String>) -> Self {
        Self(inner)
    }
}

impl std::ops::Deref for Tags {
    type Target = HashMap<String, String>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::ops::DerefMut for Tags {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl From<Tagging> for Tags {
    fn from(tagging: Tagging) -> Self {
        let mut map = HashMap::new();
        for tag in tagging.tag_set.tags {
            map.insert(tag.key, tag.value);
        }
        Self(map)
    }
}

impl FromXml for Tags {
    fn from_xml(v: String) -> crate::error::Result<Self> {
        crate::xml::de::from_string::<Tagging>(v)
            .map(Into::into)
            .map_err(Into::into)
    }
}

impl ToXml for Tags {
    fn to_xml(&self) -> crate::error::Result<String> {
        let mut result = "<Tagging><TagSet>".to_string();
        for (key, value) in &self.0 {
            result += &format!("<Tag><Key>{}</Key><Value>{}</Value></Tag>", key, value);
        }
        result += "</TagSet></Tagging>";
        return Ok(result);
    }
}
