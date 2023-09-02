//! Data types

pub mod args;
mod legal_hold;
mod object_lock_configure;
mod replication_confirguration;
pub mod response;
mod retention;
mod versioning_configuration;
use std::collections::HashMap;

pub(crate) use legal_hold::LegalHold;
pub use object_lock_configure::ObjectLockConfiguration;
pub use replication_confirguration::ReplicationConfiguration;
pub use retention::Retention;
pub use versioning_configuration::VersioningConfiguration;

use serde::{Deserialize, Serialize};

use crate::{
    errors::XmlError,
    utils::{is_urlencoded, urlencode},
};

#[derive(Default, Clone, Debug)]
pub struct QueryMap(Vec<(String, String)>);

impl QueryMap {
    pub fn new() -> Self {
        Self(Vec::new())
    }

    pub fn from_str(query_str: &str) -> Self {
        let mut qm = Self::new();
        qm.merge_str(query_str);
        qm
    }

    pub fn insert(&mut self, key: String, value: String) {
        self.0.push((key, value))
    }

    pub fn merge(&mut self, querys: Self) {
        for query in querys.0 {
            self.0.push(query);
        }
    }

    pub fn merge_str(&mut self, query_str: &str) {
        for query in query_str.split("&").filter(|x| !x.is_empty()) {
            let index = query.find("=");
            if let Some(i) = index {
                self.insert(query[0..i].to_string(), query[i + 1..].to_string());
            } else {
                self.insert(query.to_string(), "".to_string());
            }
        }
    }

    /// sort query by key
    pub fn sort(&mut self) {
        self.0.sort_by(|x, y| x.0.cmp(&y.0));
    }

    /// get query string.
    /// the empty keys will be skipped.
    /// key and value will be uri encode.
    pub fn to_query_string(self) -> String {
        self.0
            .iter()
            .filter(|(k, _)| !k.is_empty())
            .map(|(k, v)| {
                let k = if !is_urlencoded(k) {
                    urlencode(k, false)
                } else {
                    k.to_owned()
                };
                let v = if !is_urlencoded(v) {
                    urlencode(v, false)
                } else {
                    v.to_owned()
                };
                if v.is_empty() {
                    k
                } else {
                    format!("{k}={v}")
                }
            })
            .collect::<Vec<String>>()
            .join("&")
    }
}

impl Into<String> for QueryMap {
    fn into(self) -> String {
        self.to_query_string()
    }
}

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

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Bucket {
    /// The name of the bucket.
    pub name: String,
    /// Date the bucket was created. This date can change when making changes to your bucket, such as editing its bucket policy.
    pub creation_date: String,
}

#[derive(Debug, Deserialize, PartialEq, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct CommonPrefix {
    pub prefix: String,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "PascalCase")]
pub struct Part {
    e_tag: String,
    part_number: usize,
}

impl Part {
    pub fn new(e_tag: String, part_number: usize) -> Self {
        Self { e_tag, part_number }
    }
    pub fn to_tag(self) -> String {
        format!(
            "<Part><ETag>{}</ETag><PartNumber>{}</PartNumber></Part>",
            self.e_tag, self.part_number
        )
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "PascalCase")]
pub struct Initiator {
    pub display_name: String,
    #[serde(rename = "ID")]
    pub id: String,
}

impl Initiator {
    pub fn to_tag(self) -> String {
        format!(
            "<Initiator><DisplayName>{}</DisplayName><ID>{}</ID></Initiator>",
            self.display_name, self.id
        )
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
#[serde(rename_all = "PascalCase")]
pub struct MultipartUpload {
    pub checksum_algorithm: String,
    pub upload_id: String,
    pub storage_class: String,
    pub key: String,
    pub initiated: String,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
#[serde(rename_all = "PascalCase")]
pub struct Object {
    pub key: String,
    pub last_modified: String,
    pub e_tag: String,
    pub size: usize,
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

#[derive(Clone, Debug, Deserialize, PartialEq)]
#[serde(rename_all = "PascalCase")]
pub struct Owner {
    pub display_name: String,
    #[serde(rename = "ID")]
    pub id: String,
}

impl Owner {
    pub fn to_tag(self) -> String {
        format!(
            "<Owner><DisplayName>{}</DisplayName><ID>{}</ID></Owner>",
            self.display_name, self.id
        )
    }
}
