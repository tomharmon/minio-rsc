//! Server-side encryption

use hyper::HeaderMap;

use crate::{
    error::ValueError,
    utils::{base64_encode, md5sum_hash},
};

/// Server-side encryption base trait.
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

/// Server-side encryption - customer key type.
pub struct SseCustomerKey {
    headers: HeaderMap,
    copy_headers: HeaderMap,
}

impl SseCustomerKey {
    pub fn new(key: &str) -> Result<Self, ValueError> {
        if key.len() != 32 {
            return Err(ValueError::from(
                "SSE-C keys need to be 256 bit base64 encoded",
            ));
        }
        let b64_key = base64_encode(key);
        let md5_key = md5sum_hash(key.as_bytes());
        let mut headers = HeaderMap::new();
        headers.insert(
            "X-Amz-Server-Side-Encryption-Customer-Algorithm",
            "AES256".parse()?,
        );
        headers.insert(
            "X-Amz-Server-Side-Encryption-Customer-Key",
            b64_key.parse()?,
        );
        headers.insert(
            "X-Amz-Server-Side-Encryption-Customer-Key-MD5",
            md5_key.parse()?,
        );
        let mut copy_headers = HeaderMap::new();
        copy_headers.insert(
            "X-Amz-Copy-Source-Server-Side-Encryption-Customer-Algorithm",
            "AES256".parse()?,
        );
        copy_headers.insert(
            "X-Amz-Copy-Source-Server-Side-Encryption-Customer-Key",
            b64_key.parse()?,
        );
        copy_headers.insert(
            "X-Amz-Copy-Source-Server-Side-Encryption-Customer-Key-MD5",
            md5_key.parse()?,
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
pub struct SseKMS(HeaderMap);

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
                base64_encode(content.as_bytes()).parse().unwrap(),
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
pub struct SseS3(HeaderMap);

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
