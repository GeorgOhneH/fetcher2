#![allow(dead_code)]
use serde::Serialize;
use std::collections::HashMap;

use std::fmt::Debug;
mod ctypes;
mod errors;
pub use crate::ctypes::*;
pub use crate::errors::*;

pub trait Config: Sized + Clone + Send + Serialize{
    fn parse_from_app(app: &CStruct) -> Result<Self, RequiredError>;
    fn build_app() -> CStruct;
    fn update_app(self, app: &mut CStruct) -> Result<(), InvalidError>;
}

pub trait ConfigEnum: Sized + Clone + Send + Serialize {
    fn parse_from_app(app: &CEnum) -> Result<Option<Self>, RequiredError>;
    fn build_app() -> CEnum;
    fn update_app(self, app: &mut CEnum) -> Result<(), InvalidError>;
}
