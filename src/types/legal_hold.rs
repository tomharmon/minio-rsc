use serde::Deserialize;

use crate::errors::XmlError;

#[derive(Clone, Debug, Deserialize, PartialEq)]
#[serde(rename_all = "PascalCase")]
pub(crate) struct LegalHold {
    status: String,
}

impl LegalHold {
    pub(crate) fn new(status: bool) -> Self {
        Self {
            status: (if status { "ON" } else { "OFF" }).to_string(),
        }
    }

    pub fn to_xml(&self) -> String {
        format!("<LegalHold><Status>{}</Status></LegalHold>", self.status)
    }

    pub fn is_enable(&self) -> bool {
        self.status == "ON"
    }
}

impl TryFrom<&str> for LegalHold {
    type Error = XmlError;
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        quick_xml::de::from_str(value).map_err(|x| x.into())
    }
}
