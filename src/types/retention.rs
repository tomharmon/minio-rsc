use serde::Deserialize;
use strum_macros::Display;

use crate::{error::XmlError, time::UtcTime};

/// Duration unit of default retention configuration.
#[derive(Debug, Clone, Copy, PartialEq, Display, Deserialize)]
pub enum RetentionDurationUnit {
    DAYS,
    YEARS,
}

/// Retention mode, Valid Values: `GOVERNANCE | COMPLIANCE`
#[derive(Debug, Clone, Copy, PartialEq, Display, Deserialize)]
pub enum RetentionMode {
    GOVERNANCE,
    COMPLIANCE,
}

/// Object representation of request XML of `put_object_retention` API
/// and response XML of `get_object_retention` API.
#[derive(Clone, Copy, Debug, Deserialize, PartialEq)]
#[serde(rename_all = "PascalCase")]
pub struct Retention {
    /// Valid Values: GOVERNANCE | COMPLIANCE
    #[serde(with = "quick_xml::serde_helpers::text_content")]
    pub mode: RetentionMode,
    /// The date on which this Object Lock Retention will expire.
    pub retain_until_date: UtcTime,
}

impl Retention {
    /// get xml string of Retention.
    pub fn to_xml(&self) -> String {
        format!(
            "<Retention><Mode>{}</Mode><RetainUntilDate>{}</RetainUntilDate></Retention>",
            self.mode,
            self.retain_until_date.format_time()
        )
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
    let res = r#"<Retention><Mode>GOVERNANCE</Mode><RetainUntilDate>2023-09-10T08:16:28.230Z</RetainUntilDate></Retention>"#;
    let result: Retention = res.try_into().unwrap();
    println!("{}", result.to_xml());
    assert_eq!(res, result.to_xml());
}
