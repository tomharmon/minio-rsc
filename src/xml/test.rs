#[cfg(test)]
mod test {

    use serde::Deserialize;

    use crate::datatype::{
        AccessControlPolicy, CompleteMultipartUploadResult, CopyPartResult,
        InitiateMultipartUploadResult, LegalHold, ListAllMyBucketsResult, ListBucketResult,
        ListMultipartUploadsResult, ListPartsResult, ListVersionsResult, ObjectLockConfiguration,
        Retention, Tagging, VersioningConfiguration,
    };

    macro_rules! test_datatypes {
        ($ty:ty,$name:ident,$txt:expr) => {
            #[test]
            fn $name() {
                let txt = $txt.trim_start();
                let res = crate::xml::de::from_str::<$ty>(txt).unwrap();
                println!("{}", crate::xml::ser::to_string(&res).unwrap());
            }
        };
    }

    test_datatypes!(
        AccessControlPolicy,
        test_access_control_policy,
        r#"
        <AccessControlPolicy>
            <Owner>
                <ID>75aa57f09aa0c8caeab4f8c24e99d10f8e7faeebf76c078efc7c6caea54ba06a</ID>
                <DisplayName>mtd@amazon.com</DisplayName>
            </Owner>
            <AccessControlList>
                <Grant>
                    <Grantee xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance">
                        <ID>75aa57f09aa0c8caeab4f8c24e99d10f8e7faeebf76c078efc7c6caea54ba06a</ID>
                        <DisplayName>mtd@amazon.com</DisplayName>
                        <Type>CanonicalUser</Type>
                    </Grantee>
                    <Permission>FULL_CONTROL</Permission>
                </Grant>
                <Grant>
                    <Grantee xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance" xsi:type="CanonicalUser">
                        <ID>75aa57f09aa0c8caeab4f8c24e99d10f8e7faeebf76c078efc7c6caea54ba06a</ID>
                        <DisplayName>mtd@amazon.com</DisplayName>
                    </Grantee>
                    <Permission>FULL_CONTROL</Permission>
                </Grant>
            </AccessControlList>
        </AccessControlPolicy>
        "#
    );

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
        r#"<?xml version="1.0" encoding="UTF-8"?>
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
        "#
    );

    test_datatypes!(
        InitiateMultipartUploadResult,
        test_initiate_multipart_upload_result,
        r#"
        <?xml version="1.0" encoding="UTF-8"?>
        <InitiateMultipartUploadResult xmlns="http://s3.amazonaws.com/doc/2006-03-01/">
        <Bucket>file</Bucket><Key>test.txt</Key>
        <UploadId>b3621cce-9a4c-4c0e-8666-c701b8255163</UploadId>
        </InitiateMultipartUploadResult>
        "#
    );

    test_datatypes!(
        ListMultipartUploadsResult,
        test_list_multipart_uploads_result,
        r#"
        <?xml version="1.0" encoding="UTF-8"?>
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
        "#
    );

    test_datatypes!(
        ListPartsResult,
        test_list_parts_result,
        r#"
        <?xml version="1.0" encoding="UTF-8"?>
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
        "#
    );

    test_datatypes!(
        ListAllMyBucketsResult,
        test_list_all_my_buckets_result,
        r#"
        <?xml version="1.0" encoding="UTF-8"?>
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
        </ListAllMyBucketsResult>"#
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

    test_datatypes!(
        ListVersionsResult,
        tet_list_object_versions,
        r#"
        <?xml version="1.0" encoding="UTF-8"?>
        <ListVersionsResult xmlns="http://s3.amazonaws.com/doc/2006-03-01">
            <Name>bucket</Name>
            <Prefix>my</Prefix>
            <KeyMarker/>
            <VersionIdMarker/>
            <MaxKeys>5</MaxKeys>
            <IsTruncated>false</IsTruncated>
            <Version>
                <Key>my-image.jpg</Key>
                <VersionId>3/L4kqtJl40Nr8X8gdRQBpUMLUo</VersionId>
                <IsLatest>true</IsLatest>
                <LastModified>2009-10-12T17:50:30.000Z</LastModified>
                <ETag>"fba9dede5f27731c9771645a39863328"</ETag>
                <Size>434234</Size>
                <StorageClass>STANDARD</StorageClass>
                <Owner>
                    <ID>75aa57f09aa0c8caeab4f8c24e99d10f8e7faeebf76c078efc7c6caea54ba06a</ID>
                    <DisplayName>mtd@amazon.com</DisplayName>
                </Owner>
            </Version>
            <DeleteMarker>
                <Key>my-second-image.jpg</Key>
                <VersionId>03jpff543dhffds434rfdsFDN943fdsFkdmqnh892</VersionId>
                <IsLatest>true</IsLatest>
                <LastModified>2009-11-12T17:50:30.000Z</LastModified>
                <Owner>
                    <ID>75aa57f09aa0c8caeab4f8c24e99d10f8e7faeebf76c078efc7c6caea54ba06a</ID>
                    <DisplayName>mtd@amazon.com</DisplayName>
                </Owner>
            </DeleteMarker>
            <Version>
                <Key>my-second-image.jpg</Key>
                <VersionId>QUpfdndhfd8438MNFDN93jdnJFkdmqnh893</VersionId>
                <IsLatest>false</IsLatest>
                <LastModified>2009-10-10T17:50:30.000Z</LastModified>
                <ETag>"9b2cf535f27731c974343645a3985328"</ETag>
                <Size>166434</Size>
                <StorageClass>STANDARD</StorageClass>
                <Owner>
                    <ID>75aa57f09aa0c8caeab4f8c24e99d10f8e7faeebf76c078efc7c6caea54ba06a</ID>
                    <DisplayName>mtd@amazon.com</DisplayName>
                </Owner>
            </Version>
            <DeleteMarker>
                <Key>my-third-image.jpg</Key>
                <VersionId>03jpff543dhffds434rfdsFDN943fdsFkdmqnh892</VersionId>
                <IsLatest>true</IsLatest>
                <LastModified>2009-10-15T17:50:30.000Z</LastModified>
                <Owner>
                    <ID>75aa57f09aa0c8caeab4f8c24e99d10f8e7faeebf76c078efc7c6caea54ba06a</ID>
                    <DisplayName>mtd@amazon.com</DisplayName>
                </Owner>
            </DeleteMarker>
            <Version>
                <Key>my-third-image.jpg</Key>
                <VersionId>UIORUnfndfhnw89493jJFJ</VersionId>
                <IsLatest>false</IsLatest>
                <LastModified>2009-10-11T12:50:30.000Z</LastModified>
                <ETag>"772cf535f27731c974343645a3985328"</ETag>
                <Size>64</Size>
                <StorageClass>STANDARD</StorageClass>
                <Owner>
                    <ID>75aa57f09aa0c8caeab4f8c24e99d10f8e7faeebf76c078efc7c6caea54ba06a</ID>
                    <DisplayName>mtd@amazon.com</DisplayName>
                </Owner>
            </Version>
        </ListVersionsResult>
        "#
    );

    #[test]
    fn test_struct() {
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

        #[derive(Deserialize, PartialEq, Debug)]
        enum Mode {
            Off,
            On,
        }

        #[derive(Deserialize, PartialEq, Debug)]
        struct Test {
            aaa: Vec<Abc>,
            name: Option<String>,
            first: Vec<u32>,
            mode: Mode,
        }

        let j = r#"<?xml version="1.0" encoding="UTF-8"?>
        <Test><name>332</name>
        <mode>Off</mode>
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
        let s = crate::xml::de::from_str::<Test>(j);
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
