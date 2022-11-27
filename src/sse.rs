use hyper::HeaderMap;

use crate::{errors::ValueError, utils::md5sum_hash};

/// Server-side encryption base class.
pub trait Sse {
    /// Return headers.
    fn headers(&self) -> HeaderMap;

    /// Return copy headers.
    fn copy_headers(&self) -> HeaderMap {
        HeaderMap::new()
    }

    /// Return TLS required to use this server-side encryption.
    fn tls_required(&self) -> bool {
        true
    }
}

pub struct SseCustomerKey {
    headers: HeaderMap,
    copy_headers: HeaderMap,
}

/// Server-side encryption - customer key type.
impl SseCustomerKey {
    pub fn new(key: &str) -> Result<Self, ValueError> {
        if key.len() != 32 {
            return Err(ValueError::from(
                "SSE-C keys need to be 256 bit base64 encoded",
            ));
        }
        let b64_key = base64::encode(key);
        let md5_key = md5sum_hash(key.as_bytes());
        let mut headers = HeaderMap::new();
        headers.insert(
            "X-Amz-Server-Side-Encryption-Customer-Algorithm",
            "AES256".parse().unwrap(),
        );
        headers.insert(
            "X-Amz-Server-Side-Encryption-Customer-Key",
            b64_key.parse().unwrap(),
        );
        headers.insert(
            "X-Amz-Server-Side-Encryption-Customer-Key-MD5",
            md5_key.parse().unwrap(),
        );
        let mut copy_headers = HeaderMap::new();
        copy_headers.insert(
            "X-Amz-Copy-Source-Server-Side-Encryption-Customer-Algorithm",
            "AES256".parse().unwrap(),
        );
        copy_headers.insert(
            "X-Amz-Copy-Source-Server-Side-Encryption-Customer-Key",
            b64_key.parse().unwrap(),
        );
        copy_headers.insert(
            "X-Amz-Copy-Source-Server-Side-Encryption-Customer-Key-MD5",
            md5_key.parse().unwrap(),
        );
        Ok(Self {
            headers,
            copy_headers,
        })
    }
}

impl Sse for SseCustomerKey {
    fn headers(&self) -> HeaderMap {
        self.headers.clone()
    }

    fn copy_headers(&self) -> HeaderMap {
        self.copy_headers.clone()
    }
}

/// Server-side encryption - KMS type.
struct SseKMS(HeaderMap);

impl SseKMS {
    pub fn new(key: &str, content_json: Option<String>) -> Self {
        let mut header = HeaderMap::new();
        header.insert(
            "X-Amz-Server-Side-Encryption-Aws-Kms-Key-Id",
            key.parse().unwrap(),
        );
        if let Some(content) = content_json {
            header.insert(
                "X-Amz-Server-Side-Encryption-Context",
                base64::encode(content.as_bytes()).parse().unwrap(),
            );
        }
        Self(header)
    }
}

impl Sse for SseKMS {
    fn headers(&self) -> HeaderMap {
        self.0.clone()
    }
}

/// Server-side encryption - S3 type.
struct SseS3(HeaderMap);

impl SseS3 {
    pub fn new() -> Self {
        let mut header = HeaderMap::new();
        header.insert("X-Amz-Server-Side-Encryption", "AES256".parse().unwrap());
        Self(header)
    }
}

impl Sse for SseS3 {
    fn headers(&self) -> HeaderMap {
        self.0.clone()
    }

    fn tls_required(&self) -> bool {
        false
    }
}
