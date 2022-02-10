#![allow(dead_code)]
#![feature(backtrace)]
#![feature(int_roundings)]
#![feature(box_patterns)]
#![allow(clippy::new_without_default)]
#![feature(generic_associated_types)]
#![feature(adt_const_params)]

pub mod ctypes;
pub mod errors;
pub mod serializer;
pub mod deserializer;
pub mod traveller;
#[cfg(feature = "druid")]
mod druid;