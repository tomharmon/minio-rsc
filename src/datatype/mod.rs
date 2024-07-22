//! Data types

mod select_object_content;

pub use select_object_content::*;

use serde::{Deserialize, Serialize};

use crate::time::UtcTime;

#[derive(Clone, Debug, PartialEq)]
pub struct Region(pub String);

trait XmlSelf {}

macro_rules! impl_xmlself {
    ($($name:tt )*) => {
        $(
            impl XmlSelf for $name{}
        )*
    };
}

impl_xmlself!(
    CommonPrefix
    LegalHold
    VersioningConfiguration
    Retention
    CompleteMultipartUpload
    CompleteMultipartUploadResult
    InitiateMultipartUploadResult
    ListMultipartUploadsResult
    CopyPartResult
    ListPartsResult
    ListAllMyBucketsResult
    ListBucketResult
    ListVersionsResult
    ServerSideEncryptionConfiguration
    CORSConfiguration
    LocationConstraint
    PublicAccessBlockConfiguration
    AccessControlPolicy
);

pub trait ToXml {
    /// try get xml string
    fn to_xml(&self) -> crate::error::Result<String>;
}

impl<T: Serialize + XmlSelf> ToXml for T {
    fn to_xml(&self) -> crate::error::Result<String> {
        crate::xml::ser::to_string(&self).map_err(Into::into)
    }
}

pub trait FromXml: Sized {
    /// try from xml string
    fn from_xml(v: String) -> crate::error::Result<Self>;
}

impl<'de, T: Deserialize<'de> + XmlSelf> FromXml for T {
    fn from_xml(v: String) -> crate::error::Result<Self> {
        crate::xml::de::from_string(v).map_err(Into::into)
    }
}

impl Region {
    pub fn from<S>(region: S) -> Self
    where
        S: Into<String>,
    {
        return Self(region.into());
    }

    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct AccessControlList {
    pub grant: Vec<Grant>,
}

/// Contains the elements that set the ACL permissions for an object per grantee.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct AccessControlPolicy {
    pub access_control_list: AccessControlList,
    pub owner: Option<Owner>,
}

/// In terms of implementation, a Bucket is a resource.
/// An Amazon S3 bucket name is globally unique, and the namespace is shared by all AWS accounts.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct Bucket {
    /// The name of the bucket.
    pub name: String,
    /// Date the bucket was created. This date can change when making changes to your bucket, such as editing its bucket policy.
    pub creation_date: String,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct Buckets {
    #[serde(default)]
    pub bucket: Vec<Bucket>,
}

/// Container for all (if there are any) keys between Prefix and the next occurrence of the string specified by a delimiter.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct CommonPrefix {
    pub prefix: String,
}

/// The container for the completed multipart upload details.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct CompleteMultipartUpload {
    #[serde(default, rename = "Part")]
    pub parts: Vec<Part>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct CompleteMultipartUploadResult {
    pub bucket: String,
    pub key: String,
    pub e_tag: String,
    pub location: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct CopyPartResult {
    pub e_tag: String,
}

/// Describes the cross-origin access configuration for objects in an Amazon S3 bucket.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct CORSConfiguration {
    #[serde(rename = "CORSRule")]
    pub rules: Vec<CORSRule>,
}

/// Specifies a cross-origin access rule for an Amazon S3 bucket.
#[derive(Debug, Clone, Deserialize, Serialize, Default)]
#[serde(rename_all = "PascalCase")]
pub struct CORSRule {
    /// **Required**. Valid values are `GET`, `PUT`, `HEAD`, `POST`, and `DELETE`.
    #[serde(rename = "AllowedMethod", default)]
    pub allowed_methods: Vec<String>,
    /// **Required**
    #[serde(rename = "AllowedOrigin", default)]
    pub allowed_origins: Vec<String>,
    #[serde(rename = "AllowedHeader", default)]
    pub allowed_headers: Vec<String>,
    #[serde(rename = "ExposeHeader", default)]
    pub expose_headers: Vec<String>,
    #[serde(rename = "ID")]
    pub id: Option<String>,
    pub max_age_seconds: usize,
}

