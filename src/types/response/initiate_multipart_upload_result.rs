use serde::Deserialize;

use crate::errors::XmlError;

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
    let res = "HTTP/1.1 200
    <?xml version=\"1.0\" encoding=\"UTF-8\"?>
    <InitiateMultipartUploadResult xmlns=\"http://s3.amazonaws.com/doc/2006-03-01/\">
    <Bucket>file</Bucket><Key>test.txt</Key>
    <UploadId>b3621cce-9a4c-4c0e-8666-c701b8255163</UploadId>
    </InitiateMultipartUploadResult>
    ";
    let result: std::result::Result<InitiateMultipartUploadResult, XmlError> = res.try_into();
    println!("{:?}", result);
    assert!(result.is_ok());
}
