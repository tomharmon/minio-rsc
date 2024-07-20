#![allow(unused)]
use base64::Engine;
use once_cell::sync::Lazy;
use regex::Regex;

use crate::error::ValueError;

pub static EMPTY_CONTENT_SHA256: &str =
    "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855";

static _VALID_IP_ADDRESS: Lazy<Regex> = Lazy::new(|| Regex::new(r"^(\d+\.){3}\d+$").unwrap());

static _VALID_NAME: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^[a-z0-9][a-z0-9.-]{1,61}[a-z0-9]$").unwrap());

pub static _VALID_ENDPOINT: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^[A-Za-z0-9_\-.]+(:\d+)?$").unwrap());

static _IS_URLENCODE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^([0-9a-zA-Z-.~_]|(%[0-9A-F]{2}))*$").unwrap());

static _VALIE_UUID: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"^[a-fA-F0-9]{8}-[a-fA-F0-9]{4}-[a-fA-F0-9]{4}-[a-fA-F0-9]{4}-[a-fA-F0-9]{12}*$")
        .unwrap()
});

/// Check whether bucket name is valid
pub fn check_bucket_name(name: &str) -> Result<bool, ValueError> {
    if name.len() < 3 || name.len() > 63 {
        Err(ValueError::from(
            "Bucket name must be between 3 (min) and 63 (max) characters long.",
        ))?;
    };
    if !_VALID_NAME.is_match(name) {
        Err(ValueError::from(
            "Bucket name can consist only of lowercase letters, numbers, dots (.), and hyphens (-). must begin and end with a letter or number.",
        ))?;
    }
    if name.find("..").is_some() || name.find(".-").is_some() || name.find("-.").is_some() {
        Err(ValueError::from(
            "Bucket name cannot contain two adjacent periods, or a period adjacent to a hyphen.",
        ))?;
    };
    if name.starts_with("xn--") {
        Err(ValueError::from(
            "Bucket name cannot start with the prefix xn--.",
        ))?;
    }
    if name.ends_with("-s3alias") {
        Err(ValueError::from(
            "Bucket name cannot end with the suffix -s3alias.",
        ))?;
    }
    if _VALID_IP_ADDRESS.is_match(name) {
        Err(ValueError::from("Bucket name cannot be an ip address"))?;
    };
    return Ok(true);
}

/// Encode arbitrary octets as base64 using the provided [base64::engine::general_purpose::STANDARD].
/// Returns a `String`.
#[inline]
pub fn base64_encode<T: AsRef<[u8]>>(input: T) -> String {
    base64::engine::general_purpose::STANDARD.encode(input)
}

/// Compute MD5 of data and return hash as Base64 encoded value.
pub fn md5sum_hash(data: &[u8]) -> String {
    base64_encode(md5::compute(data).0)
}

/// uri encode every byte except the unreserved characters: 'A'-'Z', 'a'-'z', '0'-'9', '-', '.', '_', and '~'.
#[inline]
pub fn urlencode(data: &str, safe_slash: bool) -> String {
    urlencode_binary(data.as_bytes(), safe_slash)
}

pub fn urlencode_binary(data: &[u8], safe_slash: bool) -> String {
    let s = urlencoding::encode_binary(data).into_owned();
    if safe_slash {
        s.replace("%2F", "/")
    } else {
        s
    }
}

/// check text is uuid foramt
pub fn is_uuid(text: &str) -> bool {
    text.len() == 36 && _VALIE_UUID.is_match(text)
}

/// check text is be url encode
pub fn is_urlencoded(text: &str) -> bool {
    _IS_URLENCODE.is_match(text)
}

pub fn trim_bytes(b: &[u8]) -> &[u8] {
    let mut start = 0;
    let mut end = b.len();
    for i in 0..b.len() {
        if !b[i].is_ascii_whitespace() {
            start = i;
            break;
        }
    }
    for i in (0..b.len()).rev() {
        if !b[i].is_ascii_whitespace() {
            end = i + 1;
            break;
        }
    }
    if start <= end {
        &b[start..end]
    } else {
        &b[0..0]
    }
}

#[cfg(test)]
mod tests {
    use crate::utils::{is_urlencoded, trim_bytes};

    use super::check_bucket_name;
    #[test]
    fn test_check_bucket_name() {
        assert!(check_bucket_name("test").is_ok());
        assert!(check_bucket_name("Test").is_err());
        assert!(check_bucket_name("test..t").is_err());
        assert!(check_bucket_name("test-.d").is_err());
        assert!(check_bucket_name("xn--test").is_err());
        assert!(check_bucket_name("test-s3alias").is_err());
        assert!(check_bucket_name("127.0.0.1").is_err());
        assert!(is_urlencoded("uri-encode_.~%AA%20"));
        assert!(!is_urlencoded("uri encode"));
        assert!(!is_urlencoded("uri%2aencode"));
        assert!(!is_urlencoded("uri%2Gencode"));
    }

    #[test]
    fn test_trim_bytes() {
        assert_eq!(trim_bytes(" hello \n".as_bytes()), "hello".as_bytes());
    }
}
