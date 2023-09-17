use crate::error::XmlError;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Default)]
enum DefaultRetentionMode {
    #[default]
    GOVERNANCE,
    COMPLIANCE,
}

/// The container element for specifying the default Object Lock retention settings
/// for new objects placed in the specified bucket.
///
/// **Note**
/// - The DefaultRetention settings require **both** a `mode` and a `period`.
/// - The DefaultRetention period can be either Days or Years but you must select one.
///   You cannot specify Days and Years at the same time.
#[derive(Deserialize, Serialize, Default)]
#[serde(rename_all = "PascalCase")]
struct DefaultRetention {
    days: Option<usize>,
    /// The default Object Lock retention mode you want to apply to new objects placed in the specified bucket.
    /// Valid Values: `GOVERNANCE | COMPLIANCE`
    mode: String,
    years: Option<usize>,
}

/// The container element for an Object Lock rule.
#[derive(Deserialize, Serialize, Default)]
#[serde(rename_all = "PascalCase")]
struct ObjectLockRule {
    default_retention: DefaultRetention,
}

/// Object representation of
/// - request XML of `put_object_lock_configuration` API
/// - response XML of `get_object_lock_configuration` API.
#[derive(Deserialize, Serialize)]
#[serde(rename_all = "PascalCase", rename = "ObjectLockConfiguration")]
struct InnerObjectLockConfiguration {
    /// Indicates whether this bucket has an Object Lock configuration enabled.
    /// Enable ObjectLockEnabled when you apply ObjectLockConfiguration to a bucket.
    ///
    /// Valid Values: `Enabled`
    /// Required: No
    object_lock_enabled: String,
    rule: Option<ObjectLockRule>,
}

impl TryFrom<&str> for InnerObjectLockConfiguration {
    type Error = XmlError;
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        quick_xml::de::from_str(value).map_err(|x| x.into())
    }
}

/// The container element for Object Lock configuration parameters.\
/// see `put_object_lock_configuration` and `get_object_lock_configuration` API.
///
/// **Note**: both `mode` and `duration` settings will be effective.
#[derive(Clone, Debug, Deserialize, PartialEq, Default)]
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
    pub fn new(duration: usize, is_day: bool, is_governance: bool) -> Self {
        let mut obj = Self::default();
        obj.config(duration, is_day, is_governance);
        obj
    }

    /// - is_day: set period `Days` if true, otherwise set mode `Years`
    /// - is_governance: set mode `GOVERNANCE` if true, otherwise set mode `COMPLIANCE`.
    pub fn config(&mut self, duration: usize, is_day: bool, is_governance: bool) {
        self.duration = duration;
        self.duration_unit = (if is_day { "Days" } else { "Years" }).to_string();
        self.mode = (if is_governance {
            "GOVERNANCE"
        } else {
            "COMPLIANCE"
        })
        .to_string();
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

    /// The date on which this Object Lock Retention will expire.
    pub fn duration(&self) -> usize {
        self.duration
    }

    /// Valid Values: GOVERNANCE | COMPLIANCE
    pub fn mode(&self) -> &str {
        self.mode.as_ref()
    }

    /// period, Valid Values: Days | Years | Empty String
    pub fn period(&self) -> &str {
        self.duration_unit.as_ref()
    }
}

impl From<InnerObjectLockConfiguration> for ObjectLockConfiguration {
    fn from(inner: InnerObjectLockConfiguration) -> Self {
        if let Some(ObjectLockRule { default_retention }) = inner.rule {
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
            return Self::default();
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
    <ObjectLockEnabled>Enabled</ObjectLockEnabled>
    <Rule>
       <DefaultRetention>
          <Days>112</Days>
          <Mode>GOVERNANCE</Mode>
          <Years>1221</Years>
       </DefaultRetention>
    </Rule>
</ObjectLockConfiguration>
"#;
    let result: ObjectLockConfiguration = res.try_into().unwrap();
    println!("{:?}", result.mode());
}
