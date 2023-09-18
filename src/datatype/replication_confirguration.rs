use serde::Deserialize;

use crate::error::{Error, XmlError};

#[derive(Clone, Debug, Deserialize, PartialEq)]
#[serde(rename_all = "PascalCase")]
struct ReplicationRule {
    role: String,
}

/// Object representation of request XML of get_bucket_versioning API and response XML of set_bucket_versioning API.
#[derive(Clone, Debug, Deserialize, PartialEq)]
#[serde(rename_all = "PascalCase")]
pub struct ReplicationConfiguration {
    role: String,
    #[serde(rename = "Rule", default)]
    rules:Vec<ReplicationRule>
}

