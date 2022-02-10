use std::iter::FromIterator;

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
use crate::ctypes::CType;
use crate::errors::Error;
use druid::{lens, Data, LensExt, Widget, WidgetExt};
use druid_enums::Matcher;

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

impl CType {
    pub fn widget() -> impl Widget<Self> {
        Self::matcher()
            .string(CString::widget())
            .bool(CBool::widget())
            .integer(CInteger::widget())
            .float(CFloat::widget())
            .path(CPath::widget())
            .c_struct(CStruct::widget())
            .map(CMap::widget())
            .seq(CSeq::widget())
            .c_enum(CEnum::widget())
            .option(COption::widget().lens(lens::Identity.deref()))
    }
}
