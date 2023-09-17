use serde::{Deserialize, Serialize};

use crate::{
    error::XmlError,
    types::{CommonPrefix, Object},
};

#[derive(Debug, Deserialize, Serialize, PartialEq)]
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

impl TryFrom<&str> for ListBucketResult {
    type Error = XmlError;
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        quick_xml::de::from_str(&value).map_err(|x| x.into())
    }
}
