use quick_xml::events::Event;

use crate::errors::XmlError;

use super::Owner;

#[derive(Clone, Debug)]
pub struct Bucket {
    pub name: String,
    pub creation_date: String,
}

#[derive(Clone, Debug)]
pub struct ListAllMyBucketsResult {
    pub buckets: Vec<Bucket>,
    pub owner: Owner,
}

impl TryFrom<&[u8]> for ListAllMyBucketsResult {
    type Error = XmlError;
    fn try_from(res: &[u8]) -> Result<Self, Self::Error> {
        let mut reader = quick_xml::Reader::from_reader(res);
        reader.trim_text(true);
        let mut buckets: Vec<Bucket> = Vec::new();
        let mut name = "".to_string();
        let mut creation_date = "".to_string();
        let mut id = "".to_string();
        let mut display_name = "".to_string();
        loop {
            match reader.read_event() {
                Ok(Event::Start(ref e)) => {
                    match e.name().as_ref() {
                        b"Buckets" => buckets.clear(),
                        b"Name" => {
                            name = reader.read_text(e.to_end().name())?.into_owned();
                        }
                        b"CreationDate" => {
                            creation_date = reader.read_text(e.to_end().name())?.into_owned();
                        }
                        b"DisplayName" => {
                            display_name = reader.read_text(e.to_end().name())?.into_owned();
                        }
                        b"ID" => {
                            id = reader.read_text(e.to_end().name())?.into_owned();
                        }
                        _ => {}
                    };
                }
                Ok(Event::End(e)) => match e.name().as_ref() {
                    b"Bucket" => {
                        buckets.push(Bucket {
                            name: name.to_owned(),
                            creation_date: creation_date.to_owned(),
                        });
                    }
                    _ => {}
                },
                Err(e) => Err(e)?,
                Ok(Event::Eof) => break,
                _ => (),
            }
        }
        Ok(Self {
            owner: Owner { id, display_name },
            buckets,
        })
    }
}

impl TryFrom<&str> for ListAllMyBucketsResult {
    type Error = XmlError;
    fn try_from(res: &str) -> Result<Self, Self::Error> {
        res.as_bytes().try_into()
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
    let result: std::result::Result<ListAllMyBucketsResult, XmlError> = res.as_bytes().try_into();
    println!("{:?}", result);
    assert!(result.is_ok());
}
