use serde::de::value::StrDeserializer;
use serde::de::{DeserializeSeed, EnumAccess, Error as _, MapAccess, SeqAccess, Visitor};
use serde::de::{IntoDeserializer, VariantAccess};
use serde::Deserializer;

use crate::ctypes::cenum::{CArg, CEnum};
use crate::ctypes::cstruct::CStruct;
use crate::ctypes::map::CMap;
use crate::ctypes::seq::{CItem, CSeq};
use crate::ctypes::tuple::CTuple;
use crate::ctypes::CType;
use crate::errors::Error;

pub struct ConfigDeserializer<'a> {
    ty: &'a CType,
}

impl<'a> ConfigDeserializer<'a> {
    pub fn new(ty: &'a CType) -> Self {
        Self { ty }
    }
}

impl<'a, 'de> Deserializer<'de> for &'a mut ConfigDeserializer<'de> {
    type Error = Error;

    fn deserialize_any<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        unimplemented!()
    }

    fn deserialize_bool<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_bool(self.ty.as_bool()?.value.ok_or(Error::ValueRequired)?)
    }

    fn deserialize_i8<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.deserialize_i64(visitor)
    }

    fn deserialize_i16<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.deserialize_i64(visitor)
    }

    fn deserialize_i32<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.deserialize_i64(visitor)
    }

    fn deserialize_i64<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_i64(self.ty.as_int()?.value.ok_or(Error::ValueRequired)?)
    }

    fn deserialize_u8<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.deserialize_u64(visitor)
    }

    fn deserialize_u16<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.deserialize_u64(visitor)
    }

    fn deserialize_u32<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.deserialize_u64(visitor)
    }

    fn deserialize_u64<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_u64(self.ty.as_int()?.value.ok_or(Error::ValueRequired)? as u64)
    }

    fn deserialize_f32<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.deserialize_f64(visitor)
    }

    fn deserialize_f64<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_f64(self.ty.as_float()?.value.ok_or(Error::ValueRequired)?)
    }

    fn deserialize_char<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        unimplemented!()
    }

    fn deserialize_str<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match &self.ty {
            CType::String(cstring) => {
                visitor.visit_str(cstring.value.as_ref().ok_or(Error::ValueRequired)?)
            }
            CType::Path(cpath) => visitor.visit_str(
                cpath
                    .value
                    .as_ref()
                    .ok_or(Error::ValueRequired)?
                    .as_os_str()
                    .to_str()
                    .ok_or_else(|| Error::custom("Path it not utf8"))?,
            ),
            _ => Err(Error::ExpectedStringOrPath),
        }
    }

    fn deserialize_string<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.deserialize_str(visitor)
    }

    fn deserialize_bytes<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        unimplemented!()
    }

    fn deserialize_byte_buf<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        unimplemented!()
    }

    fn deserialize_option<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        let coption = self.ty.as_option()?;
        if coption.active {
            visitor.visit_some(&mut ConfigDeserializer::new(&coption.ty))
        } else {
            visitor.visit_none()
        }
    }

    fn deserialize_unit<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_unit()
    }

    fn deserialize_unit_struct<V>(
        self,
        _name: &'static str,
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.deserialize_unit(visitor)
    }

    fn deserialize_newtype_struct<V>(
        self,
        _name: &'static str,
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_newtype_struct(self)
    }

    fn deserialize_seq<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        let cseq = self.ty.as_seq()?;
        visitor.visit_seq(ConfigDeserializerSeq::new(cseq))
    }

    fn deserialize_tuple<V>(self, _len: usize, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        let ctuple = self.ty.as_tuple()?;
        visitor.visit_seq(ConfigDeserializerTuple::new(ctuple))
    }

    fn deserialize_tuple_struct<V>(
        self,
        _name: &'static str,
        _len: usize,
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        let ctuple = self.ty.as_tuple()?;
        visitor.visit_seq(ConfigDeserializerTuple::new(ctuple))
    }

    fn deserialize_map<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        let cmap = self.ty.as_map()?;
        visitor.visit_map(ConfigDeserializerMap::new(cmap))
    }

    fn deserialize_struct<V>(
        self,
        _name: &'static str,
        _fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        let cstruct = self.ty.as_struct()?;
        visitor.visit_map(ConfigDeserializerStruct::new(cstruct))
    }

    fn deserialize_enum<V>(
        self,
        _name: &'static str,
        _variants: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        let cenum = self.ty.as_enum()?;
        visitor.visit_enum(ConfigEnumAccess::new(cenum))
    }

    fn deserialize_identifier<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        todo!()
    }

    fn deserialize_ignored_any<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        unimplemented!()
    }
}

pub struct ConfigDeserializerStruct<'a> {
    cstruct: &'a CStruct,
    iter: im::ordmap::Iter<'a, &'static str, usize>,
    next_value: Option<usize>,
}

impl<'a> ConfigDeserializerStruct<'a> {
    pub fn new(cstruct: &'a CStruct) -> Self {
        Self {
            cstruct,
            iter: cstruct.index_map.iter(),
            next_value: None,
        }
    }
}

