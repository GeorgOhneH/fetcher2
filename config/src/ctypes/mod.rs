use std::iter::FromIterator;

use druid::{lens, Data, LensExt, Widget, WidgetExt};
use druid_enums::Matcher;
use crate::ctypes::bool::CBool;
use crate::ctypes::cenum::CEnum;
use crate::ctypes::cstruct::CStruct;
use crate::ctypes::float::CFloat;
use crate::ctypes::integer::CInteger;
use crate::ctypes::map::CHashMap;
use crate::ctypes::option::COption;
use crate::ctypes::path::CPath;
use crate::ctypes::string::CString;
use crate::ctypes::tuple::CTuple;
use crate::ctypes::vec::CVec;
use crate::errors::Error;

pub mod bool;
pub mod cenum;
pub mod float;
pub mod integer;
pub mod map;
pub mod path;
pub mod string;
pub mod cstruct;
pub mod vec;
pub mod option;
pub mod tuple;


#[derive(Debug, Clone, Data, Matcher)]
pub enum CType {
    String(CString),
    Bool(CBool),
    Integer(CInteger),
    Float(CFloat),
    Path(CPath),
    Tuple(CTuple),
    CStruct(CStruct),
    Vec(CVec),
    HashMap(CHashMap),
    CEnum(CEnum),
    Option(Box<COption>),
}

impl CType {
    pub fn is_leaf(&self) -> bool {
        use CType::*;
        match self {
            String(_) | Bool(_) | Integer(_) | Float(_) | Path(_) => true,
            CStruct(_)  | Vec(_) | HashMap(_) => false,
            CEnum(cenum) => cenum.is_leaf(),
            Option(coption) => coption.ty.is_leaf(),
            Tuple(ctuple) => todo!(),
        }
    }

    pub fn as_string(&self) -> Result<&CString, Error> {
        match self {
            Self::String(cstring) => Ok(cstring),
            _ => Err(Error::ExpectedString),
        }
    }

    pub fn as_string_mut(&mut self) -> Result<&mut CString, Error> {
        match self {
            Self::String(cstring) => Ok(cstring),
            _ => Err(Error::ExpectedString),
        }
    }

    pub fn as_int(&self) -> Result<&CInteger, Error> {
        match self {
            Self::Integer(cvalue) => Ok(cvalue),
            _ => Err(Error::ExpectedInteger),
        }
    }

    pub fn as_int_mut(&mut self) -> Result<&mut CInteger, Error> {
        match self {
            Self::Integer(cvalue) => Ok(cvalue),
            _ => Err(Error::ExpectedInteger),
        }
    }

    pub fn as_float(&self) -> Result<&CFloat, Error> {
        match self {
            Self::Float(cvalue) => Ok(cvalue),
            _ => Err(Error::ExpectedFloat),
        }
    }

    pub fn as_float_mut(&mut self) -> Result<&mut CFloat, Error> {
        match self {
            Self::Float(cvalue) => Ok(cvalue),
            _ => Err(Error::ExpectedFloat),
        }
    }

    pub fn as_bool(&self) -> Result<&CBool, Error> {
        match self {
            Self::Bool(cvalue) => Ok(cvalue),
            _ => Err(Error::ExpectedBoolean),
        }
    }

    pub fn as_bool_mut(&mut self) -> Result<&mut CBool, Error> {
        match self {
            Self::Bool(cvalue) => Ok(cvalue),
            _ => Err(Error::ExpectedBoolean),
        }
    }

    pub fn as_path(&self) -> Result<&CPath, Error> {
        match self {
            Self::Path(cvalue) => Ok(cvalue),
            _ => Err(Error::ExpectedPath),
        }
    }

    pub fn as_path_mut(&mut self) -> Result<&mut CPath, Error> {
        match self {
            Self::Path(cvalue) => Ok(cvalue),
            _ => Err(Error::ExpectedPath),
        }
    }

    pub fn as_option(&self) -> Result<&COption, Error> {
        match self {
            Self::Option(cvalue) => Ok(cvalue),
            _ => Err(Error::ExpectedOption),
        }
    }

    pub fn as_option_mut(&mut self) -> Result<&mut COption, Error> {
        match self {
            Self::Option(cvalue) => Ok(cvalue),
            _ => Err(Error::ExpectedOption),
        }
    }

    pub fn as_struct(&self) -> Result<&CStruct, Error> {
        match self {
            Self::CStruct(cvalue) => Ok(cvalue),
            _ => Err(Error::ExpectedStruct),
        }
    }

    pub fn as_struct_mut(&mut self) -> Result<&mut CStruct, Error> {
        match self {
            Self::CStruct(cvalue) => Ok(cvalue),
            _ => Err(Error::ExpectedStruct),
        }
    }

    pub fn as_tuple(&self) -> Result<&CTuple, Error> {
        match self {
            Self::Tuple(cvalue) => Ok(cvalue),
            _ => Err(Error::ExpectedTuple),
        }
    }

    pub fn as_tuple_mut(&mut self) -> Result<&mut CTuple, Error> {
        match self {
            Self::Tuple(cvalue) => Ok(cvalue),
            _ => Err(Error::ExpectedTuple),
        }
    }

    pub fn as_vec(&self) -> Result<&CVec, Error> {
        match self {
            Self::Vec(cvalue) => Ok(cvalue),
            _ => Err(Error::ExpectedVec),
        }
    }

    pub fn as_vec_mut(&mut self) -> Result<&mut CVec, Error> {
        match self {
            Self::Vec(cvalue) => Ok(cvalue),
            _ => Err(Error::ExpectedVec),
        }
    }

    pub fn as_map(&self) -> Result<&CHashMap, Error> {
        match self {
            Self::HashMap(cvalue) => Ok(cvalue),
            _ => Err(Error::ExpectedMap),
        }
    }

    pub fn as_map_mut(&mut self) -> Result<&mut CHashMap, Error> {
        match self {
            Self::HashMap(cvalue) => Ok(cvalue),
            _ => Err(Error::ExpectedMap),
        }
    }

    pub fn as_enum(&self) -> Result<&CEnum, Error> {
        match self {
            Self::CEnum(cvalue) => Ok(cvalue),
            _ => Err(Error::ExpectedEnum),
        }
    }

    pub fn as_enum_mut(&mut self) -> Result<&mut CEnum, Error> {
        match self {
            Self::CEnum(cvalue) => Ok(cvalue),
            _ => Err(Error::ExpectedEnum),
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
            .hash_map(CHashMap::widget())
            .vec(CVec::widget())
            .c_enum(CEnum::widget())
            .option(COption::widget().lens(lens::Identity.deref()))
    }
}