/// The container element for specifying the default Object Lock retention settings
/// for new objects placed in the specified bucket.
///
/// **Note**
/// - The DefaultRetention settings require **both** a `mode` and a `period`.
/// - The DefaultRetention period can be either Days or Years but you must select one.
///   You cannot specify Days and Years at the same time.
#[derive(Debug, Clone, Deserialize, Serialize, Default)]
#[serde(rename_all = "PascalCase")]
pub struct DefaultRetention {
    pub days: Option<usize>,
    pub mode: RetentionMode,
    pub years: Option<usize>,
}

/// Information about the delete marker.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct DeleteMarkerEntry {
    /// The object key.
    pub key: String,
    /// Date and time when the object was last modified.
    pub last_modified: String,
    /// Specifies whether the object is (true) or is not (false) the latest version of an object.
    pub is_latest: bool,
    /// The entity tag is an MD5 hash of that version of the object.
    pub owner: Option<Owner>,
    /// Version ID of an object.
    pub version_id: Option<String>,
}

/// Container for grant information.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct Grant {
    pub grantee: Option<Grantee>,
    pub permission: Option<Permission>,
}

/// Container for the person being granted permissions.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct Grantee {
    pub display_name: Option<String>,
    pub email_address: Option<String>,
    pub id: Option<String>,
    #[serde(alias = "Type", alias = "type")]
    pub r#type: GranteeType,
    pub uri: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "PascalCase")]
pub(crate) struct InitiateMultipartUploadResult {
    pub bucket: String,
    pub key: String,
    pub upload_id: String,
}

/// Container element that identifies who initiated the multipart upload.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct Initiator {
    pub display_name: String,
    #[serde(rename = "ID")]
    pub id: String,
}