impl<'de> MapAccess<'de> for ConfigDeserializerStruct<'de> {
    type Error = Error;

    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>, Self::Error>
    where
        K: DeserializeSeed<'de>,
    {
        if let Some((key, value)) = self.iter.next() {
            self.next_value = Some(*value);
            let field_deserializer: StrDeserializer<Error> = key.into_deserializer();
            Ok(Some(seed.deserialize(field_deserializer)?))
        } else {
            Ok(None)
        }
    }

    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value, Self::Error>
    where
        V: DeserializeSeed<'de>,
    {
        let ty = &self.cstruct.inner[self.next_value.unwrap()].ty;
        seed.deserialize(&mut ConfigDeserializer::new(ty))
    }
}

pub struct ConfigDeserializerMap<'a> {
    cmap: &'a CMap,
    iter: im::ordmap::Iter<'a, String, CType>,
    next_value: Option<&'a CType>,
}

impl<'a> ConfigDeserializerMap<'a> {
    pub fn new(cmap: &'a CMap) -> Self {
        Self {
            cmap,
            iter: cmap.inner.iter(),
            next_value: None,
        }
    }
}

impl<'de> MapAccess<'de> for ConfigDeserializerMap<'de> {
    type Error = Error;

    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>, Self::Error>
    where
        K: DeserializeSeed<'de>,
    {
        if let Some((key, value)) = self.iter.next() {
            self.next_value = Some(value);
            let str_deserializer: StrDeserializer<'de, Error> = key.as_str().into_deserializer();
            Ok(Some(seed.deserialize(str_deserializer)?))
        } else {
            Ok(None)
        }
    }

    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value, Self::Error>
    where
        V: DeserializeSeed<'de>,
    {
        seed.deserialize(&mut ConfigDeserializer::new(self.next_value.unwrap()))
    }
}

pub struct ConfigDeserializerTuple<'a> {
    ctuple: &'a CTuple,
    iter: im::vector::Iter<'a, CType>,
}

impl<'a> ConfigDeserializerTuple<'a> {
    pub fn new(ctuple: &'a CTuple) -> Self {
        Self {
            ctuple,
            iter: ctuple.inner.iter(),
        }
    }
}

impl<'de> SeqAccess<'de> for ConfigDeserializerTuple<'de> {
    type Error = Error;

    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>, Self::Error>
    where
        T: DeserializeSeed<'de>,
    {
        self.iter
            .next()
            .map(|element| seed.deserialize(&mut ConfigDeserializer::new(element)))
            .transpose()
    }
}

pub struct ConfigDeserializerSeq<'a> {
    cseq: &'a CSeq,
    iter: im::vector::Iter<'a, CItem>,
}

impl<'a> ConfigDeserializerSeq<'a> {
    pub fn new(cseq: &'a CSeq) -> Self {
        Self {
            cseq,
            iter: cseq.inner.iter(),
        }
    }
}

impl<'de> SeqAccess<'de> for ConfigDeserializerSeq<'de> {
    type Error = Error;

    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>, Self::Error>
    where
        T: DeserializeSeed<'de>,
    {
        self.iter
            .next()
            .map(|item| seed.deserialize(&mut ConfigDeserializer::new(&item.ty)))
            .transpose()
    }
}

pub struct ConfigEnumAccess<'a> {
    cenum: &'a CEnum,
}

impl<'a> ConfigEnumAccess<'a> {
    pub fn new(cenum: &'a CEnum) -> Self {
        Self { cenum }
    }
}

impl<'de> EnumAccess<'de> for ConfigEnumAccess<'de> {
    type Error = Error;
    type Variant = ConfigVariantAccess<'de>;

    fn variant_seed<V>(self, seed: V) -> Result<(V::Value, Self::Variant), Self::Error>
    where
        V: DeserializeSeed<'de>,
    {
        let carg = self.cenum.get_selected()?;
        let field_deserializer: StrDeserializer<Error> = carg.name.into_deserializer();
        Ok((
            seed.deserialize(field_deserializer)?,
            ConfigVariantAccess::new(carg),
        ))
    }
}

pub struct ConfigVariantAccess<'a> {
    carg: &'a CArg,
}

impl<'a> ConfigVariantAccess<'a> {
    pub fn new(carg: &'a CArg) -> Self {
        Self { carg }
    }
}

impl<'de> VariantAccess<'de> for ConfigVariantAccess<'de> {
    type Error = Error;

    fn unit_variant(self) -> Result<(), Self::Error> {
        self.carg.variant.as_unit()
    }

    fn newtype_variant_seed<T>(self, seed: T) -> Result<T::Value, Self::Error>
    where
        T: DeserializeSeed<'de>,
    {
        let ty = self.carg.variant.as_new_type()?;
        seed.deserialize(&mut ConfigDeserializer::new(ty))
    }

    fn tuple_variant<V>(self, _len: usize, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        let ctuple = self.carg.variant.as_tuple()?;
        visitor.visit_seq(ConfigDeserializerTuple::new(ctuple))
    }

    fn struct_variant<V>(
        self,
        _fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        let cstruct = self.carg.variant.as_struct()?;
        visitor.visit_map(ConfigDeserializerStruct::new(cstruct))
    }
}
