mod complete_multipart_upload_result;
mod initiate_multipart_upload_result;
mod list_buckets_response;
mod list_multipart_uploads_result;
mod list_objects_response;
mod list_parts_result;
use std::collections::HashMap;
use std::ops::IndexMut;

pub use complete_multipart_upload_result::*;
pub use initiate_multipart_upload_result::*;
pub use list_buckets_response::*;
pub use list_multipart_uploads_result::*;
pub use list_objects_response::*;
pub use list_parts_result::*;
use serde::Deserialize;
use serde::Serialize;

use crate::errors::XmlError;
use crate::utils::urlencode;

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

    pub fn tags(self) -> Vec<Tag> {
        self.tag_set.tags
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

    pub fn remove<T1: Into<String>>(&mut self, key: T1) -> Option<Tag> {
        let key: String = key.into();
        let mut i = 0;
        let mut find = false;
        for t in &self.tag_set.tags {
            if t.key == key {
                find = true;
                break;
            }
            i = i + 1;
        }
        if find {
            Some(self.tag_set.tags.remove(i))
        } else {
            None
        }
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
}

impl TryFrom<&str> for Tagging {
    type Error = XmlError;
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        quick_xml::de::from_str(&value).map_err(|x| x.into())
    }
}

/// Object representation of
/// - request XML of PutBucketTagging API and PutObjectTagging API
/// - response XML of GetBucketTagging API and GetObjectTagging API.
#[derive(Debug)]
pub struct Tags(HashMap<String, String>);

impl Tags {
    pub fn new() -> Self {
        Self(HashMap::new())
    }

    pub fn to_xml(&self) -> String {
        let mut result = "<Tagging><TagSet>".to_string();
        for (key, value) in &self.0 {
            result += &format!("<Tag><Key>{}</Key><Value>{}</Value></Tag>", key, value);
        }
        result += "</TagSet></Tagging>";
        return result;
    }

    pub fn to_query(&self) -> String {
        self.0
            .iter()
            .map(|(key, value)| format!("{}={}", urlencode(key, false), urlencode(value, false)))
            .collect::<Vec<String>>()
            .join("=")
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

impl TryFrom<&str> for Tags {
    type Error = XmlError;
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let taggs: Tagging = value.try_into()?;
        Ok(taggs.into())
    }
}

#[test]
fn test_tagging() {
    let result = r#"
    <?xml version="1.0" encoding="UTF-8"?>
    <Tagging>
       <TagSet>
          <Tag>
             <Key>string</Key>
             <Value>string</Value>
          </Tag>
          <Tag>
            <Key>string2</Key>
            <Value>string</Value>
          </Tag>
       </TagSet>
    </Tagging>
    "#;
    let tagging: Tags = result.try_into().unwrap();
    println!("{}", tagging.to_xml())
}
