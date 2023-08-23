use serde::Deserialize;

use crate::{
    errors::XmlError,
    types::{CommonPrefix, Object},
};

#[derive(Debug, Deserialize, PartialEq)]
#[serde(rename_all = "PascalCase")]
pub struct ListBucketResult {
    pub(crate) name: String,
    pub(crate) prefix: String,
    pub(crate) key_count: usize,
    pub(crate) max_keys: usize,
    #[serde(default)]
    pub(crate) delimiter: String,
    pub(crate) is_truncated: bool,
    pub(crate) start_after: Option<String>,
    #[serde(default)]
    pub(crate) contents: Vec<Object>,
    #[serde(default)]
    pub(crate) common_prefixes: Vec<CommonPrefix>,
    #[serde(default)]
    pub(crate) next_continuation_token: String,
    #[serde(default)]
    pub(crate) continuation_token: String,
}

impl ListBucketResult {
    pub fn name(&self) -> &str {
        self.name.as_ref()
    }

    pub fn prefix(&self) -> &str {
        self.prefix.as_ref()
    }

    pub fn key_count(&self) -> usize {
        self.key_count
    }

    pub fn max_keys(&self) -> usize {
        self.max_keys
    }

    pub fn delimiter(&self) -> &str {
        self.delimiter.as_ref()
    }

    pub fn is_truncated(&self) -> bool {
        self.is_truncated
    }

    pub fn start_after(&self) -> Option<&String> {
        self.start_after.as_ref()
    }

    pub fn common_prefixes(&self) -> &[CommonPrefix] {
        self.common_prefixes.as_ref()
    }

    pub fn contents(&self) -> &[Object] {
        self.contents.as_ref()
    }

    pub fn next_continuation_token(&self) -> &str {
        self.next_continuation_token.as_ref()
    }

    pub fn continuation_token(&self) -> &str {
        self.continuation_token.as_ref()
    }
}

impl TryFrom<&str> for ListBucketResult {
    type Error = XmlError;
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        quick_xml::de::from_str(&value).map_err(|x| x.into())
    }
}

#[test]
fn test_list_all_my_buckets_result() {
    let res = r#"
        <ListBucketResult xmlns="http://s3.amazonaws.com/doc/2006-03-01/">
        <Name>example-bucket</Name>
        <Prefix></Prefix>
        <KeyCount>2</KeyCount>
        <MaxKeys>1000</MaxKeys>
        <Delimiter>/</Delimiter>
        <IsTruncated>false</IsTruncated>
        <Contents>
          <Key>sample.jpg</Key>
          <LastModified>2011-02-26T01:56:20.000Z</LastModified>
          <ETag>"bf1d737a4d46a19f3bced6905cc8b902"</ETag>
          <Size>142863</Size>
          <StorageClass>STANDARD</StorageClass>
        </Contents>
        <CommonPrefixes>
          <Prefix>photos/2006/February/</Prefix>
        </CommonPrefixes>
        <CommonPrefixes>
          <Prefix>photos/2006/January/</Prefix>
        </CommonPrefixes>
      </ListBucketResult>	
        "#;
    let result: Result<ListBucketResult, XmlError> = res.try_into();
    println!("{:?}", result);
    assert!(result.is_ok());
}
