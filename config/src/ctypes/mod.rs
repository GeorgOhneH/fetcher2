use crate::ctypes::bool::CBool;
use crate::ctypes::cenum::CEnum;
use crate::ctypes::cstruct::CStruct;
use crate::ctypes::float::CFloat;
use crate::ctypes::integer::CInteger;
use crate::ctypes::map::CMap;
use crate::ctypes::option::COption;
use crate::ctypes::path::CPath;
use crate::ctypes::seq::CSeq;
use crate::ctypes::string::CString;
use crate::ctypes::tuple::CTuple;
use crate::ctypes::unit::CUnit;
use crate::errors::{Error, InValid};

pub mod bool;
pub mod cenum;
pub mod cstruct;
pub mod float;
pub mod integer;
pub mod map;
pub mod option;
pub mod path;
pub mod seq;
pub mod string;
pub mod tuple;
pub mod unit;

#[cfg_attr(feature = "druid", derive(druid::Data, druid_enums::Matcher))]
#[derive(Debug, Clone)]
pub enum CType {
    String(CString),
    Bool(CBool),
    Integer(CInteger),
    Float(CFloat),
    Path(CPath),
    Tuple(CTuple),
    CStruct(CStruct),
    Seq(CSeq),
    Map(CMap),
    CEnum(CEnum),
    Option(Box<COption>),
    Unit(CUnit),
}

impl CType {
    pub fn is_leaf(&self) -> bool {
        todo!()
    }
    pub fn valid(&self) -> Result<(), InValid> {
        match self {
            Self::String(string) => string.valid(),
            Self::Bool(bool) => bool.valid(),
            Self::Integer(integer) => integer.valid(),
            Self::Float(float) => float.valid(),
            Self::Path(path) => path.valid(),
            Self::Tuple(tuple) => tuple.valid(),
            Self::CStruct(cstruct) => cstruct.valid(),
            Self::Seq(seq) => seq.valid(),
            Self::Map(map) => map.valid(),
            Self::CEnum(cenum) => cenum.valid(),
            Self::Option(option) => option.valid(),
            Self::Unit(unit) => unit.valid(),
        }
    }

    pub fn set_name(&mut self, name: &'static str) {
        match self {
            Self::String(string) => string.set_name(name),
            Self::Bool(bool) => bool.set_name(name),
            Self::Integer(integer) => integer.set_name(name),
            Self::Float(float) => float.set_name(name),
            Self::Path(path) => path.set_name(name),
            Self::Tuple(tuple) => tuple.set_name(name),
            Self::CStruct(cstruct) => cstruct.set_name(name),
            Self::Seq(seq) => seq.set_name(name),
            Self::Map(map) => map.set_name(name),
            Self::CEnum(cenum) => cenum.set_name(name),
            Self::Option(option) => option.set_name(name),
            Self::Unit(unit) => unit.set_name(name),
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

    pub fn as_seq(&self) -> Result<&CSeq, Error> {
        match self {
            Self::Seq(cvalue) => Ok(cvalue),
            _ => Err(Error::ExpectedVec),
        }
    }

    pub fn as_seq_mut(&mut self) -> Result<&mut CSeq, Error> {
        match self {
            Self::Seq(cvalue) => Ok(cvalue),
            _ => Err(Error::ExpectedVec),
        }
    }

    pub fn as_map(&self) -> Result<&CMap, Error> {
        match self {
            Self::Map(cvalue) => Ok(cvalue),
            _ => Err(Error::ExpectedMap),
        }
    }

    pub fn as_map_mut(&mut self) -> Result<&mut CMap, Error> {
        match self {
            Self::Map(cvalue) => Ok(cvalue),
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
}
