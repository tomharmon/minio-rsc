use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::datatype::{Buckets, CommonPrefix, Initiator, MultipartUpload, Object, Owner, Part};

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
pub(crate) struct InitiateMultipartUploadResult {
    pub bucket: String,
    pub key: String,
    pub upload_id: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct ListAllMyBucketsResult {
    #[serde(default)]
    pub buckets: Buckets,
    pub owner: Owner,
}

// impl ListAllMyBucketsResult {
//     pub fn owner(&self) -> &Owner {
//         &self.owner
//     }

//     pub fn buckets(&self) -> &Vec<Bucket> {
//         &self.buckets.bucket
//     }

//     pub fn into_part(self) -> (Vec<Bucket>, Owner) {
//         (self.buckets.bucket, self.owner)
//     }
// }

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
