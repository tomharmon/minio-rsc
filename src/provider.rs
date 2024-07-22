//! Credential provider
use futures::Future;
use std::{env, pin::Pin};

use crate::Credentials;

pub type CredentialFuture = Pin<Box<dyn Future<Output = Credentials> + Send>>;

/// define Credential retriever.
pub trait Provider: Send + Sync {
    fn fetch(&self) -> CredentialFuture;
}

#[derive(Debug, Clone)]
pub struct StaticProvider(Credentials);

impl StaticProvider {
    pub fn new<T: Into<String>>(ak: T, sk: T, st: Option<String>) -> Self {
        Self(Credentials::new(ak, sk, st, None))
    }

    /// load Credentials from MinIO environment variables.
    /// - `MINIO_ACCESS_KEY`
    /// - `MINIO_SECRET_KEY`
    /// - `MINIO_SESSION_TOKEN`
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

    /// load Credentials from AWS environment variables.
    /// - `AWS_ACCESS_KEY_ID` or `AWS_ACCESS_KEY`
    /// - `AWS_SECRET_ACCESS_KEY` or `AWS_SECRET_KEY`
    /// - `AWS_SESSION_TOKEN`
    pub fn from_env_aws() -> Option<Self> {
        let ak = env::var("AWS_ACCESS_KEY_ID");
        let ak = if !ak.is_ok() {
            env::var("AWS_ACCESS_KEY")
        } else {
            ak
        };
        let sk = env::var("AWS_ACCESS_KEY_ID");
        let sk = if !sk.is_ok() {
            env::var("AWS_ACCESS_KEY")
        } else {
            sk
        };
        if let (Ok(ak), Ok(sk), st) = (ak, sk, env::var("MINIO_SESSION_TOKEN")) {
            Some(Self::new(ak, sk, st.ok()))
        } else {
            None
        }
    }
}

impl Provider for StaticProvider {
    fn fetch(&self) -> CredentialFuture {
        let cred = self.0.clone();
        Box::pin(async move { cred })
    }
}
