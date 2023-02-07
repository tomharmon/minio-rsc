use serde::Deserialize;

use crate::errors::XmlError;

use super::Owner;

#[derive(Debug, Deserialize, PartialEq)]
#[serde(rename_all = "PascalCase")]
pub struct Content {
    pub(crate) key: String,
    pub(crate) last_modified: String,
    #[serde(rename = "ETag")]
    pub(crate) etag: String,
    pub(crate) size: usize,
    pub(crate) storage_class: String,
    pub(crate) owner: Option<Owner>,
}

#[derive(Debug, Deserialize, PartialEq)]
#[serde(rename_all = "PascalCase")]
pub struct CommonPrefix {
    pub(crate) prefix: String,
}

#[derive(Debug, Deserialize, PartialEq)]
#[serde(rename_all = "PascalCase")]
pub struct ListBucketResult {
    pub(crate) name: String,
    pub(crate) prefix: String,
    pub(crate) key_count: usize,
    pub(crate) max_keys: usize,
    pub(crate) delimiter: String,
    pub(crate) is_truncated: bool,
    pub(crate) start_after: Option<String>,
    #[serde(default)]
    pub(crate) contents: Vec<Content>,
    #[serde(default)]
    pub(crate) common_prefixes: Vec<CommonPrefix>,
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
