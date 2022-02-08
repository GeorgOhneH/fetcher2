use crate::ctypes::bool::CBool;
use crate::ctypes::cenum::{CArg, CArgVariant, CEnumBuilder};
use crate::ctypes::cstruct::{CKwarg, CStructBuilder};
use crate::ctypes::integer::CInteger;
use crate::ctypes::option::COption;
use crate::ctypes::tuple::{CTuple, CTupleBuilder};
use crate::ctypes::CType;
use crate::errors::Error;
use std::error::Error as StdError;

pub use config_derive::Travel;

pub trait Travel {
    fn travel<T>(traveller: T) -> Result<T::Ok, T::Error>
    where
        T: Traveller;
}

impl Travel for i64 {
    fn travel<T>(traveller: T) -> Result<T::Ok, T::Error>
    where
        T: Traveller,
    {
        traveller.found_i64()
    }
}

impl Travel for bool {
    fn travel<T>(traveller: T) -> Result<T::Ok, T::Error>
    where
        T: Traveller,
    {
        traveller.found_bool()
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

impl<U, const N: usize> Travel for [U; N] where U: Travel {
    fn travel<T>(traveller: T) -> Result<T::Ok, T::Error> where T: Traveller {
        let mut state = traveller.found_tuple()?;
        for _ in 0..N {
            state.found_element::<U>()?;
        }
        state.end()
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
    fn found_option<T: Travel>(self) -> Result<Self::Ok, Self::Error>;
    fn found_tuple(self) -> Result<Self::TravellerTuple, Self::Error>;
    fn found_struct(self) -> Result<Self::TravellerStruct, Self::Error>;
    fn found_enum(self) -> Result<Self::TravellerEnum, Self::Error>;
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
        Ok(CType::Integer(CInteger::new()))
    }

    fn found_option<T: Travel>(self) -> Result<Self::Ok, Self::Error> {
        let ty = T::travel(&mut ConfigTraveller::new())?;
        Ok(CType::Option(Box::new(COption::new(ty))))
    }

    fn found_tuple(self) -> Result<Self::TravellerTuple, Self::Error> {
        Ok(ConfigTravellerTuple::new())
    }

    fn found_struct(self) -> Result<Self::TravellerStruct, Self::Error> {
        Ok(ConfigTravellerStruct::new())
    }

    fn found_enum(self) -> Result<Self::TravellerEnum, Self::Error> {
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
