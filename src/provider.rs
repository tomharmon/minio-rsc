use futures::Future;
use std::{env, pin::Pin};

use crate::Credentials;

type CredenticalFuture = Pin<Box<dyn Future<Output = Credentials>>>;
pub trait Provider {
    fn fetct(&mut self) -> CredenticalFuture;
}

#[derive(Clone)]
pub struct StaticProvider(Credentials);

impl StaticProvider {
    pub fn new<T: Into<String>>(ak: T, sk: T, st: Option<String>) -> Self {
        Self(Credentials::new(ak, sk, st, None))
    }

    /// load Credentials from env  
    /// - MINIO_ACCESS_KEY  
    /// - MINIO_SECRET_KEY
    /// - MINIO_SESSION_TOKEN
    pub fn from_env() -> Option<Self> {
        if let (Ok(ak), Ok(sk), st) = (
            env::var("MINIO_ACCESS_KEY"),
            env::var("MINIO_SECRET_KEY"),
            env::var("MINIO_SESSION_TOKEN"),
        ) {
            Some(Self::new(ak, sk, st.ok()))
        } else {
            None
        }
    }
}

impl Provider for StaticProvider {
    fn fetct(&mut self) -> CredenticalFuture {
        let cred = self.0.clone();
        Box::pin(async move { cred })
    }
}
