//! Error and Result module.
use core::fmt;
use hyper::{
    header::{InvalidHeaderName, InvalidHeaderValue},
    Error as RequestError,
};
use serde::Deserialize;
use std::{error::Error as StdError, convert::Infallible};
use std::{fmt::Display, result};

/// A `Result` typedef to use with the `minio-rsc::error` type
pub type Result<T> = result::Result<T, Error>;

/// [MinioError]
pub type Error = MinioError;

// impl fmt::Debug for Error {
//     fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
//         f.debug_tuple("minio-rsc::error")
//             // Skip the noise of the ErrorKind enum
//             .field(&self.inner)
//             .finish()
//     }
// }

// impl fmt::Display for Error {
//     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
//         fmt::Display::fmt(&self.inner, f)
//     }
// }

/// inducate an illegal variable was used.
#[derive(Debug)]
pub struct ValueError(String);

impl ValueError {
    pub fn new<T: Into<String>>(value: T) -> Self {
        Self(value.into())
    }
}

impl Display for ValueError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "value error: {}", self.0)
    }
}

impl StdError for ValueError {}

impl From<&str> for ValueError {
    fn from(err: &str) -> Self {
        Self(err.to_string())
    }
}

impl From<InvalidHeaderValue> for ValueError {
    fn from(err: InvalidHeaderValue) -> Self {
        return ValueError(err.to_string());
    }
}

impl From<InvalidHeaderName> for ValueError {
    fn from(err: InvalidHeaderName) -> Self {
        return ValueError(err.to_string());
    }
}

impl From<Infallible> for ValueError {
    fn from(err: Infallible) -> Self {
        return ValueError(err.to_string());
    }
}

/// XML parsing error.
#[derive(Debug)]
pub struct XmlError(String);

impl Display for XmlError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "xmlerror: {}", self.0)
    }
}

impl StdError for XmlError {}

impl From<quick_xml::DeError> for XmlError {
    fn from(err: quick_xml::DeError) -> Self {
        Self(err.to_string())
    }
}

impl From<quick_xml::Error> for XmlError {
    fn from(err: quick_xml::Error) -> Self {
        Self(err.to_string())
    }
}

/// S3 service returned error response.
///
#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct S3Error {
    pub code: String,
    pub message: String,
    #[serde(default)]
    pub resource: String,
    pub request_id: String,
    pub host_id: Option<String>,
    pub bucket_name: Option<String>,
    pub object_name: Option<String>,
}

impl std::fmt::Display for S3Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "S3Error: {}", self.message)
    }
}

impl StdError for S3Error {}

impl TryFrom<&[u8]> for S3Error {
    type Error = XmlError;
    fn try_from(res: &[u8]) -> std::result::Result<Self, Self::Error> {
        return Ok(quick_xml::de::from_reader(res)?);
    }
}

impl TryFrom<&str> for S3Error {
    type Error = XmlError;

    fn try_from(value: &str) -> std::result::Result<Self, Self::Error> {
        value.as_bytes().try_into()
    }
}

/// InternalException - thrown to indicate internal library error.
/// ErrorResponseException - thrown to indicate S3 service returned an error response.
/// thrown to indicate I/O error on S3 operation.
/// ServerException Thrown to indicate that S3 service returning HTTP server error.
#[derive(Debug)]
pub enum MinioError {
    /// inducate an illegal variable was used.
    ValueError(String),

    /// indicate conncet to S3 service failed.
    RequestError(RequestError),

    /// indicate XML parsing error.
    XmlError(XmlError),

    /// indicate S3 service returned error response.
    S3Error(S3Error),

    /// indicate S3 service returned invalid or no error response.
    HttpError,

    /// indicate I/O error, had on S3 operation.
    IoError(String),
}

impl StdError for MinioError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            MinioError::RequestError(e) => e.source(),
            MinioError::S3Error(e) => e.source(),
            _ => None,
        }
    }
}

impl fmt::Display for MinioError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self {
            MinioError::ValueError(e) => write!(f, "{}", e),
            MinioError::RequestError(e) => write!(f, "{}", e),
            MinioError::XmlError(e) => write!(f, "{}", e),
            MinioError::S3Error(e) => write!(f, "{}", e),
            MinioError::HttpError => write!(
                f,
                "HttpError, S3 service returned invalid or no error response."
            ),
            MinioError::IoError(e) => write!(f, "{}", e),
        }
    }
}

impl From<S3Error> for MinioError {
    fn from(err: S3Error) -> Self {
        MinioError::S3Error(err)
    }
}

// impl From<MinioError> for Error {
//     fn from(err: MinioError) -> Self {
//         Self { inner: err }
//     }
// }

impl<T: Into<ValueError>> From<T> for MinioError {
    fn from(err: T) -> Self {
        MinioError::ValueError(err.into().0)
    }
}

impl From<XmlError> for MinioError {
    fn from(err: XmlError) -> Self {
        MinioError::XmlError(err)
    }
}

impl From<RequestError> for Error {
    fn from(err: RequestError) -> Self {
        MinioError::RequestError(err)
    }
}

#[cfg(feature = "fs-tokio")]
impl From<tokio::io::Error> for Error {
    fn from(err: tokio::io::Error) -> Self {
        MinioError::IoError(err.to_string())
    }
}

impl From<reqwest::Error> for Error {
    fn from(err: reqwest::Error) -> Self {
        if err.is_builder() {
            return Self::ValueError(err.to_string());
        }
        Self::HttpError
    }
}

#[cfg(test)]
mod tests {
    use super::S3Error;
    use crate::errors::{Result, XmlError};

    #[test]
    fn test_s3_error() {
        let res = r#"<?xml version="1.0" encoding="UTF-8"?>
        <Error>
            <Code>NoSuchKey</Code>
            <Message>The resource you requested does not exist</Message>
            <Resource>/mybucket/myfoto.jpg</Resource>
            <RequestId>4442587FB7D0A2F9</RequestId>
        </Error>"#;
        let result: std::result::Result<S3Error, XmlError> = res.as_bytes().try_into();
        assert!(result.is_ok());
        println!("{:?}", result);
    }
}
