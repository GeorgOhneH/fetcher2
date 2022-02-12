use std::collections::HashMap;
use std::path::PathBuf;
use serde::{Deserialize, Serialize};
use config::ctypes::integer::RangedInt;
use config::ctypes::path::{AnyPath, StrictPath};
use config::deserializer::ConfigDeserializer;
use config::serializer::ConfigSerializer;
use config::traveller::{ConfigTraveller, Travel};

#[test]
fn full() {
    #[derive(Serialize, Deserialize, Debug, Travel, PartialEq)]
    struct TestStruct2(#[travel(default = 9)] pub i64);

    #[derive(Serialize, Deserialize, Debug, Travel, PartialEq)]
    struct TestStruct3(pub i64, pub i64);

    #[derive(Serialize, Deserialize, Debug, Travel, PartialEq)]
    struct TestStruct4;

    #[derive(Serialize, Deserialize, Debug, Travel, PartialEq)]
    struct TestStruct {
        #[travel(default = 9i64)]
        #[travel(name = "hello")]
        pub field1: i64,
        pub field2: bool,
        pub field3: Option<bool>,
        pub field4: TestEnum,
        pub field5: (i64, i64),
        pub field6: [i64; 3],
        pub field7: Vec<i64>,
        pub field8: HashMap<String, i64>,
        pub field9: HashMap<PathBuf, i64>,
        pub field10: String,
        pub field11: PathBuf,
        pub field12: RangedInt<-10, 2>,
        pub field13: StrictPath<AnyPath>,
        pub field14: u64,
        pub field15: TestStruct2,
        pub field16: TestStruct3,
        pub field17: TestStruct4,
        pub field18: (),
    }

    #[derive(Serialize, Deserialize, Debug, Travel, PartialEq)]
    enum TestEnum {
        Unit,
        One(i64),
        Two(i64, i64),
        Three { field0: i64, field1: i64 },
    }

    let mut x = TestStruct::travel(&mut ConfigTraveller::new()).unwrap();
    let s = TestStruct {
        field1: 10,
        field2: true,
        field3: None,
        field4: TestEnum::Three {
            field0: 1,
            field1: 2,
        },
        field5: (1, 2),
        field6: [0; 3],
        field7: vec![9, 3, 6],
        field8: HashMap::from([("hello".to_string(), 10)]),
        field9: HashMap::from([(PathBuf::from("hello2"), 1), (PathBuf::from("he2"), 6)]),
        field10: String::from("hidushfi"),
        field11: PathBuf::from("hidushfi"),
        field12: RangedInt(0),
        field13: StrictPath::from("hello.yml"),
        field14: 19,
        field15: TestStruct2(10),
        field16: TestStruct3(10, 11),
        field17: TestStruct4,
        field18: (),
    };

    s.serialize(&mut ConfigSerializer::new(&mut x)).unwrap();
    let r = TestStruct::deserialize(&mut ConfigDeserializer::new(&x)).unwrap();
    assert_eq!(r, s);
}