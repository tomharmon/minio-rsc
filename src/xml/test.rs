#[cfg(test)]
mod test {

    use serde::{Deserialize, Serialize};

    use crate::{
        datatype::{LegalHold, ListBucketResult, Tagging},
        xml::{de::from_str, ser::to_string},
    };

    macro_rules! test_datatypes {
        ($ty:ty,$name:ident,$txt:expr) => {
            #[test]
            fn $name() {
                let txt = $txt;
                let res = crate::xml::de::from_str::<$ty>(txt).unwrap();
                println!("{}", crate::xml::ser::to_string(&res).unwrap());
            }
        };
    }

    test_datatypes!(
        ListBucketResult,
        test_list_bucket_result,
        r#"
        <ListBucketResult xmlns="http://s3.amazonaws.com/doc/2006-03-01/">
        <Name>example-bucket</Name>
        <Prefix></Prefix>
        <KeyCount>2</KeyCount>
        <MaxKeys>1000</MaxKeys>
        <Delimiter>/</Delimiter>
        <IsTruncated>false</IsTruncated>
        <Contents>
            <Key>sample.jpg</Key>
            <LastModified>2011-02-26T01:56:20.000Z</LastModified>
            <ETag>"bf1d737a4d46a19f3bced6905cc8b902"</ETag>
            <Size>142863</Size>
            <StorageClass>STANDARD</StorageClass>
        </Contents>
        <CommonPrefixes>
            <Prefix>photos/2006/February/</Prefix>
        </CommonPrefixes>
        <CommonPrefixes>
            <Prefix>photos/2006/January/</Prefix>
        </CommonPrefixes>
        </ListBucketResult>
        "#
    );

    test_datatypes!(
        Tagging,
        test_tagging,
        r#"
        <?xml version="1.0" encoding="UTF-8"?>
        <Tagging>
            <TagSet>
                <Tag>
                    <Key>string</Key>
                    <Value>string</Value>
                </Tag>
                <Tag>
                    <Key>string2</Key>
                    <Value>string</Value>
                </Tag>
            </TagSet>
        </Tagging>
        "#
    );

    test_datatypes!(
        LegalHold,
        test_legal_hold,
        r#"<?xml version="1.0" encoding="UTF-8"?>
        <LegalHold>
            <Status>OFF</Status>
        </LegalHold>"#
    );

    #[test]
    fn test_struct() {
        let j = r#"<Test><nme><Abc><first>323</first></Abc></name></Test>0"#;
        #[derive(Deserialize, PartialEq, Debug)]
        struct Abc2 {
            first: u32,
            // seq: Vec<String>,
        }
        #[derive(Deserialize, PartialEq, Debug)]
        struct Abc {
            first: u32,
            second: Abc2,
            // seq: Vec<String>,
        }

        // #[derive(Deserialize, PartialEq, Debug)]
        // enum Mode {
        //     Off,
        //     On,
        // }

        #[derive(Deserialize, PartialEq, Debug)]
        struct Test {
            aaa: Vec<Abc>,
            name: Option<String>,
            first: Vec<u32>,
            // seq: Vec<String>,
        }

        let j = r#"
        <?xml version=\"1.0\" encoding=\"UTF-8\"?>
        <Test><name>332</name>
        <dd><dd2>dd</dd2></dd>
        <aaa><first>2121</first><second><first>2121</first></second></aaa>
        <!--ssdf-->
        <aaa><first>2121</first><second><first>2121</first></second></aaa>
        <aaa><first>2121</first><second><first>2121</first></second></aaa>
        <aaa><first>2121</first><second><first>2121</first></second></aaa>
        <aaa><first>2121</first><second><first>2121</first></second></aaa>
        <aaa><first>2121</first><second><first>2121</first></second></aaa>
        <aaa><first>2121</first><second><first>2121</first></second></aaa>
        <aaa><first>2121</first><second><first>2121</first></second></aaa>
        <aaa><first>2121</first><second><first>2121</first></second></aaa>
        <aaa><first>2121</first><second><first>2121</first></second></aaa><first>2</first><first>332</first></Test>"#;
        let s = from_str::<Test>(j);
        println!("{s:?}");

        let now = std::time::SystemTime::now();
        for i in 0..10000 {
            from_str::<Test>(j).unwrap();
        }
        let s = std::time::SystemTime::now()
            .duration_since(now)
            .unwrap()
            .as_nanos();
        println!("{s}");

        let now = std::time::SystemTime::now();
        for i in 0..10000 {
            quick_xml::de::from_str::<Test>(&j).unwrap();
        }
        let s = std::time::SystemTime::now()
            .duration_since(now)
            .unwrap()
            .as_nanos();
        println!("{s}");
    }
}
