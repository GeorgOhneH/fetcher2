use proc_macro_error::abort_call_site;

use proc_macro2::{Span, TokenStream};
use quote::{quote, ToTokens, TokenStreamExt};

use proc_macro_error::abort;

use crate::build_app::{gen_enum_build_app_fn, gen_struct_build_app_fn};
use crate::config_attr::{parse_config_attributes, ConfigAttr};
use syn::{
    self, punctuated::Punctuated, token::Comma, Attribute, Data, DataStruct, DeriveInput, Field,
    Fields, GenericArgument, Ident, LitStr, Path, PathArguments, TypePath,
};
use syn::{DataEnum, Type};

pub fn derive_config(input: &DeriveInput) -> TokenStream {
    let ident = &input.ident;

    match input.data {
        Data::Struct(DataStruct {
            fields: Fields::Named(ref fields),
            ..
        }) => gen_for_struct(ident, &fields.named, &input.attrs),
        Data::Struct(DataStruct {
            fields: Fields::Unit,
            ..
        }) => gen_for_struct(ident, &Punctuated::<Field, Comma>::new(), &input.attrs),
        Data::Enum(ref e) => gen_for_enum(ident, &input.attrs, e),
        _ => abort_call_site!("`#[derive(Config)]` only supports non-tuple structs and enums"),
    }
}

fn gen_for_struct(
    name: &Ident,
    fields: &Punctuated<Field, Comma>,
    _attrs: &[Attribute],
) -> TokenStream {
    let build_app_fn = gen_struct_build_app_fn(fields);
    let parse_fn = crate::parse_from_app::gen_struct_parse_fn(fields);
    let update_app_fn = crate::update_app::gen_struct_update_app_fn(fields);

    quote! {
        #[allow(dead_code, unreachable_code, unused_variables)]
        #[allow(
            clippy::style,
            clippy::complexity,
            clippy::pedantic,
            clippy::restriction,
            clippy::perf,
            clippy::deprecated,
            clippy::nursery,
            clippy::cargo
        )]
        #[deny(clippy::correctness)]
        impl ::config::Config for #name {
            #build_app_fn
            #parse_fn
            #update_app_fn
        }

    }
}

fn gen_for_enum(name: &Ident, _attrs: &[Attribute], e: &DataEnum) -> TokenStream {
    let build_app_fn = gen_enum_build_app_fn(e);
    let parse_fn = crate::parse_from_app::gen_enum_parse_fn(e);
    let update_app_fn = crate::update_app::gen_enum_update_app_fn(e);

    quote! {
        #[allow(dead_code, unreachable_code, unused_variables)]
        #[allow(
            clippy::style,
            clippy::complexity,
            clippy::pedantic,
            clippy::restriction,
            clippy::perf,
            clippy::deprecated,
            clippy::nursery,
            clippy::cargo
        )]
        #[deny(clippy::correctness)]
        impl ::config::ConfigEnum for #name {
            #build_app_fn
            #parse_fn
            #update_app_fn
        }

    }
}
