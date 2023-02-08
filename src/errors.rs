use core::fmt;
use hyper::{header::InvalidHeaderValue, Error as RequestError};
use std::{fmt::Display, result};

/// A `Result` typedef to use with the `minio-rsc::error` type
pub type Result<T> = result::Result<T, Error>;

// pub struct Error {
//     inner: MinioError,
// }

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
    fn new<T: Into<String>>(value: T) -> Self {
        Self(value.into())
    }
}

impl<T> From<T> for ValueError
where
    T: Display,
{
    fn from(err: T) -> Self {
        Self(err.to_string())
    }
}

/// XML parsing error.
#[derive(Debug)]
pub struct XmlError(String);

impl<T> From<T> for XmlError
where
    T: Display,
{
    fn from(err: T) -> Self {
        Self(err.to_string())
    }
}

/// S3 service returned error response.
///
#[derive(Debug)]
pub struct S3Error {
    pub code: String,
    pub message: String,
    pub resource: String,
    pub request_id: String,
    pub host_id: Option<String>,
    pub bucket_name: Option<String>,
    pub object_name: Option<String>,
}

impl TryFrom<&[u8]> for S3Error {
    type Error = XmlError;
    fn try_from(res: &[u8]) -> std::result::Result<Self, Self::Error> {
        let mut reader = quick_xml::Reader::from_reader(res);
        reader.trim_text(true);
        let mut code: Option<String> = None;
        let mut message: Option<String> = None;
        let mut resource: Option<String> = None;
        let mut request_id: Option<String> = None;
        let mut host_id: Option<String> = None;
        let mut bucket_name: Option<String> = None;
        let mut object_name: Option<String> = None;
        loop {
            match reader.read_event() {
                Ok(quick_xml::events::Event::Start(ref e)) => {
                    match e.name().as_ref() {
                        b"Error" => {}
                        b"Code" => {
                            code = Some(reader.read_text(e.to_end().name())?.into_owned());
                        }
                        b"Message" => {
                            message = Some(reader.read_text(e.to_end().name())?.into_owned());
                        }
                        b"Resource" => {
                            resource = Some(reader.read_text(e.to_end().name())?.into_owned());
                        }
                        b"RequestId" => {
                            request_id = Some(reader.read_text(e.to_end().name())?.into_owned());
                        }
                        b"BucketName" => {
                            bucket_name = Some(reader.read_text(e.to_end().name())?.into_owned());
                        }
                        b"ObjectName" => {
                            object_name = Some(reader.read_text(e.to_end().name())?.into_owned());
                        }
                        b"HostId" => {
                            host_id = Some(reader.read_text(e.to_end().name())?.into_owned());
                        }

                        _ => {}
                    };
                }
                Err(e) => Err(e)?,
                Ok(quick_xml::events::Event::Eof) => break,
                _ => (),
            }
        }
        if let (Some(code), Some(message), Some(resource), Some(request_id)) =
            (code, message, resource, request_id)
        {
            Ok(Self {
                code,
                message,
                resource,
                request_id,
                host_id,
                bucket_name,
                object_name,
            })
        } else {
            Err(XmlError("Invalid error response".to_string()))
        }
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

impl fmt::Display for MinioError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // fmt::Display::fmt(&self.inner, f)
        write!(f, "ww")
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

impl From<ValueError> for MinioError {
    fn from(err: ValueError) -> Self {
        MinioError::ValueError(err.0)
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

impl From<InvalidHeaderValue> for Error {
    fn from(err: InvalidHeaderValue) -> Self {
        return Self::ValueError(err.to_string());
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
