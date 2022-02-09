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