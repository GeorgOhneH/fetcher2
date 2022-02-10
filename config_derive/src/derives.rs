use proc_macro2::TokenStream;
use proc_macro_error::abort_call_site;
use quote::quote;
use syn::spanned::Spanned;
use syn::DataEnum;
use syn::TraitBoundModifier;
use syn::{
    self, punctuated::Punctuated, token::Comma, Attribute, Data, DataStruct, DeriveInput, Field,
    Fields, Generics, Ident, TraitBound,
};

use crate::utils::{bound_generics, create_path, lifetime_generics};

pub fn derive_travel(input: &DeriveInput) -> TokenStream {
    let ident = &input.ident;

    match &input.data {
        Data::Struct(data_struct) => {
            gen_for_struct(ident, &input.generics, &data_struct, &input.attrs)
        }
        Data::Enum(ref e) => gen_for_enum(ident, &input.attrs, e),
        Data::Union(_) => abort_call_site!("`#[derive(Travel)]` doesn't support unions"),
    }
}

fn gen_for_struct(
    name: &Ident,
    name_generics: &Generics,
    data_struct: &DataStruct,
    _attrs: &[Attribute],
) -> TokenStream {
    let travel_path = create_path(&["config", "traveller", "Travel"], name.span());
    let travel_bound = TraitBound {
        paren_token: None,
        modifier: TraitBoundModifier::None,
        lifetimes: None,
        path: travel_path,
    };
    let bounded_travel_generics = bound_generics(name_generics.clone(), travel_bound);

    let travel_impl = match &data_struct.fields {
        Fields::Unit => crate::travel::fields::gen_travel_unit_struct(),
        Fields::Unnamed(fields) => crate::travel::fields::gen_travel_tuple_struct(&fields.unnamed, name),
        Fields::Named(fields) => crate::travel::fields::gen_travel_struct(&fields.named, name),
    };

    quote! {
        impl #bounded_travel_generics ::config::traveller::Travel for #name #name_generics {
            fn travel<__T>(traveller: __T) -> std::result::Result<__T::Ok, __T::Error>
            where
                __T: ::config::traveller::Traveller,
            {
                #travel_impl
            }
        }

    }
}

fn gen_for_enum(name: &Ident, _attrs: &[Attribute], e: &DataEnum) -> TokenStream {
    let travel_impl = crate::travel::variant::gen_travel_enum(e, name);
    quote! {
        impl ::config::traveller::Travel for #name {
            fn travel<__T>(traveller: __T) -> std::result::Result<__T::Ok, __T::Error>
            where
                __T: ::config::traveller::Traveller,
            {
                #travel_impl
            }
        }
    }
}
