//! Time formatter for S3 APIs.
use chrono::{DateTime, Utc};

/// wrap of `chrono::Utc`
#[derive(Clone, Copy)]
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

    /// format date to ISO8601
    #[inline]
    pub fn aws_format_time(&self) -> String {
        self.0.format("%Y%m%dT%H%M%SZ").to_string()
    }

    /// format date to aws date
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
