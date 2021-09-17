#![allow(dead_code)]

use proc_macro::TokenStream;

use proc_macro_error::proc_macro_error;
use syn::{self, DeriveInput, parse_macro_input};

use crate::derives::{derive_config_enum, derive_config_struct};

mod build_app;
mod config_attr;
mod config_type;
mod derives;
mod deserialize;
mod parse_from_app;
mod update_app;
mod serialize;
mod utils;

#[proc_macro_derive(Config, attributes(config))]
#[proc_macro_error]
pub fn config(input: TokenStream) -> TokenStream {
    let input: DeriveInput = parse_macro_input!(input);
    derive_config_struct(&input).into()
}

#[proc_macro_derive(ConfigEnum, attributes(config))]
#[proc_macro_error]
pub fn config_enum(input: TokenStream) -> TokenStream {
    let input: DeriveInput = parse_macro_input!(input);
    derive_config_enum(&input).into()
}
