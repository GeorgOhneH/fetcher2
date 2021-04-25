use crate::*;
use serde_yaml::Value;

#[derive(Debug, Clone)]
pub enum CTypes {
    String(CString),
    Bool(CBool),
    Integer(CInteger),
    Path(CPath),
    Struct(CStruct),
    CheckableStruct(CCheckableStruct),
    Vec(CVec),
    Enum(CEnum),
}

impl CTypes {
    pub(crate) fn consume_value(&mut self, value: Value) -> Result<(), ConfigError> {
        match self {
            CTypes::String(cstring) => match value {
                Value::String(str) => {
                    cstring.set(str);
                    Ok(())
                }
                Value::Null => {
                    cstring.unset();
                    Ok(())
                }
                _ => Err(InvalidError::new("Expected String or Null").into()),
            },
            CTypes::Bool(cbool) => match value {
                Value::Bool(bool) => {
                    cbool.set(bool);
                    Ok(())
                }
                Value::Null => {
                    cbool.unset();
                    Ok(())
                }
                _ => Err(InvalidError::new("Expected Bool or Null").into()),
            },
            CTypes::Path(cpath) => match value {
                Value::String(str) => {
                    cpath.set(str)?;
                    Ok(())
                }
                Value::Null => {
                    cpath.unset();
                    Ok(())
                }
                _ => Err(InvalidError::new("Expected Bool or Null").into()),
            },
            CTypes::Integer(cinteger) => match value {
                Value::Number(num) => match num.as_i64() {
                    Some(int) => cinteger.set(int as isize).map_err(|e| e.into()),
                    None => Err(InvalidError::new("Not supported Number").into()),
                },
                Value::Null => {
                    cinteger.unset();
                    Ok(())
                }
                _ => Err(InvalidError::new("Expected Number or Null").into()),
            },
            CTypes::Struct(cstruct) => match value {
                Value::Mapping(map) => cstruct.consume_map(map),
                _ => Err(InvalidError::new("Expected Mapping").into()),
            },
            CTypes::CheckableStruct(ccheck_struct) => match value {
                Value::Mapping(map) => ccheck_struct.consume_map(map),
                Value::Null => {
                    ccheck_struct.set_checked(false);
                    Ok(())
                }
                _ => Err(InvalidError::new("Expected Mapping or Null").into()),
            },
            CTypes::Vec(cvec) => match value {
                Value::Sequence(seq) => cvec.consume_sequence(seq),
                _ => Err(InvalidError::new("Expected Sequence").into()),
            },
            CTypes::Enum(cenum) => match value {
                Value::Mapping(map) => cenum.consume_map(map),
                Value::String(str) => match cenum.set_selected(str) {
                    Ok(carg) => {
                        if carg.is_unit() {
                            Ok(())
                        } else {
                            Err(InvalidError::new("Enum must be unit").into())
                        }
                    }
                    Err(_) => Err(InvalidError::new("Key does not exit").into()),
                },
                Value::Null => {
                    cenum.unselect();
                    Ok(())
                }
                _ => Err(InvalidError::new("Expected Mapping").into()),
            },
        }
    }
}
