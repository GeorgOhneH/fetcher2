#![allow(dead_code)]
#![feature(try_trait_v2)]
#![feature(control_flow_enum)]
#![feature(backtrace)]
#![feature(type_alias_impl_trait)]
#![allow(clippy::new_without_default)]

pub mod template;
pub mod site_modules;
pub mod error;
pub mod session;
pub mod task;
pub mod utils;
pub mod settings;
pub use error::{Result, TError, TErrorKind};
