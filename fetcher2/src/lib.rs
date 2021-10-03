#![allow(dead_code)]
#![feature(try_trait_v2)]
#![feature(control_flow_enum)]
#![feature(backtrace)]
#![feature(type_alias_impl_trait)]
#![allow(clippy::new_without_default)]

pub use error::{Result, TError, TErrorKind};

pub mod error;
pub mod session;
pub mod settings;
pub mod site_modules;
pub mod task;
pub mod template;
pub mod utils;
