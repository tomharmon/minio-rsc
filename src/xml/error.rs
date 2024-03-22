use serde::de::Error as DeError;
use serde::ser::Error as SerError;
use std::convert::Infallible;
use std::fmt::Display;

#[derive(Debug)]
pub enum Error {
    UnexpectedToken {
        token: String,
        found: String,
    },
    Custom {
        field: String,
    },
    UnsupportedOperation {
        operation: String,
    },
    Io {
        source: ::std::io::Error,
    },
    FromUtf8Error {
        source: ::std::string::FromUtf8Error,
    },

    ParseIntError {
        source: ::std::num::ParseIntError,
    },

    ParseFloatError {
        source: ::std::num::ParseFloatError,
    },

    ParseBoolError {
        source: ::std::str::ParseBoolError,
    },
    // Syntax {
    //     source: ::xml::reader::Error,
    // },

    // Writer {
    //     source: ::xml::writer::Error,
    // },
}

pub type Result<T> = std::result::Result<T, Error>;

impl std::error::Error for Error {}

#[rustfmt::skip]
impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::UnexpectedToken { token, found } => write!(f, "Expected token {token}, found {found}"),
            Error::Custom { field } => write!(f, "Custom: {field}"),
            Error::UnsupportedOperation { operation } => write!(f, "UnsupportedOperation: {operation}"),
            Error::Io { source } => write!(f, "IO error: {source}"),
            Error::FromUtf8Error { source } => write!(f, "FromUtf8Error: {source}"),
            Error::ParseIntError { source } => write!(f, "ParseIntError: {source}"),
            Error::ParseFloatError { source } => write!(f, "ParseFloatError: {source}"),
            Error::ParseBoolError { source } => write!(f, "ParseBoolError: {source}"),
        }
    }
}

impl DeError for Error {
    fn custom<T: Display>(msg: T) -> Self {
        Error::Custom {
            field: msg.to_string(),
        }
    }
}

impl SerError for Error {
    fn custom<T: Display>(msg: T) -> Self {
        Error::Custom {
            field: msg.to_string(),
        }
    }
}

impl From<::std::io::Error> for Error {
    fn from(source: ::std::io::Error) -> Self {
        Error::Io { source }
    }
}

impl From<::std::string::FromUtf8Error> for Error {
    fn from(source: ::std::string::FromUtf8Error) -> Self {
        Error::FromUtf8Error { source }
    }
}

impl From<::std::num::ParseIntError> for Error {
    fn from(source: ::std::num::ParseIntError) -> Self {
        Error::ParseIntError { source }
    }
}

impl From<::std::num::ParseFloatError> for Error {
    fn from(source: ::std::num::ParseFloatError) -> Self {
        Error::ParseFloatError { source }
    }
}

impl From<::std::str::ParseBoolError> for Error {
    fn from(source: ::std::str::ParseBoolError) -> Self {
        Error::ParseBoolError { source }
    }
}

impl From<Infallible> for Error {
    fn from(err: Infallible) -> Self {
        return Error::Custom {
            field: err.to_string(),
        };
    }
}

impl From<serde_xml_rs::Error> for Error {
    fn from(err: serde_xml_rs::Error) -> Self {
        match err {
            serde_xml_rs::Error::UnexpectedToken { token, found } => {
                Error::UnexpectedToken { token, found }
            }
            serde_xml_rs::Error::Custom { field } => Error::Custom { field },
            serde_xml_rs::Error::UnsupportedOperation { operation } => {
                Error::UnsupportedOperation { operation }
            }
            serde_xml_rs::Error::Io { source } => Error::Io { source },
            serde_xml_rs::Error::FromUtf8Error { source } => Error::FromUtf8Error { source },
            serde_xml_rs::Error::ParseIntError { source } => Error::ParseIntError { source },
            serde_xml_rs::Error::ParseFloatError { source } => Error::ParseFloatError { source },
            serde_xml_rs::Error::ParseBoolError { source } => Error::ParseBoolError { source },
            serde_xml_rs::Error::Syntax { source } => Error::Custom {
                field: format!("Syntax error: {source}"),
            },
            serde_xml_rs::Error::Writer { source } => Error::Custom {
                field: format!("Writer error: {source}"),
            },
        }
    }
}
