use serde::Deserialize;

use crate::{
    errors::XmlError,
    types::{Initiator, Owner, Part},
};

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct ListPartsResult {
    pub bucket: String,
    pub key: String,
    pub upload_id: String,
    pub part_number_marker: usize,
    pub max_parts: usize,
    pub next_part_number_marker: usize,
    pub is_truncated: bool,
    #[serde(default, rename = "Part")]
    pub parts: Vec<Part>,
    pub storage_class: String,
    pub checksum_algorithm: String,
    pub initiator: Initiator,
    pub owner: Owner,
}

impl TryFrom<&str> for ListPartsResult {
    type Error = XmlError;
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Ok(quick_xml::de::from_str(&value).map_err(|x| Self::Error::from(x))?)
    }
}

#[test]
fn test_list_all_my_buckets_result() {
    let res = "HTTP/1.1 200
    <?xml version=\"1.0\" encoding=\"UTF-8\"?>
    <ListPartsResult>
    <Bucket>string</Bucket>
    <Key>string</Key>
    <UploadId>string</UploadId>
    <PartNumberMarker>1</PartNumberMarker>
    <NextPartNumberMarker>1</NextPartNumberMarker>
    <MaxParts>100</MaxParts>
    <IsTruncated>false</IsTruncated>
    <Part>
       <ChecksumCRC32>string</ChecksumCRC32>
       <ChecksumCRC32C>string</ChecksumCRC32C>
       <ChecksumSHA1>string</ChecksumSHA1>
       <ChecksumSHA256>string</ChecksumSHA256>
       <ETag>string</ETag>
       <LastModified>timestamp</LastModified>
       <PartNumber>1</PartNumber>
       <Size>222</Size>
    </Part>
    <Part>
       <ChecksumCRC32>string</ChecksumCRC32>
       <ChecksumCRC32C>string</ChecksumCRC32C>
       <ChecksumSHA1>string</ChecksumSHA1>
       <ChecksumSHA256>string</ChecksumSHA256>
       <ETag>string</ETag>
       <LastModified>timestamp</LastModified>
       <PartNumber>2</PartNumber>
       <Size>223</Size>
    </Part>
    <Initiator>
       <DisplayName>string</DisplayName>
       <ID>string</ID>
    </Initiator>
    <Owner>
       <DisplayName>string</DisplayName>
       <ID>string</ID>
    </Owner>
    <StorageClass>string</StorageClass>
    <ChecksumAlgorithm>string</ChecksumAlgorithm>
 </ListPartsResult>
    ";
    let result: std::result::Result<ListPartsResult, XmlError> = res.try_into();
    println!("{:?}", result);
    assert!(result.is_ok());
}
