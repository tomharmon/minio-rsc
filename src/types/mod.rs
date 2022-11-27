pub mod args;
pub mod response;

use crate::{errors::XmlError, utils::urlencode};

#[derive(Default, Clone, Debug)]
pub struct QueryMap(Vec<(String, String)>);

impl QueryMap {
    pub fn new() -> Self {
        Self(Vec::new())
    }

    pub fn from_str(query_str: &str) -> Self {
        let mut qm = Self::new();
        for s in query_str.split("&").filter(|x| !x.is_empty()) {
            let index = s.find("=");
            if let Some(i) = index {
                qm.insert(&s[0..i], &s[i + 1..]);
            } else {
                qm.insert(s, "");
            }
        }
        qm
    }

    pub fn insert<K: Into<String>, V: Into<String>>(&mut self, key: K, value: V) {
        self.0.push((key.into(), value.into()))
    }

    /// sort query by key
    pub fn sort(&mut self) {
        self.0.sort_by(|x, y| x.0.cmp(&y.0));
    }

    /// get query string.
    /// the empty keys will be skipped.
    /// key and value will be uri encode.
    pub fn to_query_string(self) -> String {
        self.0
            .iter()
            .filter(|(k, _)| !k.is_empty())
            .map(|(k, v)| format!("{}={}", urlencode(k), urlencode(v)))
            .collect::<Vec<String>>()
            .join("&")
    }
}

impl Into<String> for QueryMap {
    fn into(self) -> String {
        self.to_query_string()
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Region(pub String);

impl Region {
    pub fn from<S>(region: S) -> Self
    where
        S: Into<String>,
    {
        return Self(region.into());
    }

    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
}

impl TryFrom<&[u8]> for Region {
    type Error = XmlError;

    fn try_from(res: &[u8]) -> Result<Self, Self::Error> {
        let mut reader = quick_xml::Reader::from_reader(res);
        reader.trim_text(true);
        let mut location = None;
        loop {
            match reader.read_event() {
                Ok(quick_xml::events::Event::Start(ref e)) => {
                    if e.name().as_ref() == b"LocationConstraint" {
                        location = Some(reader.read_text(e.to_end().name())?.into_owned());
                    }
                }
                Err(e) => Err(e)?,
                Ok(quick_xml::events::Event::Eof) => break,
                _ => {}
            }
        }
        return Ok(Region(if let Some(s) = location {
            if s.is_empty() {
                "us-east-1".to_string()
            } else {
                s
            }
        } else {
            "us-east-1".to_string()
        }));
    }
}

impl TryFrom<&str> for Region {
    type Error = XmlError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        value.as_bytes().try_into()
    }
}