/// A legal hold configuration for an object.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct LegalHold {
    pub status: LegalHoldStatus,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct ListAllMyBucketsResult {
    #[serde(default)]
    pub buckets: Buckets,
    pub owner: Owner,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct ListBucketResult {
    pub name: String,
    pub prefix: String,
    pub key_count: usize,
    pub max_keys: usize,
    #[serde(default)]
    pub delimiter: String,
    pub is_truncated: bool,
    pub start_after: Option<String>,
    #[serde(default)]
    pub contents: Vec<Object>,
    #[serde(default)]
    pub common_prefixes: Vec<CommonPrefix>,
    #[serde(default)]
    pub next_continuation_token: String,
    #[serde(default)]
    pub continuation_token: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct ListMultipartUploadsResult {
    pub bucket: String,
    pub key_marker: String,
    pub upload_id_marker: String,
    pub next_key_marker: String,
    pub prefix: String,
    pub delimiter: String,
    pub next_upload_id_marker: String,
    pub max_uploads: usize,
    pub is_truncated: bool,
    #[serde(default, rename = "Upload")]
    pub uploads: Vec<MultipartUpload>,
    #[serde(default)]
    pub common_prefixes: Vec<CommonPrefix>,
    pub encoding_type: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct ListPartsResult {
    pub bucket: String,
    pub key: String,
    pub upload_id: String,
    pub part_number_marker: usize,
    pub max_parts: usize,
    pub next_part_number_marker: usize,
    pub is_truncated: bool,
    #[serde(default, rename = "Part")]
    pub parts: Vec<Part>,
    pub storage_class: String,
    pub checksum_algorithm: String,
    pub initiator: Initiator,
    pub owner: Owner,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct ListVersionsResult {
    /// A flag that indicates whether Amazon S3 returned all of the results
    /// that satisfied the search criteria. If your results were truncated,
    /// you can make a follow-up paginated request by using the `NextKeyMarker`
    /// and `NextVersionIdMarker` response parameters as a starting place in
    /// another request to return the rest of the results.
    pub is_truncated: bool,
    /// All of the keys rolled up into a common prefix count as a single return when calculating the number of returns.
    #[serde(default)]
    pub common_prefixes: Vec<CommonPrefix>,
    #[serde(default, rename = "Version")]
    pub versions: Vec<ObjectVersion>,
    /// Container for an object that is a delete marker.
    #[serde(default, rename = "DeleteMarker")]
    pub delete_markers: Vec<DeleteMarkerEntry>,
    pub name: String,
    pub prefix: String,
    pub max_keys: usize,
    #[serde(default)]
    pub delimiter: String,
    pub encoding_type: Option<String>,
    /// Marks the last key returned in a truncated response.
    #[serde(default)]
    pub key_marker: String,
    /// When the number of responses exceeds the value of `MaxKeys`,
    /// `NextKeyMarker` specifies the first key not returned that
    /// satisfies the search criteria. Use this value for the `key-marker`
    /// request parameter in a subsequent request.
    #[serde(default)]
    pub next_key_marker: String,
    /// Marks the last version of the key returned in a truncated response.
    #[serde(default)]
    pub version_id_marker: String,
    /// When the number of responses exceeds the value of `MaxKeys`,
    /// `NextVersionIdMarker` specifies the first object version not
    /// returned that satisfies the search criteria. Use this value
    /// for the `version-id-marker` request parameter in a subsequent request.
    #[serde(default)]
    pub next_version_id_marker: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct LocationConstraint {
    pub location_constraint: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct MultipartUpload {
    pub checksum_algorithm: String,
    pub upload_id: String,
    pub storage_class: String,
    pub key: String,
    pub initiated: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct Object {
    pub key: String,
    pub last_modified: String,
    pub e_tag: String,
    pub size: u64,
    pub storage_class: String,
    pub owner: Option<Owner>,
    pub checksum_algorithm: Option<String>,
}

/// The container element for an Object Lock rule.
#[derive(Debug, Clone, Deserialize, Serialize, Default)]
#[serde(rename_all = "PascalCase")]
pub struct ObjectLockRule {
    pub default_retention: DefaultRetention,
}

/// Object representation of
/// - request XML of `put_object_lock_configuration` API
/// - response XML of `get_object_lock_configuration` API.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct ObjectLockConfiguration {
    /// Indicates whether this bucket has an Object Lock configuration enabled.
    /// Enable ObjectLockEnabled when you apply ObjectLockConfiguration to a bucket.
    ///
    /// Valid Values: `Enabled`
    /// Required: No
    pub object_lock_enabled: String,
    pub rule: Option<ObjectLockRule>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct ObjectVersion {
    /// The object key.
    pub key: String,
    /// Date and time when the object was last modified.
    pub last_modified: String,
    /// Specifies whether the object is (true) or is not (false) the latest version of an object.
    pub is_latest: bool,
    /// The entity tag is an MD5 hash of that version of the object.
    pub e_tag: String,
    pub size: u64,
    pub storage_class: String,
    pub owner: Option<Owner>,
    /// Version ID of an object.
    pub version_id: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct Owner {
    pub display_name: String,
    #[serde(rename = "ID")]
    pub id: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct Part {
    pub e_tag: String,
    pub part_number: usize,
}

/// This data type contains information about progress of an operation.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct Progress {
    pub bytes_processed: u64,
    pub bytes_returned: u64,
    pub bytes_scanned: u64,
}

/// PublicAccessBlockConfiguration parameters
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct PublicAccessBlockConfiguration {
    pub block_public_acls: bool,
    pub block_public_policy: bool,
    pub ignore_public_acls: bool,
    pub restrict_public_buckets: bool,
}

/// A container for replication rules. You can add up to 1,000 rules. The maximum size of a replication configuration is 2 MB.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct ReplicationConfiguration {
    pub role: String,
    #[serde(rename = "Rule", default)]
    pub rules: Vec<ReplicationRule>,
}

/// Specifies which Amazon S3 objects to replicate and where to store the replicas.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct ReplicationRule {
    pub role: String,
    pub id: Option<String>,
    pub priority: Option<i64>,
}

/// Object representation of request XML of `put_object_retention` API
/// and response XML of `get_object_retention` API.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct Retention {
    /// Valid Values: GOVERNANCE | COMPLIANCE
    pub mode: RetentionMode,
    /// The date on which this Object Lock Retention will expire.
    #[serde(deserialize_with = "crate::time::deserialize_with_str")]
    pub retain_until_date: UtcTime,
}

/// Describes the default server-side encryption to apply to new objects in the bucket.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct ServerSideEncryptionByDefault {
    #[serde(rename = "SSEAlgorithm")]
    pub ssealgorithm: String,
    #[serde(rename = " KMSMasterKeyID")]
    pub kmsmaster_key_id: Option<String>,
}

/// Root level tag for the ServerSideEncryptionConfiguration parameters
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct ServerSideEncryptionConfiguration {
    #[serde(rename = "Rule")]
    pub rules: Vec<ServerSideEncryptionRule>,
}

/// Specifies the default server-side encryption configuration.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct ServerSideEncryptionRule {
    pub apply_server_side_encryption_by_default: ServerSideEncryptionByDefault,
    #[serde(default)]
    pub bucket_key_enabled: bool,
}

/// Container for the stats details.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct Stats {
    pub bytes_processed: u64,
    pub bytes_returned: u64,
    pub bytes_scanned: u64,
}

/// A container of a key value name pair.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct Tag {
    pub key: String,
    pub value: String,
}

/// A collection for a set of tags
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct TagSet {
    #[serde(rename = "Tag", default)]
    pub tags: Vec<Tag>,
}

/// Container for TagSet elements.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct Tagging {
    pub tag_set: TagSet,
}

