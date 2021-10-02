use proc_macro2::{Ident, TokenStream};
use proc_macro_error::abort;
use quote::quote;
use syn::{self, DataEnum, Fields, FieldsUnnamed, LitStr};

pub fn gen_se_enum(e: &DataEnum, enum_name: &Ident) -> TokenStream {
    let enum_name_str = LitStr::new(&enum_name.to_string(), enum_name.span());
    let match_enums = e.variants.iter().enumerate().map(|(i, var)| {
        let name = &var.ident;
        let count = i as u32;
        let name_str = LitStr::new(&name.to_string(), name.span());
        match &var.fields {
            Fields::Unnamed(FieldsUnnamed { unnamed, .. }) if unnamed.len() == 1 => {
                quote! {
                    Self::#name(ref var) => serializer.serialize_newtype_variant(#enum_name_str, #count, #name_str, var),
                }
            }
            Fields::Unit => {
                quote! {
                    Self::#name => serializer.serialize_unit_variant(#enum_name_str, #count, #name_str),
                }
            }
            _ => abort!(var.fields, "Only Unit and Single Tuple Enums are allowed"),
        }
    });

    quote! {
        impl serde::Serialize for #enum_name {
        fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
            where
                S: serde::Serializer,
        {
            match *self {
                #(#match_enums)*
            }
        }
    }
    }
}
