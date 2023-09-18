use serde::Deserialize;

use crate::{
    error::XmlError,
    datatype::{Bucket, Owner},
};

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct Buckets {
    #[serde(default)]
    bucket: Vec<Bucket>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct ListAllMyBucketsResult {
    #[serde(default)]
    buckets: Buckets,
    owner: Owner,
}

impl ListAllMyBucketsResult {
    pub fn owner(&self) -> &Owner {
        &self.owner
    }

    pub fn buckets(&self) -> &Vec<Bucket> {
        &self.buckets.bucket
    }

    pub fn into_part(self) -> (Vec<Bucket>, Owner) {
        (self.buckets.bucket, self.owner)
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
    let res = "
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
