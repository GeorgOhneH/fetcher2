use std::iter::FromIterator;

use druid::{Data, lens, LensExt, Widget, WidgetExt};
use druid_enums::Matcher;

pub use crate::ctypes::bool::*;
pub use crate::ctypes::checkable_struct::*;
use crate::ctypes::CWrapper;
pub use crate::ctypes::integer::*;
pub use crate::ctypes::float::*;
pub use crate::ctypes::map::*;
use crate::ctypes::map::CHashMap;
pub use crate::ctypes::path::*;
pub use crate::ctypes::r#enum::*;
pub use crate::ctypes::r#struct::*;
pub use crate::ctypes::string::*;
pub use crate::ctypes::vec::*;
pub use crate::ctypes::wrapper::*;
use crate::InvalidError;

mod bool;
mod checkable_struct;
mod r#enum;
mod integer;
mod map;
mod path;
mod string;
mod r#struct;
mod vec;
mod float;
mod wrapper;

#[derive(Debug, PartialEq)]
pub enum State {
    Valid,
    None,
    InValid(String),
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
        T: IntoIterator<Item = State>,
    {
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
    Float(CFloat),
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
            String(_) | Bool(_) | Integer(_) | Float(_) | Path(_) => true,
            CStruct(_) | CheckableStruct(_) | Vec(_) | HashMap(_) => false,
            Wrapper(cwrapper) => cwrapper.is_leaf(),
            CEnum(cenum) => cenum.is_leaf(),
        }
    }

    pub fn string_mut(&mut self) -> Option<&mut CString> {
        match self {
            Self::String(cstring) => Some(cstring),
            _ => None,
        }
    }

    pub fn int_mut(&mut self) -> Option<&mut CInteger> {
        match self {
            Self::Integer(cint) => Some(cint),
            _ => None,
        }
    }

    pub fn float_mut(&mut self) -> Option<&mut CFloat> {
        match self {
            Self::Float(cfloat) => Some(cfloat),
            _ => None,
        }
    }

    pub fn bool_mut(&mut self) -> Option<&mut CBool> {
        match self {
            Self::Bool(v) => Some(v),
            _ => None,
        }
    }

    pub fn path_mut(&mut self) -> Option<&mut CPath> {
        match self {
            Self::Path(v) => Some(v),
            _ => None,
        }
    }

    pub fn struct_mut(&mut self) -> Option<&mut CStruct> {
        match self {
            Self::CStruct(v) => Some(v),
            _ => None,
        }
    }

    pub fn check_struct_mut(&mut self) -> Option<&mut CCheckableStruct> {
        match self {
            Self::CheckableStruct(v) => Some(v),
            _ => None,
        }
    }

    pub fn vec_mut(&mut self) -> Option<&mut CVec> {
        match self {
            Self::Vec(v) => Some(v),
            _ => None,
        }
    }

    pub fn map_mut(&mut self) -> Option<&mut CHashMap> {
        match self {
            Self::HashMap(v) => Some(v),
            _ => None,
        }
    }

    pub fn enum_mut(&mut self) -> Option<&mut CEnum> {
        match self {
            Self::CEnum(v) => Some(v),
            _ => None,
        }
    }

    pub fn wrapper_mut(&mut self) -> Option<&mut CWrapper> {
        match self {
            Self::Wrapper(v) => Some(v),
            _ => None,
        }
    }

    pub fn state(&self) -> State {
        use CType::*;
        match self {
            String(cstring) => cstring.state(),
            Bool(cbool) => cbool.state(),
            Integer(cint) => cint.state(),
            Float(cfloat) => cfloat.state(),
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
            .float(CFloat::widget())
            .path(CPath::widget())
            .c_struct(CStruct::widget())
            .checkable_struct(CCheckableStruct::widget())
            .hash_map(CHashMap::widget())
            .vec(CVec::widget())
            .c_enum(CEnum::widget())
            .wrapper(CWrapper::widget().lens(lens::Identity.deref()))
    }
}
