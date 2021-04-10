#![allow(dead_code)]
use std::collections::HashMap;
use std::error::Error;
use std::fmt;
use std::fmt::Debug;
mod config_arg;
mod errors;
mod supported_types;
pub use crate::config_arg::*;
pub use crate::errors::*;
pub use crate::supported_types::*;

pub trait Config: Sized + Clone {
    fn parse_from_app(app: &ConfigStruct) -> Result<Self, ValueRequiredError>;
    fn build_app() -> ConfigStruct;
    fn update_app(self, app: &mut ConfigStruct) -> Result<(), ValueError>;
}
