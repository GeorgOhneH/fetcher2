use crate::ctypes::bool::CBool;
use crate::ctypes::cenum::{CArg, CArgVariant, CEnumBuilder};
use crate::ctypes::cstruct::{CKwarg, CStructBuilder};
use crate::ctypes::integer::CInteger;
use crate::ctypes::option::COption;
use crate::ctypes::tuple::{CTuple, CTupleBuilder};
use crate::ctypes::CType;
use crate::errors::Error;
use druid::im::Vector;
use druid::FileSpec;
use std::collections::HashMap;
use std::error::Error as StdError;
use std::path::PathBuf;

use crate::ctypes::map::CMap;
use crate::ctypes::path::CPath;
use crate::ctypes::seq::CSeq;
use crate::ctypes::string::CString;
pub use config_derive::Travel;
use crate::ctypes::float::CFloat;

pub trait Travel {
    fn travel<T>(traveller: T) -> Result<T::Ok, T::Error>
    where
        T: Traveller;
}

impl Travel for bool {
    fn travel<T>(traveller: T) -> Result<T::Ok, T::Error>
        where
            T: Traveller,
    {
        traveller.found_bool()
    }
}

impl Travel for i64 {
    fn travel<T>(traveller: T) -> Result<T::Ok, T::Error>
    where
        T: Traveller,
    {
        traveller.found_i64()
    }
}


impl Travel for u64 {
    fn travel<T>(traveller: T) -> Result<T::Ok, T::Error>
        where
            T: Traveller,
    {
        traveller.found_u64()
    }
}

impl Travel for f64 {
    fn travel<T>(traveller: T) -> Result<T::Ok, T::Error> where T: Traveller {
        traveller.found_f64()
    }
}

impl Travel for String {
    fn travel<T>(traveller: T) -> Result<T::Ok, T::Error>
    where
        T: Traveller,
    {
        traveller.found_str()
    }
}

impl Travel for PathBuf {
    fn travel<T>(traveller: T) -> Result<T::Ok, T::Error>
    where
        T: Traveller,
    {
        traveller.found_path(TravelPathConfig::Any)
    }
}

impl Travel for () {
    fn travel<T>(traveller: T) -> Result<T::Ok, T::Error>
    where
        T: Traveller,
    {
        traveller.found_unit()
    }
}

impl<U> Travel for Option<U>
where
    U: Travel,
{
    fn travel<T>(traveller: T) -> Result<T::Ok, T::Error>
    where
        T: Traveller,
    {
        traveller.found_option::<U>()
    }
}

impl<U, V> Travel for (U, V)
where
    U: Travel,
    V: Travel,
{
    fn travel<T>(traveller: T) -> Result<T::Ok, T::Error>
    where
        T: Traveller,
    {
        let mut state = traveller.found_tuple()?;
        state.found_element::<U>()?;
        state.found_element::<V>()?;
        state.end()
    }
}

impl<U, const N: usize> Travel for [U; N]
where
    U: Travel,
{
    fn travel<T>(traveller: T) -> Result<T::Ok, T::Error>
    where
        T: Traveller,
    {
        let mut state = traveller.found_tuple()?;
        for _ in 0..N {
            state.found_element::<U>()?;
        }
        state.end()
    }
}

impl<U: Travel> Travel for Vec<U> {
    fn travel<T>(traveller: T) -> Result<T::Ok, T::Error>
    where
        T: Traveller,
    {
        traveller.found_seq::<U>()
    }
}

impl<K, V: Travel> Travel for HashMap<K, V> {
    fn travel<T>(traveller: T) -> Result<T::Ok, T::Error>
    where
        T: Traveller,
    {
        traveller.found_map::<V>()
    }
}

pub trait Traveller {
    type Ok;
    type Error: StdError;
    type TravellerStruct: TravellerStruct<Ok = Self::Ok, Error = Self::Error>;
    type TravellerEnum: TravellerEnum<Ok = Self::Ok, Error = Self::Error>;
    type TravellerTuple: TravellerTuple<Ok = Self::Ok, Error = Self::Error>;

