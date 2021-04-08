#![allow(dead_code)]
extern crate proc_macro;

use proc_macro::TokenStream;
use proc_macro_error::proc_macro_error;
use quote::quote;
use syn::{
    self, parse_macro_input, punctuated::Punctuated, token::Comma, Attribute, Data, DataEnum,
    DataStruct, DeriveInput, Field, Fields, Ident,
};
mod derives;
mod parse;
mod build_app;

use crate::derives::derive_config;

#[proc_macro_derive(Config, attributes(config))]
#[proc_macro_error]
pub fn config(input: TokenStream) -> TokenStream {
    let input: DeriveInput = parse_macro_input!(input);
    derive_config(&input).into()
}
