use config::ctypes::integer::RangedInt;
use config::ctypes::path::{AnyPath, StrictPath};
use config::deserializer::ConfigDeserializer;
use config::serializer::ConfigSerializer;
use config::traveller::{ConfigTraveller, Travel};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

#[test]
fn enum_test() {
    #[derive(Serialize, Deserialize, Debug, Travel, PartialEq)]
    enum TestEnum {
        Unit,
        One(i64),
        Two(i64, i64),
        Three { field0: i64, field1: i64 },
    }

    let mut ty = TestEnum::travel(&mut ConfigTraveller::new()).unwrap();
    let unit = TestEnum::Unit;
    let single = TestEnum::One(10);
    let tuple = TestEnum::Two(5, 7);
    let field = TestEnum::Three {
        field0: 1,
        field1: 2,
    };

    unit.serialize(&mut ConfigSerializer::new(&mut ty)).unwrap();
    let unit_r = TestEnum::deserialize(&mut ConfigDeserializer::new(&ty)).unwrap();
    assert_eq!(unit, unit_r);

    single.serialize(&mut ConfigSerializer::new(&mut ty)).unwrap();
    let single_r = TestEnum::deserialize(&mut ConfigDeserializer::new(&ty)).unwrap();
    assert_eq!(single, single_r);

    tuple.serialize(&mut ConfigSerializer::new(&mut ty)).unwrap();
    let tuple_r = TestEnum::deserialize(&mut ConfigDeserializer::new(&ty)).unwrap();
    assert_eq!(tuple, tuple_r);

    field.serialize(&mut ConfigSerializer::new(&mut ty)).unwrap();
    let field_r = TestEnum::deserialize(&mut ConfigDeserializer::new(&ty)).unwrap();
    assert_eq!(field, field_r);
}
