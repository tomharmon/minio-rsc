mod complete_multipart_upload_result;
mod copy_part_ressult;
mod initiate_multipart_upload_result;
mod list_buckets_response;
mod list_multipart_uploads_result;
mod list_objects_response;
mod list_parts_result;
mod select_object_reader;

use serde::Deserialize;
use serde::Serialize;
use std::ops::IndexMut;

use crate::error::XmlError;
use crate::utils::urlencode;
pub use complete_multipart_upload_result::*;
pub use copy_part_ressult::*;
pub use initiate_multipart_upload_result::*;
pub use list_buckets_response::*;
pub use list_multipart_uploads_result::*;
pub use list_objects_response::*;
pub use list_parts_result::*;
pub use select_object_reader::*;

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "PascalCase")]
pub struct Tag {
    pub key: String,
    pub value: String,
}

impl Tag {
    pub fn new<T1: Into<String>, T2: Into<String>>(key: T1, value: T2) -> Self {
        Self {
            key: key.into(),
            value: value.into(),
        }
    }

    pub fn to_tag(&self) -> String {
        return format!(
            "<Tag><Key>{}</Key><Value>{}</Value></Tag>",
            self.key, self.value
        );
    }
}

impl<T1: Into<String>, T2: Into<String>> From<(T1, T2)> for Tag {
    fn from((key, value): (T1, T2)) -> Self {
        Self::new(key, value)
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "PascalCase")]
struct TagSet {
    #[serde(rename = "Tag", default)]
    tags: Vec<Tag>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "PascalCase")]
pub struct Tagging {
    tag_set: TagSet,
}

impl Tagging {
    pub fn new() -> Self {
        Self {
            tag_set: TagSet { tags: Vec::new() },
        }
    }

    pub fn insert<T1: Into<String>, T2: Into<String>>(&mut self, key: T1, value: T2) -> &mut Self {
        let key: String = key.into();
        let value: String = value.into();
        if !key.is_empty() && !value.is_empty() {
            let mut i = 0;
            for t in &self.tag_set.tags {
                if t.key == key {
                    break;
                }
                i = i + 1;
            }
            if i >= self.tag_set.tags.len() {
                self.tag_set.tags.push(Tag::new(key, value))
            } else {
                self.tag_set.tags.index_mut(i).value = value;
            }
        }
        self
    }

    pub fn to_xml(&self) -> String {
        let mut result = "<Tagging><TagSet>".to_string();
        for tag in &self.tag_set.tags {
            result += &tag.to_tag();
        }
        result += "</TagSet></Tagging>";
        return result;
    }

    pub fn to_query(&self) -> Option<String> {
        let query: String = self
            .tag_set
            .tags
            .iter()
            .map(|t| {
                format!(
                    "{}={}",
                    urlencode(&t.key, false),
                    urlencode(&t.value, false)
                )
            })
            .collect::<Vec<String>>()
            .join("=");
        if query.is_empty() {
            None
        } else {
            Some(query)
        }
    }

    pub fn tags(self) -> Vec<Tag> {
        self.tag_set.tags
    }
}

impl TryFrom<&str> for Tagging {
    type Error = XmlError;
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        quick_xml::de::from_str(&value).map_err(|x| x.into())
    }
}
