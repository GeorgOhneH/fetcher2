use crate::*;
use serde_yaml::Value;

#[derive(Debug, Clone)]
pub enum CTypes {
    String(CString),
    Bool(CBool),
    Integer(CInteger),
    Struct(CStruct),
    CheckableStruct(CCheckableStruct),
    Vec(CVec),
    Enum(CEnum),
}

impl CTypes {
    pub(crate) fn consume_value(&mut self, value: Value) -> Result<(), RequiredError> {
        match self {
            CTypes::String(cstring) => match value {
                Value::String(str) => {
                    cstring.set(Some(str));
                    Ok(())
                }
                Value::Null => {
                    cstring.set(None);
                    Ok(())
                }
                _ => Err(RequiredError::new("Expected String or Null".to_owned())),
            },
            CTypes::Bool(cbool) => match value {
                Value::Bool(bool) => {
                    cbool.set(Some(bool));
                    Ok(())
                }
                Value::Null => {
                    cbool.set(None);
                    Ok(())
                }
                _ => Err(RequiredError::new("Expected Bool or Null".to_owned())),
            },
            CTypes::Integer(cinteger) => match value {
                Value::Number(num) => match num.as_i64() {
                    Some(int) => cinteger
                        .set(Some(int as isize))
                        .map_err(|_| RequiredError::new("Int not valid".to_owned())),
                    None => Err(RequiredError::new("Not supported Number".to_owned())),
                },
                Value::Null => {
                    cinteger.set(None).unwrap();
                    Ok(())
                }
                _ => Err(RequiredError::new("Expected Number or Null".to_owned())),
            },
            CTypes::Struct(cstruct) => match value {
                Value::Mapping(map) => cstruct.consume_map(map),
                _ => Err(RequiredError::new("Expected Mapping".to_owned())),
            },
            CTypes::CheckableStruct(ccheck_struct) => match value {
                Value::Mapping(map) => ccheck_struct.consume_map(map),
                Value::Null => {
                    ccheck_struct.set_checked(false);
                    Ok(())
                }
                _ => Err(RequiredError::new("Expected Mapping or Null".to_owned())),
            },
            CTypes::Vec(cvec) => match value {
                Value::Sequence(seq) => cvec.consume_sequence(seq),
                _ => Err(RequiredError::new("Expected Sequence".to_owned())),
            },
            CTypes::Enum(cenum) => match value {
                Value::Mapping(map) => cenum.consume_map(map),
                Value::String(str) => match cenum.set_selected(str) {
                    Ok(carg) => {
                        if carg.is_unit() {
                            Ok(())
                        } else {
                            Err(RequiredError::new("Enum must be unit".to_owned()))
                        }
                    }
                    Err(_) => Err(RequiredError::new("Key does not exit".to_owned())),
                },
                Value::Null => {
                    cenum.unselect();
                    Ok(())
                }
                _ => Err(RequiredError::new("Expected Mapping".to_owned())),
            },
        }
    }
}
