use std::path::PathBuf;

use serde::ser::{
    Error as _, Impossible, SerializeMap, SerializeSeq, SerializeStruct, SerializeStructVariant,
    SerializeTuple, SerializeTupleStruct, SerializeTupleVariant,
};
use serde::{Serialize, Serializer};

use crate::ctypes::cstruct::CStruct;
use crate::ctypes::map::CMap;
use crate::ctypes::seq::CSeq;
use crate::ctypes::tuple::CTuple;
use crate::ctypes::CType;
use crate::errors::Error;

pub struct ConfigSerializer<'a> {
    ty: &'a mut CType,
}

impl<'a> ConfigSerializer<'a> {
    pub fn new(ty: &'a mut CType) -> Self {
        Self { ty }
    }
}

impl<'a: 'b, 'b> Serializer for &'a mut ConfigSerializer<'b> {
    type Ok = ();
    type Error = Error;
    type SerializeSeq = ConfigSerializeSeq<'b>;
    type SerializeTuple = ConfigSerializeTuple<'b>;
    type SerializeTupleStruct = ConfigSerializeTuple<'b>;
    type SerializeTupleVariant = ConfigSerializeTuple<'b>;
    type SerializeMap = ConfigSerializeMap<'b>;
    type SerializeStruct = ConfigSerializeStruct<'b>;
    type SerializeStructVariant = ConfigSerializeStruct<'b>;

    fn serialize_bool(self, v: bool) -> Result<Self::Ok, Self::Error> {
        self.ty.as_bool_mut()?.value = Some(v);
        Ok(())
    }

    fn serialize_i8(self, v: i8) -> Result<Self::Ok, Self::Error> {
        self.serialize_i64(v as i64)
    }

    fn serialize_i16(self, v: i16) -> Result<Self::Ok, Self::Error> {
        self.serialize_i64(v as i64)
    }

    fn serialize_i32(self, v: i32) -> Result<Self::Ok, Self::Error> {
        self.serialize_i64(v as i64)
    }

    fn serialize_i64(self, v: i64) -> Result<Self::Ok, Self::Error> {
        self.ty.as_int_mut()?.value = Some(v);
        Ok(())
    }

    fn serialize_u8(self, v: u8) -> Result<Self::Ok, Self::Error> {
        self.serialize_i64(v as i64)
    }

    fn serialize_u16(self, v: u16) -> Result<Self::Ok, Self::Error> {
        self.serialize_i64(v as i64)
    }

    fn serialize_u32(self, v: u32) -> Result<Self::Ok, Self::Error> {
        self.serialize_i64(v as i64)
    }

    fn serialize_u64(self, v: u64) -> Result<Self::Ok, Self::Error> {
        self.serialize_i64(v as i64)
    }

    fn serialize_f32(self, v: f32) -> Result<Self::Ok, Self::Error> {
        self.serialize_f64(v as f64)
    }

    fn serialize_f64(self, v: f64) -> Result<Self::Ok, Self::Error> {
        self.ty.as_float_mut()?.value = Some(v);
        Ok(())
    }

    fn serialize_char(self, _v: char) -> Result<Self::Ok, Self::Error> {
        unimplemented!()
    }

    fn serialize_str(self, v: &str) -> Result<Self::Ok, Self::Error> {
        match &mut self.ty {
            CType::String(cstring) => cstring.value = Some(String::from(v)),
            CType::Path(cpath) => cpath.value = Some(PathBuf::from(v)),
            _ => return Err(Error::ExpectedStringOrPath),
        }
        Ok(())
    }

    fn serialize_bytes(self, _v: &[u8]) -> Result<Self::Ok, Self::Error> {
        todo!()
    }

    fn serialize_none(self) -> Result<Self::Ok, Self::Error> {
        self.ty.as_option_mut()?.active = false;
        Ok(())
    }

    fn serialize_some<T: ?Sized>(self, value: &T) -> Result<Self::Ok, Self::Error>
    where
        T: Serialize,
    {
        let coption = self.ty.as_option_mut()?;
        coption.active = true;
        value.serialize(&mut ConfigSerializer::new(&mut coption.ty))
    }

    fn serialize_unit(self) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }

    fn serialize_unit_struct(self, _name: &'static str) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }

    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
    ) -> Result<Self::Ok, Self::Error> {
        let cenum = self.ty.as_enum_mut()?;
        cenum.set_selected(variant)?.variant.as_unit()
    }

    fn serialize_newtype_struct<T: ?Sized>(
        self,
        _name: &'static str,
        value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: Serialize,
    {
        value.serialize(self)
    }

    fn serialize_newtype_variant<T: ?Sized>(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: Serialize,
    {
        let cenum = self.ty.as_enum_mut()?;
        let ctype = cenum.set_selected_mut(variant)?.variant.as_new_type_mut()?;
        value.serialize(&mut ConfigSerializer::new(ctype))
    }

    fn serialize_seq(self, _len: Option<usize>) -> Result<Self::SerializeSeq, Self::Error> {
        let cseq = self.ty.as_seq_mut()?;
        Ok(ConfigSerializeSeq::new(cseq))
    }

    fn serialize_tuple(self, _len: usize) -> Result<Self::SerializeTuple, Self::Error> {
        let ctuple = self.ty.as_tuple_mut()?;
        Ok(ConfigSerializeTuple::new(ctuple))
    }

    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleStruct, Self::Error> {
        let ctuple = self.ty.as_tuple_mut()?;
        Ok(ConfigSerializeTuple::new(ctuple))
    }

    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleVariant, Self::Error> {
        let cenum = self.ty.as_enum_mut()?;
        let ctype = cenum.set_selected_mut(variant)?.variant.as_tuple_mut()?;
        Ok(ConfigSerializeTuple::new(ctype))
    }

    fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap, Self::Error> {
        let cmap = self.ty.as_map_mut()?;
        Ok(ConfigSerializeMap::new(cmap))
    }

    fn serialize_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStruct, Self::Error> {
        let cstruct = self.ty.as_struct_mut()?;
        Ok(ConfigSerializeStruct::new(cstruct))
    }

    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStructVariant, Self::Error> {
        let cenum = self.ty.as_enum_mut()?;
        let cstruct = cenum.set_selected_mut(variant)?.variant.as_struct_mut()?;
        Ok(ConfigSerializeStruct::new(cstruct))
    }
}

pub struct ConfigSerializeStruct<'a> {
    cstruct: &'a mut CStruct,
}

