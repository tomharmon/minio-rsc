use serde::{Deserialize, Serialize};

use crate::error::XmlError;

/// A legal hold configuration for an object.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "PascalCase")]
pub(crate) struct LegalHold {
    status: String,
}

impl LegalHold {
    /// new legal hold configuration with status
    pub(crate) fn new(status: bool) -> Self {
        Self {
            status: (if status { "ON" } else { "OFF" }).to_string(),
        }
    }

    /// Indicates whether the specified object has a legal hold in place.
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

#[test]
fn test_legal_hold() {
    let s = LegalHold::new(true);
    println!("{}", crate::xml::ser::to_string(&s).unwrap());
}
