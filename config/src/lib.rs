#![allow(dead_code)]
#![feature(backtrace)]
#![feature(int_roundings)]
#![feature(box_patterns)]

use std::fmt::Debug;

pub use config_derive::{Config, ConfigEnum};

pub use crate::ctypes::*;
pub use crate::errors::*;

mod ctypes;
mod errors;
mod widgets;

pub trait Config: Sized + Send + Debug {
    fn parse_from_app(app: &CStruct) -> Result<Self, RequiredError>;
    fn builder() -> CStructBuilder;
    fn update_app(self, app: &mut CStruct) -> Result<(), InvalidError>;
    fn default() -> Result<Self, RequiredError> {
        Self::parse_from_app(&Self::builder().build())
    }
}

pub trait ConfigEnum: Sized + Send + Debug {
    fn parse_from_app(app: &CEnum) -> Result<Option<Self>, RequiredError>;
    fn builder() -> CEnumBuilder;
    fn update_app(self, app: &mut CEnum) -> Result<(), InvalidError>;
}
