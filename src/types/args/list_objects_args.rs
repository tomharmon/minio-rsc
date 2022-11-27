use super::super::QueryMap;

use super::BaseArgs;
pub struct ListObjectsArgs {
    continuation_token: Option<String>,
    delimiter: Option<String>,
    encoding_type: Option<String>,
    fetch_owner: bool,
    start_after: Option<String>,
    max_keys: usize,
    prefix: Option<String>,
    expected_bucket_owner: Option<String>,
    request_payer: Option<String>,
}

impl ListObjectsArgs {
    pub fn default() -> Self {
        Self {
            continuation_token: None,
            delimiter: None,
            fetch_owner: false,
            max_keys: 1000,
            prefix: None,
            start_after: None,
            encoding_type: None,
            expected_bucket_owner: None,
            request_payer: None,
        }
    }

    pub fn continuation_token<T: Into<String>>(mut self, token: T) -> Self {
        self.continuation_token = Some(token.into());
        self
    }

    pub fn delimiter<T: Into<String>>(mut self, delimiter: T) -> Self {
        self.delimiter = Some(delimiter.into());
        self
    }

    pub fn encoding_type<T: Into<String>>(mut self, encoding_type: T) -> Self {
        self.encoding_type = Some(encoding_type.into());
        self
    }

    pub fn fetch_owner(mut self, fetch_owner: bool) -> Self {
        self.fetch_owner = fetch_owner;
        self
    }

    pub fn start_after<T: Into<String>>(mut self, start_after: T) -> Self {
        self.start_after = Some(start_after.into());
        self
    }

    pub fn max_keys(mut self, max_keys: usize) -> Self {
        self.max_keys = max_keys;
        if self.max_keys > 1000 {
            self.max_keys = 1000;
        }
        self
    }

    pub fn prefix<T: Into<String>>(mut self, prefix: T) -> Self {
        self.prefix = Some(prefix.into());
        self
    }

    pub fn expected_bucket_owner<T: Into<String>>(mut self, expected_bucket_owner: T) -> Self {
        self.expected_bucket_owner = Some(expected_bucket_owner.into());
        self
    }

    pub fn request_payer<T: Into<String>>(mut self, request_payer: T) -> Self {
        self.request_payer = Some(request_payer.into());
        self
    }
}

impl BaseArgs for ListObjectsArgs {
    fn extra_query_map(&self) -> QueryMap {
        let mut querys: QueryMap = QueryMap::default();
        querys.insert("list-type", "2");

        if let Some(encoding_type) = &self.encoding_type {
            querys.insert("encoding-type", encoding_type);
        }
        if let Some(delimiter) = &self.delimiter {
            querys.insert("delimiter".to_string(), delimiter.clone());
        }
        if let Some(token) = &self.continuation_token {
            querys.insert("continuation-token".to_string(), token.clone());
        }
        if self.fetch_owner {
            querys.insert("fetch-owner".to_string(), "true".to_string());
        }
        if let Some(prefix) = &self.prefix {
            querys.insert("prefix".to_string(), prefix.clone());
        }
        if let Some(start_after) = &self.start_after {
            querys.insert("start-after".to_string(), start_after.clone());
        }
        querys.insert("max-keys".to_string(), format!("{}", self.max_keys));

        return querys;
    }
}