impl<'a> ConfigSerializeStruct<'a> {
    fn new(cstruct: &'a mut CStruct) -> Self {
        Self { cstruct }
    }

    fn ser_field<T: ?Sized>(&mut self, key: &'static str, value: &T) -> Result<(), Error>
    where
        T: Serialize,
    {
        let ckwarg = self.cstruct.get_mut(key)?;
        value.serialize(&mut ConfigSerializer::new(&mut ckwarg.ty))
    }
}

impl<'b> SerializeStruct for ConfigSerializeStruct<'b> {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T: ?Sized>(
        &mut self,
        key: &'static str,
        value: &T,
    ) -> Result<(), Self::Error>
    where
        T: Serialize,
    {
        self.ser_field(key, value)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }
}

impl<'b> SerializeStructVariant for ConfigSerializeStruct<'b> {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T: ?Sized>(
        &mut self,
        key: &'static str,
        value: &T,
    ) -> Result<(), Self::Error>
    where
        T: Serialize,
    {
        self.ser_field(key, value)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }
}

pub struct ConfigSerializeSeq<'a> {
    cvec: &'a mut CSeq,
}

impl<'a> ConfigSerializeSeq<'a> {
    fn new(cvec: &'a mut CSeq) -> Self {
        cvec.inner.clear();
        Self { cvec }
    }
}

impl<'b> SerializeSeq for ConfigSerializeSeq<'b> {
    type Ok = ();
    type Error = Error;

    fn serialize_element<T: ?Sized>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: Serialize,
    {
        let mut ty = self.cvec.template.as_ref().clone();
        value.serialize(&mut ConfigSerializer::new(&mut ty))?;
        self.cvec.push(ty);
        Ok(())
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }
}

pub struct ConfigSerializeMap<'a> {
    cmap: &'a mut CMap,
    next_key: Option<String>,
}

impl<'a> ConfigSerializeMap<'a> {
    fn new(cmap: &'a mut CMap) -> Self {
        Self {
            cmap,
            next_key: None,
        }
    }
}

impl<'b> SerializeMap for ConfigSerializeMap<'b> {
    type Ok = ();
    type Error = Error;

    fn serialize_key<T: ?Sized>(&mut self, key: &T) -> Result<(), Self::Error>
    where
        T: Serialize,
    {
        self.next_key = Some(key.serialize(MapKeySerializer)?);
        Ok(())
    }

    fn serialize_value<T: ?Sized>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: Serialize,
    {
        let mut ty = self.cmap.value_template.as_ref().clone();
        value.serialize(&mut ConfigSerializer::new(&mut ty))?;
        self.cmap.inner.insert(self.next_key.take().unwrap(), ty);
        Ok(())
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }
}

pub struct ConfigSerializeTuple<'a> {
    ctuple: &'a mut CTuple,
    idx: usize,
}

impl<'a> ConfigSerializeTuple<'a> {
    pub fn new(ctuple: &'a mut CTuple) -> Self {
        Self { ctuple, idx: 0 }
    }

    fn ser_field<T: ?Sized>(&mut self, value: &T) -> Result<(), Error>
    where
        T: Serialize,
    {
        let ty = self.ctuple.get_mut(self.idx)?;
        self.idx += 1;
        value.serialize(&mut ConfigSerializer::new(ty))
    }

    fn ser_end(self) -> Result<(), Error> {
        if self.idx != self.ctuple.len() {
            Err(Error::custom("Not all field were serialized"))
        } else {
            Ok(())
        }
    }
}

