use serde::Deserialize;

use crate::errors::XmlError;

/// Object representation of 
/// - request XML of `get_bucket_versioning` API
/// - response XML of `set_bucket_versioning` API.
#[derive(Clone, Debug, Deserialize, PartialEq)]
#[serde(rename_all = "PascalCase")]
pub struct VersioningConfiguration {
    mfa_delete: Option<String>,
    status: Option<String>,
}

impl VersioningConfiguration {
    /// Create a new VersioningConfiguration object with given status.
    pub fn new(status: bool, mfa_delete: Option<bool>) -> Self {
        Self {
            mfa_delete: mfa_delete.map(|m| (if m { "Enabled" } else { "Disabled" }).to_string()),
            status: Some((if status { "Enabled" } else { "Suspended" }).to_string()),
        }
    }

    pub fn is_mfa_delete_enabled(&self) -> bool {
        self.mfa_delete == Some("Enabled".to_string())
    }

    pub fn is_status_enabled(&self) -> bool {
        self.status == Some("Enabled".to_string())
    }

    pub fn set_mfa_delete(&mut self, mfa_delete: bool) {
        self.mfa_delete = Some((if mfa_delete { "Enabled" } else { "Disabled" }).to_string());
    }

    pub fn set_status_enable(&mut self, enable: bool) {
        self.status = Some((if enable { "Enabled" } else { "Suspended" }).to_string());
    }

    pub fn to_xml(&self) -> String {
        let mut result = "<VersioningConfiguration>".to_string();
        if let Some(mfa) = &self.mfa_delete {
            result += &format!("<MfaDelete>{}</MfaDelete>", mfa);
        }
        if let Some(status) = &self.status {
            result += &format!("<Status>{}</Status>", status);
        } else {
            result += "<Status>Suspended</Status>";
        }
        result += "</VersioningConfiguration>";
        println!("{}", result);
        return result;
    }
}

impl TryFrom<&str> for VersioningConfiguration {
    type Error = XmlError;
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        quick_xml::de::from_str(value).map_err(|x| x.into())
    }
}