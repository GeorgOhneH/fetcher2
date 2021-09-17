use crate::config_type::{parse_type, ConfigType};
use proc_macro2::{Ident, TokenStream};
use quote::quote;
use syn::spanned::Spanned;
use syn::{self, punctuated::Punctuated, token::Comma, Field, LitStr};

pub fn gen_field_names(fields: &Punctuated<Field, Comma>) -> Vec<TokenStream> {
    fields
        .iter()
        .filter_map(|field| {
            let typ = parse_type(&field.ty, &field.attrs);
            if let ConfigType::Skip(_) = typ {
                None
            } else {
                Some(gen_field_name(field))
            }
        })
        .collect()
}

pub fn gen_field_name_strs(fields: &Punctuated<Field, Comma>) -> Vec<TokenStream> {
    fields
        .iter()
        .filter_map(|field| {
            let typ = parse_type(&field.ty, &field.attrs);
            if let ConfigType::Skip(_) = typ {
                None
            } else {
                Some(gen_field_name_string(field))
            }
        })
        .collect()
}

pub fn gen_field_name(field: &Field) -> TokenStream {
    let field_name = field.ident.as_ref().expect("Unreachable");
    quote! { #field_name }
}

pub fn gen_field_name_string(field: &Field) -> TokenStream {
    let field_name = field.ident.as_ref().expect("Unreachable");
    let name = LitStr::new(&field_name.to_string(), field.span());
    quote! { #name }
}
