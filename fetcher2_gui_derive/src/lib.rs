#![allow(dead_code)]

mod derives;

use proc_macro::TokenStream;

use crate::derives::{derive_node, derive_root_node};
use proc_macro_error::proc_macro_error;
use quote::quote;
use syn::{self, parse_macro_input, DeriveInput};

#[proc_macro_derive(TreeNode, attributes(node))]
#[proc_macro_error]
pub fn tree_node(input: TokenStream) -> TokenStream {
    let input: DeriveInput = parse_macro_input!(input);
    derive_node(&input).into()
}

#[proc_macro_derive(TreeNodeRoot, attributes(node))]
#[proc_macro_error]
pub fn tree_node_root(input: TokenStream) -> TokenStream {
    let input: DeriveInput = parse_macro_input!(input);
    derive_root_node(&input).into()
}
