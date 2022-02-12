#![allow(dead_code)]

use proc_macro::TokenStream;

use proc_macro_error::proc_macro_error;
use syn::{self, parse_macro_input, DeriveInput};

use crate::derives::derive_travel;

mod config_attr;
mod derives;
mod travel;
mod utils;

#[proc_macro_derive(Travel, attributes(travel))]
#[proc_macro_error]
pub fn travel(input: TokenStream) -> TokenStream {
    let input: DeriveInput = parse_macro_input!(input);
    derive_travel(&input).into()
}
