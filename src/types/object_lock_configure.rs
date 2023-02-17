use crate::errors::XmlError;
use serde::Deserialize;

#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
struct DefaultRetention {
    days: Option<usize>,
    mode: String,
    years: Option<usize>,
}

#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
struct Rule {
    default_retention: DefaultRetention,
}

/// Object representation of request XML of `put_object_lock_configuration` API
/// and response XML of `get_object_lock_configuration` API.
#[derive(Deserialize)]
#[serde(rename_all = "PascalCase", rename = "ObjectLockConfiguration")]
struct InnerObjectLockConfiguration {
    object_lock_enabled: String,
    rule: Rule,
}

impl TryFrom<&str> for InnerObjectLockConfiguration {
    type Error = XmlError;
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        quick_xml::de::from_str(value).map_err(|x| x.into())
    }
}

/// Object representation of request XML of `put_object_lock_configuration` API
/// and response XML of `get_object_lock_configuration` API.
#[derive(Clone, Debug, Deserialize, PartialEq)]
#[serde(rename_all = "PascalCase")]
pub struct ObjectLockConfiguration {
    /// Valid Values: GOVERNANCE | COMPLIANCE
    mode: String,
    /// The date on which this Object Lock Retention will expire.
    duration: usize,
    /// Valid Values: Days | Years
    duration_unit: String,
}

impl ObjectLockConfiguration {
    pub fn to_xml(&self) -> String {
        let mut result = "<ObjectLockConfiguration><ObjectLockEnabled>Enabled</ObjectLockEnabled><Rule><DefaultRetention>".to_string();

        result += "</DefaultRetention></Rule></ObjectLockConfiguration>";
        result
    }

    // pub fn retain_until_date(&self) -> usize {
    //     self.retain_until_date
    // }
}

impl From<InnerObjectLockConfiguration> for ObjectLockConfiguration {
    fn from(inner: InnerObjectLockConfiguration) -> Self {
        let (duration, unit) = if let Some(duration) = inner.rule.default_retention.days {
            (duration, "Days")
        } else if let Some(duration) = inner.rule.default_retention.years {
            (duration, "Years")
        } else {
            (0, "")
        };
        Self {
            mode: inner.rule.default_retention.mode,
            duration: duration,
            duration_unit: unit.to_string(),
        }
    }
}

impl TryFrom<&str> for ObjectLockConfiguration {
    type Error = XmlError;
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        quick_xml::de::from_str(value)
            .map_err(|x| x.into())
            .map(|r: InnerObjectLockConfiguration| r.into())
    }
}

#[test]
fn test_object_lock_configure() {
    let res = r#"
<ObjectLockConfiguration>
    <ObjectLockEnabled>string</ObjectLockEnabled>
    <Rule>
       <DefaultRetention>
          <Days>112</Days>
          <Mode>GOVERNANCE</Mode>
          <Years>1221</Years>
       </DefaultRetention>
    </Rule>
 </ObjectLockConfiguration>
"#;
    let result: std::result::Result<ObjectLockConfiguration, XmlError> = res.try_into();
    println!("{:?}", result);
    assert!(result.is_ok());
}
