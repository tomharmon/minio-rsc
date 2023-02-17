use serde::Deserialize;

use crate::{
    errors::XmlError,
    types::{Bucket, Owner},
};

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub(crate) struct Buckets {
    #[serde(default)]
    pub(crate) bucket: Vec<Bucket>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct ListAllMyBucketsResult {
    #[serde(default)]
    pub(crate) buckets: Buckets,
    pub(crate) owner: Owner,
}

impl ListAllMyBucketsResult {
    pub fn owner(&self) -> &Owner {
        &self.owner
    }

    pub fn buckets(&self) -> &Vec<Bucket> {
        &self.buckets.bucket
    }
}

impl TryFrom<&str> for ListAllMyBucketsResult {
    type Error = XmlError;
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Ok(quick_xml::de::from_str(&value).map_err(|x| Self::Error::from(x))?)
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
