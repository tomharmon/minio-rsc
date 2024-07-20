//! Time formatter for S3 APIs.
use chrono::{DateTime, Utc};
use serde::{Deserialize, Deserializer, Serialize};

/// wrap of `chrono::Utc`
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct UtcTime(DateTime<Utc>);

impl UtcTime {
    #[inline]
    pub fn new(datetime: DateTime<Utc>) -> Self {
        Self(datetime)
    }

    /// Returns current utc time
    #[inline]
    pub fn now() -> Self {
        Self::new(Utc::now())
    }

    #[inline]
    pub(crate) fn before(&self, timestamp: i64) -> bool {
        timestamp < self.0.timestamp()
    }

    /// format date to ISO8601, like`2023-09-10T08:26:43.296Z`
    #[inline]
    pub fn format_time(&self) -> String {
        self.0.format("%Y-%m-%dT%H:%M:%S%.3fZ").to_string()
    }

    /// format date to ISO8601, like`20230910T082643Z`
    ///
    /// Used in S3 signatures.
    #[inline]
    pub fn aws_format_time(&self) -> String {
        self.0.format("%Y%m%dT%H%M%SZ").to_string()
    }

    /// format date to aws date.
    ///
    /// Used in S3 signatures
    #[inline]
    pub fn aws_format_date(&self) -> String {
        self.0.format("%Y%m%d").to_string()
    }
}

impl From<DateTime<Utc>> for UtcTime {
    fn from(datetime: DateTime<Utc>) -> Self {
        Self::new(datetime)
    }
}

impl Default for UtcTime {
    /// default: current utc time.
    fn default() -> Self {
        Self::now()
    }
}

/// format date to ISO8601
#[inline]
pub fn aws_format_time(t: &UtcTime) -> String {
    t.0.format("%Y%m%dT%H%M%SZ").to_string()
}

/// format date to aws date
#[inline]
pub fn aws_format_date(t: &UtcTime) -> String {
    t.0.format("%Y%m%d").to_string()
}

pub fn deserialize_with_str<'de, D>(deserializer: D) -> Result<UtcTime, D::Error>
where
    D: Deserializer<'de>,
{
    let value = <DateTime<Utc>>::deserialize(deserializer)?;
    Ok(UtcTime::new(value))
}
