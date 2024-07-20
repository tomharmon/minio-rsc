use std::pin::Pin;

use bytes::{Bytes, BytesMut};
use futures_core::Stream;
use futures_util::StreamExt;
use std::result::Result;

use crate::{signer::sha256_hash, utils::EMPTY_CONTENT_SHA256};

/// adding the `x-amz-content-sha256` header with one of the following values
#[allow(unused)]
pub(crate) enum PayloadHash {
    /// This value is the actual checksum of your object and is only possible when you are uploading the data in a single chunk.
    Checksum(String),
    /// Use this when you are uploading the object as a single unsigned chunk.
    Unsigned,
    /// Use this when sending a payload over multiple chunks, and the chunks are signed using `AWS4-HMAC-SHA256`. This produces a SigV4 signature.
    Streaming,
    /// Use this when sending a payload over multiple chunks, and the chunks are signed using `AWS4-HMAC-SHA256`. This produces a SigV4 signature.
    /// In addition, the digest for the chunks is included as a trailing header.
    StreamingTrailer,
    EmptySha256,
}

impl PayloadHash {
    pub fn checksum(hash: String) -> Self {
        Self::Checksum(hash)
    }

    pub fn as_str(&self) -> &str {
        match self {
            PayloadHash::Checksum(v) => v.as_str(),
            PayloadHash::EmptySha256 => EMPTY_CONTENT_SHA256,
            PayloadHash::Unsigned => "UNSIGNED-PAYLOAD",
            PayloadHash::Streaming => "STREAMING-AWS4-HMAC-SHA256-PAYLOAD",
            PayloadHash::StreamingTrailer => "STREAMING-AWS4-HMAC-SHA256-PAYLOAD-TRAILER",
        }
    }
}

/// Payload for http request
pub enum Data<E> {
    /// Transferring Payload in a Single Chunk
    Bytes(Bytes),
    /// Transferring Payload in Multiple Chunks, `usize` the total byte length of the stream.
    Stream(
        Pin<Box<dyn Stream<Item = Result<Bytes, E>> + Sync + Send>>,
        usize,
    ),
}

impl<E> Data<E> {
    /// get an empty Bytes Data.
    #[inline]
    pub fn empty() -> Self {
        Self::Bytes(Bytes::new())
    }

    /// convert Stream Data into Bytes Data
    pub async fn convert(self) -> Result<Self, E> {
        Ok(match self {
            Data::Stream(mut s, l) => {
                let mut buf = BytesMut::with_capacity(l);
                while let Some(data) = s.next().await {
                    buf.extend_from_slice(&data?);
                }
                Data::Bytes(buf.freeze())
            }
            _ => self,
        })
    }

    #[inline]
    pub fn len(&self) -> usize {
        match self {
            Data::Bytes(data) => data.len(),
            Data::Stream(_, len) => len.clone(),
        }
    }

    pub(crate) fn payload_hash(&self) -> PayloadHash {
        match self {
            Data::Stream(_, _) => PayloadHash::Streaming,
            Data::Bytes(data) if data.len() == 0 => PayloadHash::EmptySha256,
            Data::Bytes(data) => PayloadHash::checksum(sha256_hash(data)),
        }
    }
}

impl<E> Default for Data<E> {
    fn default() -> Self {
        Self::empty()
    }
}

impl<E> From<Option<Bytes>> for Data<E> {
    fn from(value: Option<Bytes>) -> Self {
        match value {
            Some(v) => Self::Bytes(v),
            None => Self::empty(),
        }
    }
}

impl<E> From<Bytes> for Data<E> {
    fn from(value: Bytes) -> Self {
        Self::Bytes(value)
    }
}

impl<E> From<String> for Data<E> {
    fn from(value: String) -> Self {
        Self::Bytes(value.into())
    }
}

impl<E> From<&'static str> for Data<E> {
    fn from(value: &'static str) -> Self {
        Self::Bytes(value.into())
    }
}

impl<E> From<Vec<u8>> for Data<E> {
    fn from(value: Vec<u8>) -> Self {
        Self::Bytes(value.into())
    }
}

impl<E>
    From<(
        Pin<Box<dyn Stream<Item = Result<Bytes, E>> + Sync + Send>>,
        usize,
    )> for Data<E>
{
    fn from(
        value: (
            Pin<Box<dyn Stream<Item = Result<Bytes, E>> + Sync + Send>>,
            usize,
        ),
    ) -> Self {
        Self::Stream(value.0, value.1)
    }
}
