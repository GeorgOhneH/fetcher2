#![allow(dead_code)]
extern crate proc_macro;

use proc_macro::TokenStream;
use proc_macro_error::proc_macro_error;

use syn::{self, parse_macro_input, DeriveInput};
mod build_app;
mod derives;
mod config_attr;
mod parse_from_app;
mod update_app;

use crate::derives::derive_config;

#[proc_macro_derive(Config, attributes(config))]
#[proc_macro_error]
pub fn config(input: TokenStream) -> TokenStream {
    let input: DeriveInput = parse_macro_input!(input);
    derive_config(&input).into()
}
