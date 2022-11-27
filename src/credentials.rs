#[derive(Debug, Clone)]
pub struct Credentials {
    access_key: String,
    secret_key: String,
    session_token: Option<String>,
    expiration: Option<usize>,
}

impl Credentials {
    pub fn new<T: Into<String>>(ak: T, sk: T, st: Option<String>, exp: Option<usize>) -> Self {
        Credentials {
            access_key: ak.into(),
            secret_key: sk.into(),
            session_token: st.into(),
            expiration: exp,
        }
    }

    pub fn access_key(&self) -> &str {
        &self.access_key
    }

    pub fn secret_key(&self) -> &str {
        &self.secret_key
    }

    pub fn session_token(&self) -> &Option<String> {
        &self.session_token
    }

    pub fn is_expired(self) -> bool {
        if let Some(exp) = self.expiration {
            exp < 1000000
        } else {
            false
        }
    }
}
