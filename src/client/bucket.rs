use std::path::Path;
use std::pin::Pin;

use bytes::Bytes;
use futures_core::Stream;
use hyper::Method;
use reqwest::Response;

use super::{ObjectStat, SelectObjectReader};
use super::{
    BucketArgs, CopySource, KeyArgs, ListBucketResult, ListObjectsArgs, ObjectLockConfig, Tags,
};
use crate::datatype::Retention;
use crate::datatype::SelectRequest;
use crate::{error::Result, Minio};

/// Instantiate an Bucket which wrap [Minio] and [BucketArgs].
/// Provides operations on objects.
pub struct Bucket {
    pub(super) client: Minio,
    pub(super) bucket: BucketArgs,
}

macro_rules! proxy_object {
    ($name:ident, $reponse:ty $(,$an:ident=>$at:ty)*) => {
        #[inline]
        pub async fn $name<K>(&self, key: K, $($an:$at),*) -> Result<$reponse>
        where
            K: Into<KeyArgs>,
        {
            self.client
                .$name(self.bucket.clone(), key, $($an),*)
                .await
        }
    };
}

macro_rules! proxy_bucket {
    ($name:ident, $reponse:ty) => {
        #[inline]
        pub async fn $name(&self) -> Result<$reponse> {
            self.client.$name(self.bucket.clone()).await
        }
    };

    ($name:ident, $reponse:ty, $args:ty) => {
        #[inline]
        pub async fn $name(&self, args: $args) -> Result<$reponse> {
            self.client.$name(self.bucket.clone(), args).await
        }
    };
}

type FsStream = Pin<Box<dyn Stream<Item = Result<Bytes>> + Send>>;

impl Bucket {
    #[inline]
    pub fn bucket_args(&self) -> BucketArgs {
        self.bucket.clone()
    }

    /// Check if exists.
    /// If exists and you have permission to access it, return [Ok(true)], otherwise [Ok(false)]
    pub async fn exists(&self) -> Result<bool> {
        let bucket: BucketArgs = self.bucket.clone();
        self.client
            ._bucket_executor(bucket, Method::HEAD)
            .send()
            .await
            .map(|res| res.status().is_success())
    }

    proxy_bucket!(list_objects, ListBucketResult, ListObjectsArgs);
    proxy_bucket!(delete_object_lock_config, ());
    proxy_bucket!(get_object_lock_config, ObjectLockConfig);
    proxy_bucket!(set_object_lock_config, (), ObjectLockConfig);

    proxy_object!(get_object, Response);
    proxy_object!(put_object, (), data=>Bytes);
    proxy_object!(put_object_stream, (), stream=>FsStream, len=>Option<usize>);
    proxy_object!(copy_object, (), cp=> CopySource);
    proxy_object!(remove_object, ());
    proxy_object!(stat_object, Option<ObjectStat>);
    proxy_object!(is_object_legal_hold_enabled, bool);
    proxy_object!(enable_object_legal_hold_enabled, ());
    proxy_object!(disable_object_legal_hold_enabled, ());
    proxy_object!(get_object_tags, Tags);
    proxy_object!(set_object_tags, (), tags=>Tags);
    proxy_object!(delete_object_tags, ());
    proxy_object!(get_object_retention, Retention);
    proxy_object!(set_object_retention, (), retention=>Retention);
    proxy_object!(select_object_content, SelectObjectReader, request=>SelectRequest);

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

impl Into<BucketArgs> for Bucket {
    fn into(self) -> BucketArgs {
        self.bucket
    }
}

impl Into<BucketArgs> for &Bucket {
    fn into(self) -> BucketArgs {
        self.bucket.clone()
    }
}