/// Describes the versioning state of an Amazon S3 bucket.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct VersioningConfiguration {
    /// Specifies whether MFA delete is enabled in the bucket versioning configuration.
    /// This element is only returned if the bucket has been configured with MFA delete.
    /// If the bucket has never been so configured, this element is not returned.
    ///
    /// Valid Values: Enabled | Disabled
    pub mfa_delete: Option<MFADelete>,

    /// The versioning state of the bucket.
    ///
    /// Valid Values: Enabled | Suspended
    pub status: Option<VersioningStatus>,
}

//////////////////  Enum Type

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub enum ChecksumAlgorithm {
    CRC32,
    CRC32C,
    SHA1,
    SHA256,
}

/// Type of grantee
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub enum GranteeType {
    CanonicalUser,
    AmazonCustomerByEmail,
    Group,
}

/// Specifies whether MFA delete is enabled in the bucket versioning configuration.
/// This element is only returned if the bucket has been configured with MFA delete.
/// If the bucket has never been so configured, this element is not returned.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub enum MFADelete {
    Enabled,
    Disabled,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub enum LegalHoldStatus {
    ON,
    OFF,
}

/// Retention mode, Valid Values: `GOVERNANCE | COMPLIANCE`
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Default)]
pub enum RetentionMode {
    #[default]
    GOVERNANCE,
    COMPLIANCE,
}

/// The permission given to the grantee.. Valid Values: `FULL_CONTROL | WRITE | WRITE_ACP | READ | READ_ACP`
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub enum Permission {
    FULL_CONTROL,
    WRITE,
    WRITE_ACP,
    READ,
    READ_ACP,
}

/// Valid Values: `Enabled | Disabled`
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub enum Status {
    Enabled,
    Disabled,
}

/// The versioning state of the bucket.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub enum VersioningStatus {
    Enabled,
    Suspended,
}
