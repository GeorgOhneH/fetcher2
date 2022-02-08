use crate::ctypes::cenum::CArgVariant;
use crate::ctypes::cstruct::CStruct;
use crate::ctypes::map::CHashMap;
use crate::ctypes::tuple::CTuple;
use crate::ctypes::vec::CVec;
use crate::ctypes::CType;
use crate::errors::Error;
use serde::ser::{
    Error as _, SerializeMap, SerializeSeq, SerializeStruct, SerializeStructVariant,
    SerializeTuple, SerializeTupleStruct, SerializeTupleVariant,
};
use serde::{Serialize, Serializer};
use valuable::{Value, Visit};

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
    type SerializeTupleStruct = NotImplemented;
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
        self.ty.as_int_mut()?.value = Some(v as isize);
        Ok(())
    }

    fn serialize_u8(self, v: u8) -> Result<Self::Ok, Self::Error> {
        todo!()
    }

    fn serialize_u16(self, v: u16) -> Result<Self::Ok, Self::Error> {
        todo!()
    }

    fn serialize_u32(self, v: u32) -> Result<Self::Ok, Self::Error> {
        todo!()
    }

    fn serialize_u64(self, v: u64) -> Result<Self::Ok, Self::Error> {
        todo!()
    }

    fn serialize_f32(self, v: f32) -> Result<Self::Ok, Self::Error> {
        todo!()
    }

    fn serialize_f64(self, v: f64) -> Result<Self::Ok, Self::Error> {
        todo!()
    }

    fn serialize_char(self, v: char) -> Result<Self::Ok, Self::Error> {
        todo!()
    }

    fn serialize_str(self, v: &str) -> Result<Self::Ok, Self::Error> {
        todo!()
    }

    fn serialize_bytes(self, v: &[u8]) -> Result<Self::Ok, Self::Error> {
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
        todo!()
    }

    fn serialize_unit_struct(self, name: &'static str) -> Result<Self::Ok, Self::Error> {
        todo!()
    }

    fn serialize_unit_variant(
        self,
        name: &'static str,
        variant_index: u32,
        variant: &'static str,
    ) -> Result<Self::Ok, Self::Error> {
        let cenum = self.ty.as_enum_mut()?;
        cenum.set_selected(variant)?.variant.as_unit()
    }

    fn serialize_newtype_struct<T: ?Sized>(
        self,
        name: &'static str,
        value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: Serialize,
    {
        todo!()
    }

    fn serialize_newtype_variant<T: ?Sized>(
        self,
        name: &'static str,
        variant_index: u32,
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

    fn serialize_seq(self, len: Option<usize>) -> Result<Self::SerializeSeq, Self::Error> {
        todo!()
    }

    fn serialize_tuple(self, len: usize) -> Result<Self::SerializeTuple, Self::Error> {
        let ctuple = self.ty.as_tuple_mut()?;
        Ok(ConfigSerializeTuple::new(ctuple))
    }

    fn serialize_tuple_struct(
        self,
        name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleStruct, Self::Error> {
        todo!()
    }

    fn serialize_tuple_variant(
        self,
        name: &'static str,
        variant_index: u32,
        variant: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleVariant, Self::Error> {
        let cenum = self.ty.as_enum_mut()?;
        let ctype = cenum.set_selected_mut(variant)?.variant.as_tuple_mut()?;
        Ok(ConfigSerializeTuple::new(ctype))
    }

    fn serialize_map(self, len: Option<usize>) -> Result<Self::SerializeMap, Self::Error> {
        todo!()
    }

    fn serialize_struct(
        self,
        name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeStruct, Self::Error> {
        let cstruct = self.ty.as_struct_mut()?;
        Ok(ConfigSerializeStruct::new(cstruct))
    }

    fn serialize_struct_variant(
        self,
        name: &'static str,
        variant_index: u32,
        variant: &'static str,
        len: usize,
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
        let ckwarg = self.cstruct.get_mut(key)?;
        value.serialize(&mut ConfigSerializer::new(&mut ckwarg.ty))
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
        let ckwarg = self.cstruct.get_mut(key)?;
        value.serialize(&mut ConfigSerializer::new(&mut ckwarg.ty))
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }
}

pub struct ConfigSerializeSeq<'a> {
    cvec: &'a mut CVec,
}

impl<'a> ConfigSerializeSeq<'a> {
    fn new(cvec: &'a mut CVec) -> Self {
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
        todo!()
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        todo!()
    }
}

pub struct ConfigSerializeMap<'a> {
    cmap: &'a mut CHashMap,
}

impl<'a> ConfigSerializeMap<'a> {
    fn new(cmap: &'a mut CHashMap) -> Self {
        Self { cmap }
    }
}

impl<'b> SerializeMap for ConfigSerializeMap<'b> {
    type Ok = ();
    type Error = Error;

    fn serialize_key<T: ?Sized>(&mut self, key: &T) -> Result<(), Self::Error>
    where
        T: Serialize,
    {
        todo!()
    }

    fn serialize_value<T: ?Sized>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: Serialize,
    {
        todo!()
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        todo!()
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
}

impl<'b> SerializeTuple for ConfigSerializeTuple<'b> {
    type Ok = ();
    type Error = Error;

    fn serialize_element<T: ?Sized>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: Serialize,
    {
        let ty = self.ctuple.get_mut(self.idx)?;
        self.idx += 1;
        value.serialize(&mut ConfigSerializer::new(ty))
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        if self.idx != self.ctuple.len() {
            Err(Error::custom("Not all field were serialized"))
        } else {
            Ok(())
        }
    }
}

impl<'b> SerializeTupleVariant for ConfigSerializeTuple<'b> {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T: ?Sized>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: Serialize,
    {
        let ty = self.ctuple.get_mut(self.idx)?;
        self.idx += 1;
        value.serialize(&mut ConfigSerializer::new(ty))
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        if self.idx != self.ctuple.len() {
            Err(Error::custom("Not all field were serialized"))
        } else {
            Ok(())
        }
    }
}

pub struct NotImplemented;

impl SerializeTuple for NotImplemented {
    type Ok = ();
    type Error = Error;

    fn serialize_element<T: ?Sized>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: Serialize,
    {
        todo!()
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        todo!()
    }
}

impl SerializeTupleStruct for NotImplemented {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T: ?Sized>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: Serialize,
    {
        todo!()
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        todo!()
    }
}

impl SerializeStructVariant for NotImplemented {
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
        todo!()
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        todo!()
    }
}
