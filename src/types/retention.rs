use serde::Deserialize;

use crate::errors::XmlError;

/// Object representation of request XML of `put_object_retention` API
/// and response XML of `get_object_retention` API.
#[derive(Clone, Debug, Deserialize, PartialEq)]
#[serde(rename_all = "PascalCase")]
pub struct Retention {
    /// Valid Values: GOVERNANCE | COMPLIANCE
    mode: String,
    /// The date on which this Object Lock Retention will expire.
    retain_until_date: usize,
}

impl Retention {
    pub fn to_xml(&self) -> String {
        format!(
            "<LegalHold><Mode>{}</Mode><RetainUntilDate>{}</RetainUntilDate></LegalHold>",
            self.mode, self.retain_until_date
        )
    }

    pub fn retain_until_date(&self) -> usize {
        self.retain_until_date
    }
}

impl TryFrom<&str> for Retention {
    type Error = XmlError;
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        quick_xml::de::from_str(value).map_err(|x| x.into())
    }
}

#[test]
fn test_retention() {
    let res = r#"
<Retention>
    <Mode>GOVERNANCE</Mode>
    <RetainUntilDate>54564561</RetainUntilDate>
</Retention>
"#;
    let result: std::result::Result<Retention, XmlError> = res.try_into();
    println!("{:?}", result);
    assert!(result.is_ok());
}
