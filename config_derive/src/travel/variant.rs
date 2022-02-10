use crate::travel::fields::gen_found_fields;
use proc_macro2::{Ident, TokenStream};
use proc_macro_error::abort;
use quote::{format_ident, quote};
use syn::punctuated::Punctuated;
use syn::token::Comma;
use syn::{self, DataEnum, Field, Fields, FieldsNamed, FieldsUnnamed, LitStr};

pub fn gen_travel_enum(e: &DataEnum, enum_name: &Ident) -> TokenStream {
    let enum_name_str = LitStr::new(&enum_name.to_string(), enum_name.span());
    let gen_found_variants = e.variants.iter().enumerate().map(|(i, var)| {
        let name = &var.ident;
        let name_str = LitStr::new(&name.to_string(), name.span());
        match &var.fields {
            Fields::Unit => {
                quote! {
                    state.found_unit_variant(#name_str)?;
                }
            }
            Fields::Unnamed(FieldsUnnamed { unnamed, .. }) if unnamed.len() == 1 => {
                let field_ty = &unnamed[0].ty;
                quote! {
                    state.found_newtype_variant::<#field_ty>(#name_str)?;
                }
            }
            Fields::Unnamed(FieldsUnnamed { unnamed, .. }) => {
                let state_name = format_ident!("state{}", i);
                let gen_founds = gen_found_fields(unnamed, &state_name);
                quote! {
                    let mut #state_name = state.found_tuple_variant(#name_str)?;
                    #(#gen_founds)*
                    #state_name.end()?;
                }
            }
            Fields::Named(FieldsNamed { named, .. }) => {
                let state_name = format_ident!("state{}", i);
                let gen_founds = gen_found_fields(named, &state_name);
                quote! {
                    let mut #state_name = state.found_struct_variant(#name_str)?;
                    #(#gen_founds)*
                    #state_name.end()?;
                }
            }
        }
    });

    quote! {
        use ::config::traveller::TravellerStructVariant as _;
        use ::config::traveller::TravellerTupleVariant as _;
        let mut state = traveller.found_enum(#enum_name_str)?;
        #(#gen_found_variants)*
        state.end()
    }
}
