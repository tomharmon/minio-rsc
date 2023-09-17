use serde::Deserialize;

use crate::error::XmlError;

use super::{CommonPrefix, Initiator, MultipartUpload, Owner, Part};

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct CompleteMultipartUploadResult {
    bucket: String,
    key: String,
    e_tag: String,
    location: String,
}

impl CompleteMultipartUploadResult {
    pub fn bucket(&self) -> &str {
        self.bucket.as_ref()
    }

    pub fn key(&self) -> &str {
        self.key.as_ref()
    }

    pub fn e_tag(&self) -> &str {
        self.e_tag.as_ref()
    }

    pub fn location(&self) -> &str {
        self.location.as_ref()
    }
}

impl TryFrom<&str> for CompleteMultipartUploadResult {
    type Error = XmlError;
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Ok(quick_xml::de::from_str(&value).map_err(|x| Self::Error::from(x))?)
    }
}

#[test]
fn test_complete_multipart_upload_result() {
    let res = "
    <?xml version=\"1.0\" encoding=\"UTF-8\"?>
    <CompleteMultipartUploadResult>
        <Location>string</Location>
        <Bucket>string</Bucket>
        <Key>string</Key>
        <ETag>string</ETag>
        <ChecksumCRC32>string</ChecksumCRC32>
        <ChecksumCRC32C>string</ChecksumCRC32C>
        <ChecksumSHA1>string</ChecksumSHA1>
        <ChecksumSHA256>string</ChecksumSHA256>
    </CompleteMultipartUploadResult>
    ";
    let result: std::result::Result<CompleteMultipartUploadResult, XmlError> = res.try_into();
    assert!(result.is_ok());
}

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
    assert!(result.is_ok());
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct InitiateMultipartUploadResult {
    pub bucket: String,
    pub key: String,
    pub upload_id: String,
}

impl TryFrom<&str> for InitiateMultipartUploadResult {
    type Error = XmlError;
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Ok(quick_xml::de::from_str(&value).map_err(|x| Self::Error::from(x))?)
    }
}

#[test]
fn test_list_all_my_buckets_result() {
    let res = "
    <?xml version=\"1.0\" encoding=\"UTF-8\"?>
    <InitiateMultipartUploadResult xmlns=\"http://s3.amazonaws.com/doc/2006-03-01/\">
    <Bucket>file</Bucket><Key>test.txt</Key>
    <UploadId>b3621cce-9a4c-4c0e-8666-c701b8255163</UploadId>
    </InitiateMultipartUploadResult>
    ";
    let result: std::result::Result<InitiateMultipartUploadResult, XmlError> = res.try_into();
    assert!(result.is_ok());
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct ListMultipartUploadsResult {
    pub bucket: String,
    pub key_marker: String,
    pub upload_id_marker: String,
    pub next_key_marker: String,
    pub prefix: String,
    pub delimiter: String,
    pub next_upload_id_marker: String,
    pub max_uploads: usize,
    pub is_truncated: bool,
    #[serde(default, rename = "Upload")]
    pub uploads: Vec<MultipartUpload>,
    #[serde(default)]
    pub common_prefixes: Vec<CommonPrefix>,
    pub encoding_type: Option<String>,
}

impl TryFrom<&str> for ListMultipartUploadsResult {
    type Error = XmlError;
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Ok(quick_xml::de::from_str(&value).map_err(|x| Self::Error::from(x))?)
    }
}

#[test]
fn test_list_multipart_uploads_result() {
    let res = "
    <?xml version=\"1.0\" encoding=\"UTF-8\"?>
    <ListMultipartUploadsResult>
    <Bucket>string</Bucket>
    <KeyMarker>string</KeyMarker>
    <UploadIdMarker>string</UploadIdMarker>
    <NextKeyMarker>string</NextKeyMarker>
    <Prefix>string</Prefix>
    <Delimiter>string</Delimiter>
    <NextUploadIdMarker>string</NextUploadIdMarker>
    <MaxUploads>1000</MaxUploads>
    <IsTruncated>false</IsTruncated>
    <Upload>
        <ChecksumAlgorithm>string</ChecksumAlgorithm>
        <Initiated>timestamp</Initiated>
        <Initiator>
            <DisplayName>string</DisplayName>
            <ID>string</ID>
        </Initiator>
        <Key>string</Key>
        <Owner>
            <DisplayName>string</DisplayName>
            <ID>string</ID>
        </Owner>
        <StorageClass>string</StorageClass>
        <UploadId>string</UploadId>
    </Upload>
    <Upload>
        <ChecksumAlgorithm>string</ChecksumAlgorithm>
        <Initiated>timestamp</Initiated>
        <Initiator>
            <DisplayName>string</DisplayName>
            <ID>string</ID>
        </Initiator>
        <Key>string</Key>
        <Owner>
            <DisplayName>string</DisplayName>
            <ID>string</ID>
        </Owner>
        <StorageClass>string</StorageClass>
        <UploadId>string</UploadId>
    </Upload>
    <CommonPrefixes>
        <Prefix>string</Prefix>
    </CommonPrefixes>
    <CommonPrefixes>
        <Prefix>string</Prefix>
    </CommonPrefixes>
    <EncodingType>string</EncodingType>
    </ListMultipartUploadsResult>
    ";
    let result: std::result::Result<ListMultipartUploadsResult, XmlError> = res.try_into();
    assert!(result.is_ok());
}

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
fn test_list_parts_result() {
    let res = "
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
    assert!(result.is_ok());
}
