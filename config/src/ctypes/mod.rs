mod bool;
mod checkable_struct;
mod r#enum;
mod integer;
mod map;
mod path;
mod string;
mod r#struct;
mod vec;
mod wrapper;

pub use crate::ctypes::bool::*;
pub use crate::ctypes::checkable_struct::*;
pub use crate::ctypes::integer::*;
use crate::ctypes::map::CHashMap;
pub use crate::ctypes::map::*;
pub use crate::ctypes::path::*;
pub use crate::ctypes::r#enum::*;
pub use crate::ctypes::r#struct::*;
pub use crate::ctypes::string::*;
pub use crate::ctypes::vec::*;
pub use crate::ctypes::wrapper::*;
use crate::ctypes::CWrapper;
use crate::{ConfigError, InvalidError};
use serde_yaml::Value;
use druid::{Data, Widget, WidgetExt, lens, LensExt};
use druid_enums::Matcher;
use std::iter::FromIterator;
use druid::widget::Label;

#[derive(Debug)]
pub enum State {
    Valid,
    None,
    InValid(String)
}


impl From<InvalidError> for State {
    fn from(err: InvalidError) -> Self {
        State::invalid(err.into_msg())
    }
}

impl From<Result<(), InvalidError>> for State {
    fn from(r: Result<(), InvalidError>) -> Self {
        match r {
            Ok(_) => State::Valid,
            Err(err) => err.into(),
        }
    }
}

impl FromIterator<State> for State {
    fn from_iter<T>(iter: T) -> Self
        where
            T: IntoIterator<Item = State> {
        let mut encountered_none = false;
        for state in iter.into_iter() {
            match state {
                State::InValid(_) => return state,
                State::None => encountered_none = true,
                State::Valid => (),
            }
        }
        if encountered_none {
            State::None
        } else {
            State::Valid
        }
    }
}


impl State {
    pub fn invalid(text: impl Into<String>) -> Self {
        Self::InValid(text.into())
    }
}

#[derive(Debug, Clone, Data, Matcher)]
pub enum CType {
    String(CString),
    Bool(CBool),
    Integer(CInteger),
    Path(CPath),
    CStruct(CStruct),
    CheckableStruct(CCheckableStruct),
    Vec(CVec),
    HashMap(CHashMap),
    CEnum(CEnum),
    Wrapper(Box<CWrapper>),
}

impl CType {
    pub fn is_leaf(&self) -> bool {
        use CType::*;
        match self {
            String(_) | Bool(_) | Integer(_) | Path(_) => true,
            CStruct(_) | CheckableStruct(_) | Vec(_) | HashMap(_) => false,
            Wrapper(cwrapper) => cwrapper.is_leaf(),
            CEnum(cenum) => cenum.is_leaf(),
        }
    }

    pub(crate) fn consume_value(&mut self, value: Value) -> Result<(), ConfigError> {
        match self {
            CType::String(cstring) => match value {
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
            CType::Bool(cbool) => match value {
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
            CType::Path(cpath) => match value {
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
            CType::Integer(cinteger) => match value {
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
            CType::CStruct(cstruct) => match value {
                Value::Mapping(map) => cstruct.consume_map(map),
                _ => Err(InvalidError::new("Expected Mapping").into()),
            },
            CType::CheckableStruct(ccheck_struct) => match value {
                Value::Mapping(map) => ccheck_struct.consume_map(map),
                Value::Null => {
                    ccheck_struct.set_checked(false);
                    Ok(())
                }
                _ => Err(InvalidError::new("Expected Mapping or Null").into()),
            },
            CType::Vec(cvec) => match value {
                Value::Sequence(seq) => cvec.consume_sequence(seq),
                _ => Err(InvalidError::new("Expected Sequence").into()),
            },
            CType::HashMap(chash_map) => match value {
                Value::Mapping(map) => chash_map.consume_map(map),
                _ => Err(InvalidError::new("Expected Mapping").into()),
            },
            CType::Wrapper(cwrapper) => cwrapper.consume_value(value),
            CType::CEnum(cenum) => match value {
                Value::Mapping(map) => cenum.consume_map(map),
                Value::String(str) => match cenum.set_selected(&str) {
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

    pub fn state(&self) -> State {
        use CType::*;
        match self {
            String(cstring) => cstring.state(),
            Bool(cbool) => cbool.state(),
            Integer(cint) => cint.state(),
            Path(cpath) => cpath.state(),
            CStruct(cstruct) => cstruct.state(),
            CheckableStruct(checkable_struct) => checkable_struct.state(),
            Vec(cvec) => cvec.state(),
            HashMap(cmap) => cmap.state(),
            CEnum(cenum) => cenum.state(),
            Wrapper(cwrapper) => cwrapper.state(),
        }
    }

    pub fn widget() -> impl Widget<Self> {
        Self::matcher()
            .string(CString::widget())
            .bool(CBool::widget())
            .integer(CInteger::widget())
            .path(CPath::widget())
            .c_struct(CStruct::widget())
            .checkable_struct(CCheckableStruct::widget())
            .hash_map(CHashMap::widget())
            .vec(CVec::widget())
            .c_enum(CEnum::widget())
            .wrapper(CWrapper::widget().lens(lens::Identity.deref()))
    }
}
