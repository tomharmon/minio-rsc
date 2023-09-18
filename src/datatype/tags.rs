use std::collections::HashMap;

use crate::{error::XmlError, utils::urlencode};

use super::Tagging;

/// Tags
/// - request XML of put_bucket_tags API and put_object_tags API
/// - response XML of set_bucket_tags API and set_object_tags API.
#[derive(Debug, Clone)]
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

    pub fn insert<K: Into<String>, V: Into<String>>(&mut self, key: K, value: V) -> &mut Self {
        self.0.insert(key.into(), value.into());
        self
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
        crate::xml::de::from_str::<Tagging>(value)
            .map_err(XmlError::from)
            .map(Into::into)
    }
}
