use serde::Deserialize;

use crate::{
    errors::XmlError,
    types::{CommonPrefix, MultipartUpload},
};

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
    println!("{:?}", result);
    assert!(result.is_ok());
}
