use serde::Deserialize;

use crate::errors::XmlError;

use super::Owner;

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Bucket {
    pub name: String,
    pub creation_date: String,
}

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct Buckets {
    bucket: Vec<Bucket>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct _ListAllMyBucketsResult {
    #[serde(default)]
    buckets: Buckets,
    owner: Owner,
}

#[derive(Clone, Debug)]
pub struct ListAllMyBucketsResult {
    pub buckets: Vec<Bucket>,
    pub owner: Owner,
}

impl TryFrom<&str> for ListAllMyBucketsResult {
    type Error = XmlError;
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let inner: _ListAllMyBucketsResult =
            quick_xml::de::from_str(&value).map_err(|x| Self::Error::from(x))?;
        Ok(ListAllMyBucketsResult {
            buckets: inner.buckets.bucket,
            owner: inner.owner,
        })
    }
}

#[test]
fn test_list_all_my_buckets_result() {
    let res = "HTTP/1.1 200
        <?xml version=\"1.0\" encoding=\"UTF-8\"?>
        <ListAllMyBucketsResult>
           <Buckets>
              <Bucket>
                 <CreationDate>timestamp</CreationDate>
                 <Name>string</Name>
              </Bucket>
              <Bucket>
                 <CreationDate>timestamp2</CreationDate>
                 <Name>string2</Name>
              </Bucket>
           </Buckets>
           <Owner>
              <DisplayName>string</DisplayName>
              <ID>string</ID>
           </Owner>
        </ListAllMyBucketsResult>";
    let result: std::result::Result<ListAllMyBucketsResult, XmlError> = res.try_into();
    println!("{:?}", result);
    assert!(result.is_ok());
}
