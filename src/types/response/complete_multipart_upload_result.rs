use serde::Deserialize;

use crate::errors::XmlError;

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
    println!("{:?}", result);
    assert!(result.is_ok());
}