impl<'b> SerializeTuple for ConfigSerializeTuple<'b> {
    type Ok = ();
    type Error = Error;

    fn serialize_element<T: ?Sized>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: Serialize,
    {
        self.ser_field(value)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        self.ser_end()
    }
}

impl<'b> SerializeTupleStruct for ConfigSerializeTuple<'b> {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T: ?Sized>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: Serialize,
    {
        self.ser_field(value)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        self.ser_end()
    }
}

impl<'b> SerializeTupleVariant for ConfigSerializeTuple<'b> {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T: ?Sized>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: Serialize,
    {
        self.ser_field(value)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        self.ser_end()
    }
}

struct MapKeySerializer;

impl Serializer for MapKeySerializer {
    type Ok = String;
    type Error = Error;
    type SerializeSeq = Impossible<Self::Ok, Self::Error>;
    type SerializeTuple = Impossible<Self::Ok, Self::Error>;
    type SerializeTupleStruct = Impossible<Self::Ok, Self::Error>;
    type SerializeTupleVariant = Impossible<Self::Ok, Self::Error>;
    type SerializeMap = Impossible<Self::Ok, Self::Error>;
    type SerializeStruct = Impossible<Self::Ok, Self::Error>;
    type SerializeStructVariant = Impossible<Self::Ok, Self::Error>;

    fn serialize_bool(self, _v: bool) -> Result<Self::Ok, Self::Error> {
        Err(Error::ExpectedString)
    }

    fn serialize_i8(self, _v: i8) -> Result<Self::Ok, Self::Error> {
        Err(Error::ExpectedString)
    }

    fn serialize_i16(self, _v: i16) -> Result<Self::Ok, Self::Error> {
        Err(Error::ExpectedString)
    }

    fn serialize_i32(self, _v: i32) -> Result<Self::Ok, Self::Error> {
        Err(Error::ExpectedString)
    }

    fn serialize_i64(self, _v: i64) -> Result<Self::Ok, Self::Error> {
        Err(Error::ExpectedString)
    }

    fn serialize_u8(self, _v: u8) -> Result<Self::Ok, Self::Error> {
        Err(Error::ExpectedString)
    }

    fn serialize_u16(self, _v: u16) -> Result<Self::Ok, Self::Error> {
        Err(Error::ExpectedString)
    }

    fn serialize_u32(self, _v: u32) -> Result<Self::Ok, Self::Error> {
        Err(Error::ExpectedString)
    }

    fn serialize_u64(self, _v: u64) -> Result<Self::Ok, Self::Error> {
        Err(Error::ExpectedString)
    }

    fn serialize_f32(self, _v: f32) -> Result<Self::Ok, Self::Error> {
        Err(Error::ExpectedString)
    }

    fn serialize_f64(self, _v: f64) -> Result<Self::Ok, Self::Error> {
        Err(Error::ExpectedString)
    }

    fn serialize_char(self, _v: char) -> Result<Self::Ok, Self::Error> {
        Err(Error::ExpectedString)
    }

    fn serialize_str(self, v: &str) -> Result<Self::Ok, Self::Error> {
        Ok(String::from(v))
    }

    fn serialize_bytes(self, _v: &[u8]) -> Result<Self::Ok, Self::Error> {
        Err(Error::ExpectedString)
    }

    fn serialize_none(self) -> Result<Self::Ok, Self::Error> {
        Err(Error::ExpectedString)
    }

    fn serialize_some<T: ?Sized>(self, _value: &T) -> Result<Self::Ok, Self::Error>
    where
        T: Serialize,
    {
        Err(Error::ExpectedString)
    }

    fn serialize_unit(self) -> Result<Self::Ok, Self::Error> {
        Err(Error::ExpectedString)
    }

    fn serialize_unit_struct(self, _name: &'static str) -> Result<Self::Ok, Self::Error> {
        Err(Error::ExpectedString)
    }

    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
    ) -> Result<Self::Ok, Self::Error> {
        Err(Error::ExpectedString)
    }

    fn serialize_newtype_struct<T: ?Sized>(
        self,
        _name: &'static str,
        _value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: Serialize,
    {
        Err(Error::ExpectedString)
    }

    fn serialize_newtype_variant<T: ?Sized>(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: Serialize,
    {
        Err(Error::ExpectedString)
    }

    fn serialize_seq(self, _len: Option<usize>) -> Result<Self::SerializeSeq, Self::Error> {
        Err(Error::ExpectedString)
    }

    fn serialize_tuple(self, _len: usize) -> Result<Self::SerializeTuple, Self::Error> {
        Err(Error::ExpectedString)
    }

    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleStruct, Self::Error> {
        Err(Error::ExpectedString)
    }

    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleVariant, Self::Error> {
        Err(Error::ExpectedString)
    }

    fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap, Self::Error> {
        Err(Error::ExpectedString)
    }

    fn serialize_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStruct, Self::Error> {
        Err(Error::ExpectedString)
    }

    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStructVariant, Self::Error> {
        Err(Error::ExpectedString)
    }
}
