use proc_macro2::TokenStream;
use proc_macro_error::abort;
use quote::quote;
use syn::{self, spanned::Spanned, DataEnum, Fields, FieldsUnnamed, LitStr};

use crate::config_type::parse_type;
use crate::parse_from_app::utils::gen_arg;

pub fn gen_enum_parse_fn(e: &DataEnum) -> TokenStream {
    let augmentation = gen_carg(e);
    quote! {
        fn parse_from_app(cenum: &::config::CEnum) -> std::result::Result<Option<Self>, ::config::RequiredError> {
            let selected = cenum.get_selected();
            match selected {
                Some(carg) => {
                    #augmentation
                },
                None => Ok(None),
            }
        }
    }
}

fn gen_carg(e: &DataEnum) -> TokenStream {
    let data_expanded_members = e.variants.iter().map(|var| {
        let name = &var.ident;
        let name_lit = LitStr::new(&name.to_string(), var.ident.span());
        match &var.fields {
            Fields::Unnamed(FieldsUnnamed { unnamed, .. }) if unnamed.len() == 1 => {
                let field = &unnamed[0];
                let config_type = parse_type(&field.ty, &var.attrs);
                let config = gen_arg(
                    &config_type,
                    quote! {carg.get().unwrap()},
                    field.span(),
                    &name_lit,
                );
                quote! {
                    #name_lit => {
                        match #config {
                            Ok(s) => Ok(Some(Self::#name(s))),
                            Err(err) => Err(err),
                        }
                    }
                }
            }
            Fields::Unit => {
                quote! {
                    #name_lit => Ok(Some(Self::#name))
                }
            }
            _ => abort!(var.fields, "Only Structs are allowed"),
        }
    });

    quote! {
        match carg.name().as_str() {
            #(#data_expanded_members,)*
            _ => panic!("Should never happen"),
        }
    }
}

// fn gen_value_arg(config_type: &ConfigType) -> TokenStream {
//     match config_type {
//         ConfigType::Struct(path) => {
//             quote! {
//                 #name_lit => {
//                     let config = #path::parse_from_app(carg.get().unwrap());
//                     match config {
//                         Ok(s) => Ok(Some(Self::#name(s))),
//                         Err(err) => Err(err),
//                     }
//                 }
//             }
//         }
//         ConfigType::Wrapper(path, inner_type, name) => {
//             let inner = gen_value_arg(inner_type);
//             quote! { #name::new(#inner) }
//         }
//         _ => abort!(var.fields, "Only Structs/Wrappers are allowed"),
//     }
// }
