use hyper::HeaderMap;

use super::super::QueryMap;

use super::BaseArgs;
pub struct ListMultipartUploadsArgs {
    bucket_name: String,
    delimiter: String,
    encoding_type: String,
    key_marker: Option<String>,
    max_uploads: usize,
    prefix: String,
    upload_id_marker: Option<String>,
    extra_headers: Option<HeaderMap>,
    extra_query_params: Option<String>,
    expected_bucket_owner: Option<String>,
}

impl ListMultipartUploadsArgs {
    pub fn new(bucket_name: String) -> Self {
        Self {
            bucket_name,
            delimiter: "".to_string(),
            encoding_type: "".to_string(),
            max_uploads: 1000,
            prefix: "".to_string(),
            key_marker: None,
            upload_id_marker: None,
            expected_bucket_owner: None,
            extra_query_params: None,
            extra_headers: None,
        }
    }

    pub fn bucket_name(&self) -> &str {
        &self.bucket_name
    }

    pub fn delimiter<T: Into<String>>(mut self, delimiter: T) -> Self {
        self.delimiter = delimiter.into();
        self
    }

    pub fn encoding_type<T: Into<String>>(mut self, encoding_type: T) -> Self {
        self.encoding_type = encoding_type.into();
        self
    }

    pub fn key_marker<T: Into<String>>(mut self, key_marker: T) -> Self {
        self.key_marker = Some(key_marker.into());
        self
    }

    pub fn upload_id_marker<T: Into<String>>(mut self, upload_id_marker: T) -> Self {
        self.upload_id_marker = Some(upload_id_marker.into());
        self
    }

    pub fn max_uploads(mut self, max_uploads: usize) -> Self {
        self.max_uploads = max_uploads;
        if self.max_uploads > 1000 {
            self.max_uploads = 1000;
        }
        self
    }

    pub fn prefix<T: Into<String>>(mut self, prefix: T) -> Self {
        self.prefix = prefix.into();
        self
    }

    pub fn expected_bucket_owner<T: Into<String>>(mut self, expected_bucket_owner: T) -> Self {
        self.expected_bucket_owner = Some(expected_bucket_owner.into());
        self
    }
}

impl BaseArgs for ListMultipartUploadsArgs {
    fn extra_query_map(&self) -> QueryMap {
        let mut querys: QueryMap = QueryMap::default();
        querys.insert("uploads".to_string(), "".to_string());
        querys.insert("delimiter".to_string(), self.delimiter.to_string());
        querys.insert("max-uploads".to_string(), self.max_uploads.to_string());
        querys.insert("prefix".to_string(), self.prefix.to_string());
        querys.insert("encoding-type".to_string(), self.encoding_type.to_string());
        if let Some(encoding_type) = &self.key_marker {
            querys.insert("key-marker".to_string(), encoding_type.to_string());
        }
        if let Some(delimiter) = &self.upload_id_marker {
            querys.insert("upload-id-marker".to_string(), delimiter.clone());
        }
        return querys;
    }

    fn extra_headers(&self) -> HeaderMap {
        let mut headermap = HeaderMap::new();
        if let Some(owner) = &self.expected_bucket_owner {
            if let Ok(val) = owner.parse() {
                headermap.insert("x-amz-expected-bucket-owner", val);
            }
        }
        headermap
    }
}
