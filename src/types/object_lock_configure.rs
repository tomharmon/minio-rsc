use crate::errors::XmlError;
use serde::Deserialize;

#[derive(Deserialize, Default)]
#[serde(rename_all = "PascalCase")]
struct DefaultRetention {
    days: Option<usize>,
    mode: String,
    years: Option<usize>,
}

#[derive(Deserialize, Default)]
#[serde(rename_all = "PascalCase")]
struct Rule {
    default_retention: DefaultRetention,
}

/// Object representation of
/// - request XML of `put_object_lock_configuration` API
/// - response XML of `get_object_lock_configuration` API.
#[derive(Deserialize)]
#[serde(rename_all = "PascalCase", rename = "ObjectLockConfiguration")]
struct InnerObjectLockConfiguration {
    object_lock_enabled: String,
    rule: Option<Rule>,
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
    pub fn new() -> Self {
        Self {
            mode: "".to_string(),
            duration: 0,
            duration_unit: "".to_string(), // duration_unit: (if unit { "Days" } else { "Years" }).to_string(),
        }
    }

    pub fn set_mode(&mut self, is_governance: bool) {
        self.mode = (if is_governance { "GOVERNANCE" } else { "COMPLIANCE" }).to_string()
    }

    pub fn set_duration(&mut self, duration: usize, is_day: bool) {
        self.duration = duration;
        self.duration_unit = (if is_day { "Days" } else { "Years" }).to_string()
    }

    pub fn to_xml(&self) -> String {
        let mut result =
            "<ObjectLockConfiguration><ObjectLockEnabled>Enabled</ObjectLockEnabled>".to_string();
        if !self.mode.is_empty() && !self.duration_unit.is_empty() {
            result += "<Rule><DefaultRetention>";
            result += &format!("<Mode>{}</Mode>", self.mode);
            result += &format!(
                "<{}>{}</{}>",
                self.duration_unit, self.duration, self.duration_unit
            );
            result += "</DefaultRetention></Rule>";
        }
        result += "</ObjectLockConfiguration>";
        result
    }
}

impl From<InnerObjectLockConfiguration> for ObjectLockConfiguration {
    fn from(inner: InnerObjectLockConfiguration) -> Self {
        if let Some(Rule { default_retention }) = inner.rule {
            let (duration, unit) = if let Some(duration) = default_retention.days {
                (duration, "Days")
            } else if let Some(duration) = default_retention.years {
                (duration, "Years")
            } else {
                (0, "")
            };
            Self {
                mode: default_retention.mode,
                duration,
                duration_unit: unit.to_string(),
            }
        } else {
            return Self::new();
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
