use std::path::Path;
use std::pin::Pin;

use bytes::Bytes;
use futures_core::Stream;
use reqwest::Response;

use crate::{
    error::Result,
    types::{
        args::SelectRequest,
        response::{ListBucketResult, SelectObjectReader},
        ObjectStat, Retention, Tags,
    },
    Minio,
};

use super::{BucketArgs, CopySource, KeyArgs, ListObjectsArgs};

/// Instantiate an Bucket which wrap [Minio] and [BucketArgs].
/// Provides operations on objects.
pub struct Bucket {
    pub(super) client: Minio,
    pub(super) bucket: BucketArgs,
}

macro_rules! proxy_object {
    ($name:ident, $reponse:ty) => {
        #[inline]
        pub async fn $name<K>(&self, key: K) -> Result<$reponse>
        where
            K: Into<KeyArgs>,
        {
            self.client.$name(self.bucket.clone(), key).await
        }
    };

    ($name:ident, $reponse:ty, $args:ty) => {
        #[inline]
        pub async fn $name<K>(&self, key: K, args: $args) -> Result<$reponse>
        where
            K: Into<KeyArgs>,
        {
            self.client.$name(self.bucket.clone(), key, args).await
        }
    };
    ($name:ident, $reponse:ty, $args1:ty, $args2:ty) => {
        #[inline]
        pub async fn $name<K>(&self, key: K, args1: $args1, args2: $args2) -> Result<$reponse>
        where
            K: Into<KeyArgs>,
        {
            self.client
                .$name(self.bucket.clone(), key, args1, args2)
                .await
        }
    };
}

macro_rules! proxy_bucket {
    ($name:ident, $reponse:ty) => {
        #[inline]
        pub async fn $name<B>(&self) -> Result<$reponse>
        where
            B: Into<BucketArgs>,
        {
            self.client.$name(self.bucket.clone()).await
        }
    };

    ($name:ident, $reponse:ty, $args:ty) => {
        #[inline]
        pub async fn $name<B>(&self, args: $args) -> Result<$reponse>
        where
            B: Into<BucketArgs>,
        {
            self.client.$name(self.bucket.clone(), args).await
        }
    };
}

type FsStream = Pin<Box<dyn Stream<Item = Result<Bytes>> + Send>>;

impl Bucket {
    proxy_bucket!(list_objects, ListBucketResult, ListObjectsArgs);

    proxy_object!(get_object, Response);
    proxy_object!(put_object, (), Bytes);
    proxy_object!(put_object_stream, (), FsStream, Option<usize>);
    proxy_object!(copy_object, (), CopySource);
    proxy_object!(remove_object, ());
    proxy_object!(stat_object, Option<ObjectStat>);
    proxy_object!(is_object_legal_hold_enabled, bool);
    proxy_object!(enable_object_legal_hold_enabled, ());
    proxy_object!(disable_object_legal_hold_enabled, ());
    proxy_object!(get_object_tags, Tags);
    proxy_object!(set_object_tags, (), Tags);
    proxy_object!(delete_object_tags, ());
    proxy_object!(get_object_retention, Retention);
    proxy_object!(set_object_retention, (), Retention);
    proxy_object!(select_object_content, SelectObjectReader, SelectRequest);

    #[cfg(feature = "fs-tokio")]
    #[inline]
    pub async fn fget_object<K, P>(&self, key: K, path: P) -> Result<()>
    where
        K: Into<KeyArgs>,
        P: AsRef<Path>,
    {
        self.client
            .fget_object(self.bucket.clone(), key, path)
            .await
    }

    #[cfg(feature = "fs-tokio")]
    #[inline]
    pub async fn fput_object<K, P>(&self, key: K, path: P) -> Result<()>
    where
        K: Into<KeyArgs>,
        P: AsRef<Path>,
    {
        self.client
            .fput_object(self.bucket.clone(), key, path)
            .await
    }
}
