use serde::Deserialize;

use crate::{
    errors::XmlError,
    types::{CommonPrefix, Object},
};

#[derive(Debug, Deserialize, PartialEq)]
#[serde(rename_all = "PascalCase")]
pub struct ListBucketResult {
    pub name: String,
    pub prefix: String,
    pub key_count: usize,
    pub max_keys: usize,
    #[serde(default)]
    pub delimiter: String,
    pub is_truncated: bool,
    pub start_after: Option<String>,
    #[serde(default)]
    pub contents: Vec<Object>,
    #[serde(default)]
    pub common_prefixes: Vec<CommonPrefix>,
    #[serde(default)]
    pub next_continuation_token: String,
    #[serde(default)]
    pub continuation_token: String,
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
