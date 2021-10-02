use proc_macro2::TokenStream;
use proc_macro_error::abort_call_site;
use quote::quote;
use syn::{
    self, Attribute, Data, DataStruct, DeriveInput, Field, Fields, Generics,
    Ident, punctuated::Punctuated, token::Comma, TraitBound,
};
use syn::{DataEnum};
use syn::spanned::Spanned;
use syn::TraitBoundModifier;

use crate::build_app::{gen_enum_build_app_fn, gen_struct_build_app_fn};
use crate::utils::{bound_generics, create_path, lifetime_generics};

pub fn derive_config_struct(input: &DeriveInput) -> TokenStream {
    let ident = &input.ident;

    match input.data {
        Data::Struct(DataStruct {
            fields: Fields::Named(ref fields),
            ..
        }) => gen_for_struct(ident, &input.generics, &fields.named, &input.attrs),
        Data::Struct(DataStruct {
            fields: Fields::Unit,
            ..
        }) => gen_for_struct(
            ident,
            &input.generics,
            &Punctuated::<Field, Comma>::new(),
            &input.attrs,
        ),
        Data::Enum(ref _e) => abort_call_site!("`#[derive(ConfigEnum)]`"),
        _ => abort_call_site!("`#[derive(Config)]` only supports non-tuple structs and enums"),
    }
}

pub fn derive_config_enum(input: &DeriveInput) -> TokenStream {
    let ident = &input.ident;

    match input.data {
        Data::Struct(_) => abort_call_site!("`#[derive(Config)]`"),
        Data::Enum(ref e) => gen_for_enum(ident, &input.attrs, e),
        _ => abort_call_site!("`#[derive(Config)]` only supports non-tuple structs and enums"),
    }
}

fn gen_for_struct(
    name: &Ident,
    name_generics: &Generics,
    fields: &Punctuated<Field, Comma>,
    _attrs: &[Attribute],
) -> TokenStream {
    let config_path = create_path(&[("config", None), ("Config", None)], name.span());
    let config_bound = TraitBound {
        paren_token: None,
        modifier: TraitBoundModifier::None,
        lifetimes: None,
        path: config_path,
    };
    let se_path = create_path(&[("serde", None), ("Serialize", None)], name.span());
    let se_bound = TraitBound {
        paren_token: None,
        modifier: TraitBoundModifier::None,
        lifetimes: None,
        path: se_path,
    };
    let bounded_config_generics = bound_generics(name_generics.clone(), config_bound.clone());
    let bounded_se_generics = bound_generics(name_generics.clone(), se_bound.clone());
    let de_generics = lifetime_generics(name_generics.clone(), "'de");
    let de_generics = bound_generics(de_generics, config_bound);
    let de_path = create_path(
        &[("serde", None), (&"Deserilize", Some("'de"))],
        name_generics.span(),
    );
    let de_bound = TraitBound {
        paren_token: None,
        modifier: TraitBoundModifier::None,
        lifetimes: None,
        path: de_path,
    };
    let de_generics = bound_generics(de_generics, de_bound);


    let build_app_fn = gen_struct_build_app_fn(fields);
    let parse_fn = crate::parse_from_app::gen_struct_parse_fn(fields);
    let update_app_fn = crate::update_app::gen_struct_update_app_fn(fields);
    let de_field = crate::deserialize::r#struct::gen_field(fields);
    let de_visitor = crate::deserialize::r#struct::gen_visitor(fields, name, name_generics);

    let se_impl = crate::serialize::r#struct::gen_se(fields, name);

    quote! {
        impl #bounded_config_generics ::config::Config for #name #name_generics {
            #build_app_fn
            #parse_fn
            #update_app_fn
        }


        impl #bounded_se_generics serde::Serialize for #name #name_generics {
            fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
            where
                S: serde::Serializer,
            {
                #se_impl
            }
        }

        impl #de_generics serde::Deserialize<'de> for #name #name_generics {
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
    let de_impl = crate::deserialize::r#enum::gen_de_enum(e, name);
    let se_impl = crate::serialize::r#enum::gen_se_enum(e, name);

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

        #se_impl

        #de_impl

    }
}
