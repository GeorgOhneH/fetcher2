#![allow(dead_code)]
#![feature(backtrace)]
#![feature(int_roundings)]
#![feature(box_patterns)]
#![allow(clippy::new_without_default)]
#![feature(generic_associated_types)]

use serde::de::{EnumAccess, MapAccess, SeqAccess, Visitor};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::fmt::Formatter;
use ::valuable::{Valuable, Value, Visit};

pub mod ctypes;
pub mod errors;
pub mod serializer;
mod widgets;
pub mod deserializer;
pub mod traveller;

struct Hello {
    integer: usize,
}
struct RangedInt<const MIN: i64, const MAX: i64>(pub i64);


impl<const MIN: i64, const MAX: i64> Serialize for RangedInt<MIN, MAX> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where S: Serializer {
        serializer.serialize_i64(self.0)
    }
}

impl<'de, const MIN: i64, const MAX: i64> Deserialize<'de> for RangedInt<MIN, MAX> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_i64(SmallIntVisitor)
    }
}

struct SmallIntVisitor<const MIN: i64, const MAX: i64>;

impl<'de, const MIN: i64, const MAX: i64> Visitor<'_> for SmallIntVisitor<MIN, MAX> {
    type Value = RangedInt<MIN, MAX>;

    fn expecting(&self, formatter: &mut Formatter) -> std::fmt::Result {
        todo!()
    }

    fn visit_i64<E>(self, v: i64) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        if v >= MIN && v <= MAX {
            Ok(RangedInt(v))
        } else {
            Err(E::custom("wrong"))
        }
    }
}
