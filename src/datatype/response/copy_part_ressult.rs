use serde::Deserialize;

use crate::error::XmlError;

#[derive(Debug, Deserialize, PartialEq)]
#[serde(rename_all = "PascalCase")]
pub struct CopyPartResult {
    pub e_tag: String,
}

impl TryFrom<&str> for CopyPartResult {
    type Error = XmlError;
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        quick_xml::de::from_str(&value).map_err(|x| x.into())
    }
}

#[test]
fn test_copy_part_result() {
    let res = r#"
    <CopyPartResult>
    <ETag>string</ETag>
    <LastModified>timestamp</LastModified>
    <ChecksumCRC32>string</ChecksumCRC32>
    <ChecksumCRC32C>string</ChecksumCRC32C>
    <ChecksumSHA1>string</ChecksumSHA1>
    <ChecksumSHA256>string</ChecksumSHA256>
    </CopyPartResult>
        "#;
    let result: Result<CopyPartResult, XmlError> = res.try_into();
    println!("{:?}", result);
    assert!(result.is_ok());
}
