use std::path::Path;
use std::pin::Pin;

use crate::errors::Result;
use crate::signer::RECOMMEND_CHUNK_SIZE;
use bytes::Bytes;
use bytes::BytesMut;
use futures::stream::Stream;
use futures_util::pin_mut;
use futures_util::FutureExt;
use std::task::{Context, Poll};
#[cfg(feature = "fs-tokio")]
use tokio::fs::File;
#[cfg(feature = "fs-tokio")]
use tokio::io::AsyncReadExt;
#[cfg(feature = "fs-tokio")]
use tokio::io::AsyncWriteExt;

#[cfg(feature = "fs-tokio")]
pub(crate) struct TokioFileStream {
    file: File,
    len: usize,
}

impl TokioFileStream {
    pub async fn new<P>(path: P) -> Result<Self>
    where
        P: AsRef<Path>,
    {
        let file = tokio::fs::File::open(path).await?;
        let meta = file.metadata().await?;
        let len = meta.len() as usize;
        Ok(Self { file, len })
    }

    pub(crate) fn len(&self) -> usize {
        self.len
    }
}

#[cfg(feature = "fs-tokio")]
impl Stream for TokioFileStream {
    type Item = Result<Bytes>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context) -> Poll<Option<Self::Item>> {
        let mut buf = BytesMut::with_capacity(RECOMMEND_CHUNK_SIZE);
        let rd = self.file.read_buf(&mut buf);
        pin_mut!(rd);
        let s = futures_core::ready!(rd.poll_unpin(cx));
        match s {
            Ok(s) => {
                if s > 0 {
                    Poll::Ready(Some(Ok(buf.freeze())))
                } else {
                    Poll::Ready(None)
                }
            }
            Err(e) => Poll::Ready(Some(Err(e.into()))),
        }
    }
}
