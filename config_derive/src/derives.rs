use proc_macro_error::abort_call_site;

use proc_macro2::TokenStream;
use quote::quote;

use crate::build_app::{gen_enum_build_app_fn, gen_struct_build_app_fn};

use syn::DataEnum;
use syn::{
    self, punctuated::Punctuated, token::Comma, Attribute, Data, DataStruct, DeriveInput, Field,
    Fields, Ident,
};

pub fn derive_config_struct(input: &DeriveInput) -> TokenStream {
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
        Data::Enum(ref e) => abort_call_site!("`#[derive(ConfigEnum)]`"),
        _ => abort_call_site!("`#[derive(Config)]` only supports non-tuple structs and enums"),
    }
}

pub fn derive_config_enum(input: &DeriveInput) -> TokenStream {
    let ident = &input.ident;

    match input.data {
        Data::Struct(DataStruct {
            fields: Fields::Named(ref fields),
            ..
        }) => gen_for_struct(ident, &fields.named, &input.attrs),
        Data::Struct(DataStruct {
            fields: Fields::Unit,
            ..
        }) => abort_call_site!("`#[derive(Config)]`"),
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
    let de_field = crate::deserialize::r#struct::gen_field(fields);
    let de_visitor = crate::deserialize::r#struct::gen_visitor(fields, name);

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

        impl<'de> serde::Deserialize<'de> for #name {
            fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                #de_field

                #de_visitor
            }
        }

    }
}

fn gen_for_enum(name: &Ident, _attrs: &[Attribute], e: &DataEnum) -> TokenStream {
    let build_app_fn = gen_enum_build_app_fn(e);
    let parse_fn = crate::parse_from_app::gen_enum_parse_fn(e);
    let update_app_fn = crate::update_app::gen_enum_update_app_fn(e);
    let de_trait = crate::deserialize::r#enum::gen_de_enum(e, name);

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

        #de_trait

    }
}
