//! Data types

mod legal_hold;
mod object_lock_configure;
mod replication_confirguration;
pub mod response;
mod retention;
mod versioning_configuration;
use std::collections::HashMap;
mod list_buckets_response;
mod list_objects_response;
mod multi_part_upload;
mod select_object_content;
mod tags;

pub(crate) use legal_hold::LegalHold;
pub use list_buckets_response::ListAllMyBucketsResult;
pub use list_objects_response::ListBucketResult;
pub use multi_part_upload::{
    CompleteMultipartUploadResult, CopyPartResult, InitiateMultipartUploadResult,
    ListMultipartUploadsResult, ListPartsResult,
};
pub use object_lock_configure::ObjectLockConfiguration;
pub use replication_confirguration::ReplicationConfiguration;
pub use retention::{Retention, RetentionDurationUnit, RetentionMode};
pub use select_object_content::*;
pub use tags::Tags;
pub use versioning_configuration::VersioningConfiguration;

use serde::{Deserialize, Serialize};

use crate::error::XmlError;

#[derive(Clone, Debug, PartialEq)]
pub struct Region(pub String);

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

impl TryFrom<&[u8]> for Region {
    type Error = XmlError;

    fn try_from(res: &[u8]) -> Result<Self, Self::Error> {
        let mut reader = quick_xml::Reader::from_reader(res);
        reader.trim_text(true);
        let mut location = None;
        loop {
            match reader.read_event() {
                Ok(quick_xml::events::Event::Start(ref e)) => {
                    if e.name().as_ref() == b"LocationConstraint" {
                        location = Some(reader.read_text(e.to_end().name())?.into_owned());
                    }
                }
                Err(e) => Err(e)?,
                Ok(quick_xml::events::Event::Eof) => break,
                _ => {}
            }
        }
        return Ok(Region(if let Some(s) = location {
            if s.is_empty() {
                "us-east-1".to_string()
            } else {
                s
            }
        } else {
            "us-east-1".to_string()
        }));
    }
}

impl TryFrom<&str> for Region {
    type Error = XmlError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        value.as_bytes().try_into()
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Bucket {
    /// The name of the bucket.
    pub name: String,
    /// Date the bucket was created. This date can change when making changes to your bucket, such as editing its bucket policy.
    pub creation_date: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "PascalCase")]
pub struct CommonPrefix {
    pub prefix: String,
}

/// Container element that identifies who initiated the multipart upload.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "PascalCase")]
pub struct Initiator {
    pub display_name: String,
    #[serde(rename = "ID")]
    pub id: String,
}

/// A legal hold configuration for an object.
// #[derive(Debug, Clone, Deserialize, PartialEq)]
// #[serde(rename_all = "PascalCase")]
// pub struct LegalHold {
//     pub status: String,
// }

#[derive(Clone, Debug, Deserialize, PartialEq)]
#[serde(rename_all = "PascalCase")]
pub struct MultipartUpload {
    pub checksum_algorithm: String,
    pub upload_id: String,
    pub storage_class: String,
    pub key: String,
    pub initiated: String,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
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

#[derive(Debug, Clone)]
pub struct ObjectStat {
    pub(crate) bucket_name: String,
    pub(crate) object_name: String,
    pub(crate) last_modified: String,
    pub(crate) etag: String,
    pub(crate) content_type: String,
    pub(crate) version_id: String,
    pub(crate) size: usize,
    pub(crate) metadata: HashMap<String, String>,
}

impl ObjectStat {
    pub fn bucket_name(&self) -> &str {
        self.bucket_name.as_ref()
    }

    pub fn object_name(&self) -> &str {
        self.object_name.as_ref()
    }

    pub fn last_modified(&self) -> &str {
        self.last_modified.as_ref()
    }

    pub fn etag(&self) -> &str {
        self.etag.as_ref()
    }

    pub fn content_type(&self) -> &str {
        self.content_type.as_ref()
    }

    pub fn version_id(&self) -> &str {
        self.version_id.as_ref()
    }

    pub fn size(&self) -> usize {
        self.size
    }

    pub fn metadata(&self) -> &HashMap<String, String> {
        &self.metadata
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "PascalCase")]
pub struct Owner {
    pub display_name: String,
    #[serde(rename = "ID")]
    pub id: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "PascalCase")]
pub struct Part {
    pub e_tag: String,
    pub part_number: usize,
}

/// This data type contains information about progress of an operation.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "PascalCase")]
pub struct Progress {
    pub bytes_processed: u64,
    pub bytes_returned: u64,
    pub bytes_scanned: u64,
}

/// Container for the stats details.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "PascalCase")]
pub struct Stats {
    pub bytes_processed: u64,
    pub bytes_returned: u64,
    pub bytes_scanned: u64,
}

/// A container of a key value name pair.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "PascalCase")]
pub struct Tag {
    pub key: String,
    pub value: String,
}

/// A collection for a set of tags
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "PascalCase")]
pub struct TagSet {
    #[serde(rename = "Tag", default)]
    pub tags: Vec<Tag>,
}

/// Container for TagSet elements.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "PascalCase")]
pub struct Tagging {
    pub tag_set: TagSet,
}

//////////////////  Enum Type

pub enum ChecksumAlgorithm {
    CRC32,
    CRC32C,
    SHA1,
    SHA256,
}
