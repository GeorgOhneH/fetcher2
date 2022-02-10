use crate::config_attr::parse_config_attributes;
use proc_macro2::{Ident, TokenStream};
use proc_macro_error::abort;
use quote::{format_ident, quote};
use regex::internal::Input;
use syn::punctuated::Punctuated;
use syn::spanned::Spanned;
use syn::Field;
use syn::LitStr;
use syn::{self, token::Comma};

pub fn gen_travel_unit_struct() -> TokenStream {
    quote! { traveller.found_unit_struct() }
}

pub fn gen_travel_struct(fields: &Punctuated<Field, Comma>, name: &Ident) -> TokenStream {
    let state_name = format_ident!("state",);
    let gen_founds = gen_found_fields(fields, &state_name);
    let str_name = LitStr::new(&name.to_string(), name.span());

    quote! {
        use ::config::traveller::TravellerStruct as _;
        let mut #state_name = traveller.found_struct(#str_name)?;
        #(#gen_founds)*
        #state_name.end()
    }
}

pub fn gen_travel_tuple_struct(fields: &Punctuated<Field, Comma>, name: &Ident) -> TokenStream {
    let str_name = LitStr::new(&name.to_string(), name.span());

    if fields.len() == 1 {
        let field = &fields[0];
        let field_ty = &field.ty;
        let attr = parse_config_attributes(&field.attrs);
        if let Some(skip) = &attr.skip {
            abort!(skip, "bro, What do you want me to do?")
        }
        if let Some((_, default_expr)) = attr.default {
            quote! { traveller.found_newtype_struct_with_default::<#field_ty>(#str_name, #default_expr) }
        } else {
            quote! { traveller.found_newtype_struct::<#field_ty>(#str_name) }
        }
    } else {
        let state_name = format_ident!("state",);
        let gen_founds = gen_found_fields(fields, &state_name);

        quote! {
            use ::config::traveller::TravellerTuple as _;
            let mut #state_name = traveller.found_tuple_struct(#str_name)?;
            #(#gen_founds)*
            #state_name.end()
        }
    }
}

pub fn gen_found_fields(fields: &Punctuated<Field, Comma>, state_name: &Ident) -> Vec<TokenStream> {
    fields
        .iter()
        .filter_map(|field| {
            let attr = parse_config_attributes(&field.attrs);
            if attr.skip.is_some() {
                return None;
            }
            let field_ty = &field.ty;
            let token_stream = if let Some(field_name) = field.ident.as_ref() {
                let name = LitStr::new(&field_name.to_string(), field.span());
                if let Some((_, default_expr)) = attr.default {
                    quote! { #state_name.found_field_with_default::<#field_ty>(#name, #default_expr)?; }
                } else {
                    quote! { #state_name.found_field::<#field_ty>(#name)?; }
                }
            } else if let Some((_, default_expr)) = attr.default {
                quote! { #state_name.found_element_with_default::<#field_ty>(#default_expr)?; }
            } else {
                quote! { #state_name.found_element::<#field_ty>()?; }
            };
            Some(token_stream)
        })
        .collect()
}