    fn found_bool(self) -> Result<Self::Ok, Self::Error>;
    fn found_i64(self) -> Result<Self::Ok, Self::Error>;
    fn found_u64(self) -> Result<Self::Ok, Self::Error>;
    fn found_ranged_int(self, min: i64, max: i64) -> Result<Self::Ok, Self::Error>;
    fn found_f64(self) -> Result<Self::Ok, Self::Error>;
    fn found_ranged_float(self, min: f64, max: f64) -> Result<Self::Ok, Self::Error>;
    fn found_str(self) -> Result<Self::Ok, Self::Error>;
    fn found_path(self, path_config: TravelPathConfig) -> Result<Self::Ok, Self::Error>;
    fn found_unit(self) -> Result<Self::Ok, Self::Error>;
    fn found_unit_struct(self) -> Result<Self::Ok, Self::Error>;
    fn found_option<T: Travel>(self) -> Result<Self::Ok, Self::Error>;
    fn found_tuple(self) -> Result<Self::TravellerTuple, Self::Error>;
    fn found_tuple_struct(self, name: &'static str) -> Result<Self::TravellerTuple, Self::Error>;
    fn found_seq<T: Travel>(self) -> Result<Self::Ok, Self::Error>;
    fn found_map<V: Travel>(self) -> Result<Self::Ok, Self::Error>;
    fn found_struct(self, name: &'static str) -> Result<Self::TravellerStruct, Self::Error>;
    fn found_enum(self, name: &'static str) -> Result<Self::TravellerEnum, Self::Error>;
}

#[derive(Debug, Clone)]
pub enum TravelPathConfig {
    Any,
    Relative,
    Absolute,
    AbsoluteExist,
    AbsoluteExistDir,
    AbsoluteExistFile(Vector<FileSpec>),
}

pub trait TravellerStruct {
    type Ok;
    type Error: StdError;

    fn found_field<T: ?Sized>(&mut self, key: &'static str) -> Result<(), Self::Error>
    where
        T: Travel;

    fn end(self) -> Result<Self::Ok, Self::Error>;
}

pub trait TravellerEnum {
    type Ok;
    type Error: StdError;

    type TravellerTupleVariant<'a>: TravellerTupleVariant<Error = Self::Error>
    where
        Self: 'a;

    type TravellerStructVariant<'a>: TravellerStructVariant<Error = Self::Error>
    where
        Self: 'a;

    fn found_unit_variant(&mut self, name: &'static str) -> Result<(), Self::Error>;

    fn found_newtype_variant<T: ?Sized>(&mut self, name: &'static str) -> Result<(), Self::Error>
    where
        T: Travel;

    fn found_tuple_variant<'a>(
        &'a mut self,
        key: &'static str,
    ) -> Result<Self::TravellerTupleVariant<'a>, Self::Error>;

    fn found_struct_variant<'a>(
        &'a mut self,
        key: &'static str,
    ) -> Result<Self::TravellerStructVariant<'a>, Self::Error>;

    fn end(self) -> Result<Self::Ok, Self::Error>;
}

pub trait TravellerTupleVariant {
    type Error: StdError;

    fn found_element<T: ?Sized>(&mut self) -> Result<(), Self::Error>
    where
        T: Travel;

    fn end(self) -> Result<(), Self::Error>;
}

pub trait TravellerStructVariant {
    type Error: StdError;

    fn found_field<T: ?Sized>(&mut self, key: &'static str) -> Result<(), Self::Error>
    where
        T: Travel;

    fn end(self) -> Result<(), Self::Error>;
}

pub trait TravellerTuple {
    type Ok;
    type Error: StdError;

    fn found_element<T: ?Sized>(&mut self) -> Result<(), Self::Error>
    where
        T: Travel;

    fn end(self) -> Result<Self::Ok, Self::Error>;
}

pub struct ConfigTraveller {}

impl ConfigTraveller {
    pub fn new() -> Self {
        Self {}
    }
}

impl<'a> Traveller for &'a mut ConfigTraveller {
    type Ok = CType;
    type Error = Error;

    type TravellerStruct = ConfigTravellerStruct;
    type TravellerEnum = ConfigTravellerEnum;
    type TravellerTuple = ConfigTravellerTuple;

    fn found_bool(self) -> Result<Self::Ok, Self::Error> {
        Ok(CType::Bool(CBool::new()))
    }

    fn found_i64(self) -> Result<Self::Ok, Self::Error> {
        Ok(CType::Integer(CInteger::new(i64::MIN, i64::MAX)))
    }

    fn found_u64(self) -> Result<Self::Ok, Self::Error> {
        Ok(CType::Integer(CInteger::new(0, i64::MAX)))
    }

    fn found_ranged_int(self, min: i64, max: i64) -> Result<Self::Ok, Self::Error> {
        Ok(CType::Integer(CInteger::new(min, max)))
    }

    fn found_f64(self) -> Result<Self::Ok, Self::Error> {
        Ok(CType::Float(CFloat::new(f64::MIN, f64::MAX)))
    }

    fn found_ranged_float(self, min: f64, max: f64) -> Result<Self::Ok, Self::Error> {
        Ok(CType::Float(CFloat::new(min, max)))
    }

    fn found_str(self) -> Result<Self::Ok, Self::Error> {
        Ok(CType::String(CString::new()))
    }

    fn found_path(
        self,
        path_config: TravelPathConfig,
    ) -> Result<Self::Ok, Self::Error> {
        Ok(CType::Path(CPath::new(path_config)))
    }

    fn found_unit(self) -> Result<Self::Ok, Self::Error> {
        panic!("unit is not supported. Consider skipping this field")
    }

    fn found_unit_struct(self) -> Result<Self::Ok, Self::Error> {
        panic!("unit_structs are not supported. Consider skipping this field")
    }

    fn found_option<T: Travel>(self) -> Result<Self::Ok, Self::Error> {
        let ty = T::travel(&mut ConfigTraveller::new())?;
        Ok(CType::Option(Box::new(COption::new(ty))))
    }

    fn found_tuple(self) -> Result<Self::TravellerTuple, Self::Error> {
        Ok(ConfigTravellerTuple::new())
    }

    fn found_tuple_struct(self, name: &'static str) -> Result<Self::TravellerTuple, Self::Error> {
        Ok(ConfigTravellerTuple::new())
    }

    fn found_seq<T: Travel>(self) -> Result<Self::Ok, Self::Error> {
        let ty = T::travel(&mut ConfigTraveller::new())?;
        Ok(CType::Seq(CSeq::new(ty)))
    }

    fn found_map<V: Travel>(self) -> Result<Self::Ok, Self::Error> {
        let value = V::travel(&mut ConfigTraveller::new())?;
        Ok(CType::Map(CMap::new(value)))
    }

    fn found_struct(self, name: &'static str) -> Result<Self::TravellerStruct, Self::Error> {
        Ok(ConfigTravellerStruct::new())
    }

    fn found_enum(self, name: &'static str) -> Result<Self::TravellerEnum, Self::Error> {
        Ok(ConfigTravellerEnum::new())
    }
}

pub struct ConfigTravellerStruct {
    cstruct: CStructBuilder,
}

impl ConfigTravellerStruct {
    pub fn new() -> Self {
        Self {
            cstruct: CStructBuilder::new(),
        }
    }
}

impl TravellerStruct for ConfigTravellerStruct {
    type Ok = CType;
    type Error = Error;

    fn found_field<T: ?Sized>(&mut self, key: &'static str) -> Result<(), Self::Error>
    where
        T: Travel,
    {
        let ty = T::travel(&mut ConfigTraveller::new())?;
        self.cstruct.arg(CKwarg::new(key, ty));
        Ok(())
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(CType::CStruct(self.cstruct.build()))
    }
}

pub struct ConfigTravellerEnum {
    cenum: CEnumBuilder,
}

impl ConfigTravellerEnum {
    pub fn new() -> Self {
        Self {
            cenum: CEnumBuilder::new(),
        }
    }
}

impl TravellerEnum for ConfigTravellerEnum {
    type Ok = CType;
    type Error = Error;
    type TravellerTupleVariant<'a> = ConfigTravellerTupleVariant<'a>;
    type TravellerStructVariant<'a> = ConfigTravellerStructVariant<'a>;

    fn found_unit_variant(&mut self, name: &'static str) -> Result<(), Self::Error> {
        let carg = CArg::new(name, CArgVariant::Unit);
        self.cenum.arg(carg);
        Ok(())
    }

    fn found_newtype_variant<T: ?Sized>(&mut self, name: &'static str) -> Result<(), Self::Error>
    where
        T: Travel,
    {
        let ty = T::travel(&mut ConfigTraveller::new())?;
        let carg = CArg::new(name, CArgVariant::NewType(ty));
        self.cenum.arg(carg);
        Ok(())
    }

    fn found_tuple_variant<'a>(
        &'a mut self,
        key: &'static str,
    ) -> Result<Self::TravellerTupleVariant<'a>, Self::Error> {
        Ok(ConfigTravellerTupleVariant::new(key, &mut self.cenum))
    }

    fn found_struct_variant<'a>(
        &'a mut self,
        key: &'static str,
    ) -> Result<Self::TravellerStructVariant<'a>, Self::Error> {
        Ok(ConfigTravellerStructVariant::new(key, &mut self.cenum))
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(CType::CEnum(self.cenum.build()))
    }
}

pub struct ConfigTravellerTupleVariant<'a> {
    name: &'static str,
    cenum: &'a mut CEnumBuilder,
    ctuple: CTupleBuilder,
}

impl<'a> ConfigTravellerTupleVariant<'a> {
    fn new(name: &'static str, cenum: &'a mut CEnumBuilder) -> Self {
        Self {
            name,
            cenum,
            ctuple: CTupleBuilder::new(),
        }
    }
}

impl<'a> TravellerTupleVariant for ConfigTravellerTupleVariant<'a> {
    type Error = Error;

    fn found_element<T: ?Sized>(&mut self) -> Result<(), Self::Error>
    where
        T: Travel,
    {
        let ty = T::travel(&mut ConfigTraveller::new())?;
        self.ctuple.add_element(ty);
        Ok(())
    }

    fn end(self) -> Result<(), Self::Error> {
        let ctuple = self.ctuple.build();
        self.cenum
            .arg(CArg::new(self.name, CArgVariant::Tuple(ctuple)));
        Ok(())
    }
}

pub struct ConfigTravellerStructVariant<'a> {
    name: &'static str,
    cenum: &'a mut CEnumBuilder,
    cstruct: CStructBuilder,
}

impl<'a> ConfigTravellerStructVariant<'a> {
    fn new(name: &'static str, cenum: &'a mut CEnumBuilder) -> Self {
        Self {
            name,
            cenum,
            cstruct: CStructBuilder::new(),
        }
    }
}

impl<'a> TravellerStructVariant for ConfigTravellerStructVariant<'a> {
    type Error = Error;

    fn found_field<T: ?Sized>(&mut self, key: &'static str) -> Result<(), Self::Error>
    where
        T: Travel,
    {
        let ty = T::travel(&mut ConfigTraveller::new())?;
        let kwarg = CKwarg::new(key, ty);
        self.cstruct.arg(kwarg);
        Ok(())
    }

    fn end(self) -> Result<(), Self::Error> {
        let cstruct = self.cstruct.build();
        self.cenum
            .arg(CArg::new(self.name, CArgVariant::Struct(cstruct)));
        Ok(())
    }
}

pub struct ConfigTravellerTuple {
    ctuple: CTupleBuilder,
}

impl ConfigTravellerTuple {
    pub fn new() -> Self {
        Self {
            ctuple: CTupleBuilder::new(),
        }
    }
}

impl TravellerTuple for ConfigTravellerTuple {
    type Ok = CType;
    type Error = Error;

    fn found_element<T: ?Sized>(&mut self) -> Result<(), Self::Error>
    where
        T: Travel,
    {
        let ty = T::travel(&mut ConfigTraveller::new())?;
        self.ctuple.add_element(ty);
        Ok(())
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(CType::Tuple(self.ctuple.build()))
    }
}
