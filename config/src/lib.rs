#![allow(dead_code)]
use std::collections::HashMap;


use std::fmt::Debug;
mod errors;
mod ctypes;
pub use crate::errors::*;
pub use crate::ctypes::*;

pub trait Config: Sized + Clone {
    fn parse_from_app(app: &CStruct) -> Result<Self, RequiredError>;
    fn build_app() -> CStruct;
    fn update_app(self, app: &mut CStruct) -> Result<(), MsgError>;
}

pub trait ConfigEnum: Sized + Clone {
    fn parse_from_app(app: &CEnum) -> Result<Self, RequiredError>;
    fn build_app() -> CEnum;
    fn update_app(self, app: &mut CEnum) -> Result<(), MsgError>;
}
