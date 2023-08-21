//! Time formatter for S3 APIs.
use chrono::{DateTime, Utc};

/// format date to ISO8601
#[inline]
pub fn aws_format_time(t: &DateTime<Utc>) -> String {
    t.format("%Y%m%dT%H%M%SZ").to_string()
}

/// format date to aws date
#[inline]
pub fn aws_format_date(t: &DateTime<Utc>) -> String {
    t.format("%Y%m%d").to_string()
}