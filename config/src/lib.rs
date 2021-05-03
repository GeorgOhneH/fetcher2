#![allow(dead_code)]
use serde::Serialize;
use std::collections::HashMap;

use std::fmt::Debug;
mod ctypes;
mod errors;
pub use crate::ctypes::*;
pub use crate::errors::*;

pub trait Config: Sized + Send + Serialize + Debug {
    fn parse_from_app(app: &CStruct) -> Result<Self, RequiredError>;
    fn build_app() -> CStruct;
    fn update_app(self, app: &mut CStruct) -> Result<(), InvalidError>;

    fn load_from_str(str: &str) -> Result<Self, ConfigError> {
        let mut app = Self::build_app();
        app.load_from_string(str)?;
        let result = Self::parse_from_app(&app)?;
        Ok(result)
    }
}

pub trait ConfigEnum: Sized + Send + Serialize + Debug {
    fn parse_from_app(app: &CEnum) -> Result<Option<Self>, RequiredError>;
    fn build_app() -> CEnum;
    fn update_app(self, app: &mut CEnum) -> Result<(), InvalidError>;
}
