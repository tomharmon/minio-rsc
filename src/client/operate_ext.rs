use core::str;
use std::pin::Pin;

use crate::{datatype::Object, error::Result, Minio};
use async_stream::stream as Stream2;
use futures_core::Stream;
use futures_util::{stream, StreamExt};

use super::{BucketArgs, ListObjectsArgs};

/// Added extension operate.
/// All operations are experimental.
impl Minio {
    /// Reads all objects starting with the prefix of the bucket.
    /// Returns an async stream of [Object]
    /// ## Example
    /// ```rust
    /// # use minio_rsc::Minio;
    /// use futures_util::{stream, StreamExt};
    ///
    /// # async fn example(minio: Minio){
    /// let mut objs = minio.list_objects_stream("bucket".into(), "videos/");
    /// while let Some(obj) = objs.next().await{
    ///  // .....
    /// }
    /// # }
    /// ```
    pub fn list_objects_stream<'a>(
        &'a self,
        bucket: BucketArgs,
        prefix: &'a str,
    ) -> Pin<Box<dyn Stream<Item = Result<Object>> + Send + 'a>> {
        let mut args: Option<ListObjectsArgs> = Some(
            ListObjectsArgs::default()
                .max_keys(1000)
                .prefix(prefix)
                .delimiter(""),
        );
        let stm = Stream2!({
            while let Some(arg) = args.take() {
                let res = self.list_objects(bucket.clone(), arg).await;
                if let Ok(res) = &res {
                    if res.is_truncated {
                        args = Some(
                            ListObjectsArgs::default()
                                .max_keys(1000)
                                .prefix(prefix)
                                .delimiter("")
                                .continuation_token(res.next_continuation_token.as_str()),
                        );
                    }
                }
                yield res
            }
        });
        Box::pin(stm.flat_map(|f| {
            stream::iter(match f {
                Ok(f) => f.contents.into_iter().map(Result::Ok).collect::<Vec<_>>(),
                Err(e) => vec![Err(e)],
            })
        }))
    }
}
