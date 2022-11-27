use serde::Deserialize;

use crate::errors::XmlError;

use super::Owner;

#[derive(Debug, Deserialize, PartialEq)]
#[serde(rename_all = "PascalCase")]
pub struct Content {
    key: String,
    last_modified: String,
    #[serde(rename = "ETag")]
    etag: String,
    size: usize,
    storage_class: String,
    owner: Option<Owner>,
}

#[derive(Debug, Deserialize, PartialEq)]
#[serde(rename_all = "PascalCase")]
pub struct CommonPrefix {
    prefix: String,
}

#[derive(Debug, Deserialize, PartialEq)]
#[serde(rename_all = "PascalCase")]
pub struct ListBucketResult {
    name: String,
    prefix: String,
    key_count: usize,
    max_keys: usize,
    delimiter: String,
    is_truncated: bool,
    start_after: Option<String>,
    #[serde(default)]
    contents: Vec<Content>,
    #[serde(default)]
    common_prefixes: Vec<CommonPrefix>,
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
