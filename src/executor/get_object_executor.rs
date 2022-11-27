use std::path::Path;

use futures::StreamExt;
use hyper::header;
use hyper::HeaderMap;
use hyper::Method;
use reqwest::Response;
use tokio::fs::File;
use tokio::io::AsyncWriteExt;

use crate::client::Minio;
use crate::errors::Result;
use crate::sse::Sse;
use crate::types::QueryMap;

#[derive(Clone)]
pub struct GetObjectExecutor<'a> {
    bucket_name: String,
    object_name: String,
    offset: usize,
    length: usize,
    version_id: Option<String>,
    extra_querys: QueryMap,
    ssec_headers: HeaderMap,
    request_headers: HeaderMap,
    client: &'a Minio,
}

impl<'a> GetObjectExecutor<'a> {
    pub fn new<S1: Into<String>, S2: Into<String>>(
        client: &'a Minio,
        bucket_name: S1,
        object_name: S2,
    ) -> Self {
        return Self {
            bucket_name: bucket_name.into(),
            object_name: object_name.into(),
            offset: 0,
            length: 0,
            version_id: None,
            extra_querys: QueryMap::new(),
            ssec_headers: HeaderMap::new(),
            request_headers: HeaderMap::new(),
            client,
        };
    }

    pub fn offset(mut self, offset: usize) -> Self {
        self.offset = offset;
        self
    }

    pub fn length(mut self, length: usize) -> Self {
        self.length = length;
        self
    }

    pub fn version_id<S: Into<String>>(mut self, version_id: S) -> Self {
        self.version_id = Some(version_id.into());
        self
    }

    pub fn ssec<T>(mut self, ssec: &T) -> Self
    where
        T: Sse,
    {
        self.ssec_headers = ssec.headers();
        self
    }

    pub async fn send(mut self) -> Result<Response> {
        if let Some(version_id) = &self.version_id {
            self.extra_querys.insert("versionId", version_id);
        };
        merge_headermap(&mut self.ssec_headers, self.request_headers);
        if self.offset > 0 || self.length > 0 {
            let ranger = if self.length > 0 {
                format!("bytes={}-{}", self.offset, self.offset + self.length - 1)
            } else {
                format!("bytes={}-", self.offset)
            };
            self.ssec_headers
                .insert(header::RANGE, ranger.parse().unwrap());
        }
        self.client
            ._execute(
                Method::GET,
                self.client.region(),
                Some(self.bucket_name),
                Some(self.object_name),
                None,
                Some(self.ssec_headers),
                Some(self.extra_querys.into()),
            )
            .await
    }

    pub async fn write_to<P>(self, path: P) -> Result<()>
    where
        P: AsRef<Path>,
    {
        let res = self.send().await?;
        if !res.status().is_success(){
            res.text().await.unwrap();
            Ok(())
        }else{
            let mut stream = res.bytes_stream();
            let mut file = File::create(path).await?;
            while let Some(item) = stream.next().await {
                if let Ok(datas) = item {
                    file.write_all(&datas).await?;
                }
            }
            Ok(())
        }
    }
}

fn merge_headermap(header1: &mut HeaderMap, header2: HeaderMap) {
    for (key, val) in header2 {
        if let Some(k) = key {
            header1.insert(k, val);
        }
    }
}
