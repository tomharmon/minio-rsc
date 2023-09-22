#[cfg(test)]
mod test {

    use serde::Deserialize;

    use crate::{
        datatype::{
            CompleteMultipartUploadResult, CopyPartResult, InitiateMultipartUploadResult,
            LegalHold, ListAllMyBucketsResult, ListBucketResult, ListMultipartUploadsResult,
            ListPartsResult, ObjectLockConfiguration, Retention, Tagging, VersioningConfiguration,
        },
        xml::de::from_str,
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

    test_datatypes!(
        Retention,
        test_retention,
        r#"<Retention><Mode>GOVERNANCE</Mode><RetainUntilDate>2023-09-10T08:16:28.230Z</RetainUntilDate></Retention>"#
    );

    test_datatypes!(
        CopyPartResult,
        test_copy_part_result,
        r#"<CopyPartResult>
        <ETag>string</ETag>
        <LastModified>timestamp</LastModified>
        <ChecksumCRC32>string</ChecksumCRC32>
        <ChecksumCRC32C>string</ChecksumCRC32C>
        <ChecksumSHA1>string</ChecksumSHA1>
        <ChecksumSHA256>string</ChecksumSHA256>
        </CopyPartResult>"#
    );

    test_datatypes!(
        ObjectLockConfiguration,
        test_object_lock_configure,
        r#"
        <ObjectLockConfiguration>
            <ObjectLockEnabled>Enabled</ObjectLockEnabled>
            <Rule>
                <DefaultRetention>
                    <Days>112</Days>
                    <Mode>GOVERNANCE</Mode>
                    <Years>1221</Years>
                </DefaultRetention>
            </Rule>
        </ObjectLockConfiguration>
        "#
    );

    test_datatypes!(
        CompleteMultipartUploadResult,
        test_complete_multipart_upload_result,
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?>
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
        "
    );

    test_datatypes!(
        InitiateMultipartUploadResult,
        test_initiate_multipart_upload_result,
        "
        <?xml version=\"1.0\" encoding=\"UTF-8\"?>
        <InitiateMultipartUploadResult xmlns=\"http://s3.amazonaws.com/doc/2006-03-01/\">
        <Bucket>file</Bucket><Key>test.txt</Key>
        <UploadId>b3621cce-9a4c-4c0e-8666-c701b8255163</UploadId>
        </InitiateMultipartUploadResult>
        "
    );

    test_datatypes!(
        ListMultipartUploadsResult,
        test_list_multipart_uploads_result,
        "
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
        "
    );

    test_datatypes!(
        ListPartsResult,
        test_list_parts_result,
        "
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
        "
    );

    test_datatypes!(
        ListAllMyBucketsResult,
        test_list_all_my_buckets_result,
        "
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
        </ListAllMyBucketsResult>"
    );

    test_datatypes!(
        VersioningConfiguration,
        test_versioning_configuration,
        r#"<?xml version="1.0" encoding="UTF-8"?>
        <VersioningConfiguration>
            <Status>Enabled</Status>
            <MfaDelete>Enabled</MfaDelete>
        </VersioningConfiguration>"#
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

        // let now = std::time::SystemTime::now();
        // for i in 0..10000 {
        //     from_str::<Test>(j).unwrap();
        // }
        // let s = std::time::SystemTime::now()
        //     .duration_since(now)
        //     .unwrap()
        //     .as_nanos();
        // println!("{s}");
    }
}
