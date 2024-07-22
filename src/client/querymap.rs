use crate::utils::{is_urlencoded, urlencode};

#[derive(Default, Clone, Debug)]
pub struct QueryMap(Vec<(String, String)>);

impl QueryMap {
    pub fn new() -> Self {
        Self(Vec::new())
    }

    pub fn from_str(query_str: &str) -> Self {
        let mut qm = Self::new();
        qm.merge_str(query_str);
        qm
    }

    pub fn insert(&mut self, key: String, value: String) {
        self.0.push((key, value))
    }

    pub fn merge(&mut self, querys: Self) {
        self.0.extend(querys.0);
    }

    pub fn merge_str(&mut self, query_str: &str) {
        for query in query_str.split("&").filter(|x| !x.is_empty()) {
            let index = query.find("=");
            if let Some(i) = index {
                self.insert(query[0..i].to_string(), query[i + 1..].to_string());
            } else {
                self.insert(query.to_string(), "".to_string());
            }
        }
    }

    /// sort query by key
    pub fn sort(&mut self) {
        self.0.sort_by(|x, y| x.0.cmp(&y.0));
    }

    /// get query string.
    /// the empty keys will be skipped.
    /// key and value will be uri encode.
    #[inline]
    pub fn to_query_string(self) -> String {
        self.0
            .iter()
            .filter(|(k, _)| !k.is_empty())
            .map(|(k, v)| {
                let k = if !is_urlencoded(k) {
                    urlencode(k, false)
                } else {
                    k.to_owned()
                };
                let v = if !is_urlencoded(v) {
                    urlencode(v, false)
                } else {
                    v.to_owned()
                };
                if v.is_empty() {
                    k
                } else {
                    format!("{k}={v}")
                }
            })
            .collect::<Vec<String>>()
            .join("&")
    }
}

impl Into<String> for QueryMap {
    fn into(self) -> String {
        self.to_query_string()
    }
}
