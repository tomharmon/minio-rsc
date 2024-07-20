use crate::time::UtcTime;

/// Represents credentials access key, secret key and session token.
#[derive(Debug, Clone)]
pub struct Credentials {
    access_key: String,
    secret_key: String,
    session_token: Option<String>,
    expiration: Option<i64>,
}

impl Credentials {
    pub fn new<T1: Into<String>,T2: Into<String>>(ak: T1, sk: T2, st: Option<String>, exp: Option<i64>) -> Self {
        Credentials {
            access_key: ak.into(),
            secret_key: sk.into(),
            session_token: st,
            expiration: exp,
        }
    }

    /// Get access key.
    pub fn access_key(&self) -> &str {
        self.access_key.as_ref()
    }

    /// Get secret key.
    pub fn secret_key(&self) -> &str {
        self.secret_key.as_ref()
    }

    /// Get session token.
    pub fn session_token(&self) -> Option<&String> {
        self.session_token.as_ref()
    }

    /// Check whether this credentials expired or not.
    pub fn is_expired(&self) -> bool {
        if let Some(exp) = self.expiration {
            let now = UtcTime::now();
            now.before(exp - 10)
        } else {
            false
        }
    }
}
