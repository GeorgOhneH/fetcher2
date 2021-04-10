use proc_macro2::{Span, TokenStream};

use quote::quote;
use syn::{self, punctuated::Punctuated, token::Comma, Field, LitStr};

use crate::derives::{convert_type, SupportedTypes};

pub fn gen_parse_fn(fields: &Punctuated<Field, Comma>) -> TokenStream {
    let augmentation = gen_kwargs(fields);
    quote! {
        fn parse_from_app(app: &::config::ConfigStruct) -> Result<Self, ::config::ValueRequiredError> {
            #augmentation
        }
    }
}

fn gen_kwargs(fields: &Punctuated<Field, Comma>) -> TokenStream {
    let keywords: Vec<TokenStream> = fields
        .iter()
        .map(|field| {
            let field_name = field.ident.as_ref().expect("Unreachable");
            quote! {#field_name}
        })
        .collect();
    let args: Vec<TokenStream> = fields
        .iter()
        .map(|field| {
            let field_name = field.ident.as_ref().expect("Unreachable");
            let field_name_str = LitStr::new(&field_name.to_string(), field_name.span());
            let match_arg = quote! {app.get_ty(&#field_name_str.to_string()).unwrap()};
            let typ = convert_type(&field.ty);
            gen_arg(&typ, match_arg, &field_name.span(), &field_name_str)
        })
        .collect();

    quote! {
        #(
            let #keywords = #args;
            if let Err(err) = #keywords {
                return Err(err)
            };
        )*
        Ok(Self {
            #(
                #keywords: #keywords.unwrap(),
            )*
        })
    }
}

fn gen_arg(
    typ: &SupportedTypes,
    match_arg: TokenStream,
    span: &Span,
    field_name_str: &LitStr,
) -> TokenStream {
    let option_arg = gen_option_arg(typ, match_arg, span, field_name_str);
    if !typ.is_inside_option() {
        quote! {match #option_arg {
            Ok(value) => match value {
                Some(x) => Ok(x.clone()),
                None => Err(::config::ValueRequiredError::new(#field_name_str.to_string())),
            },
            Err(err) => Err(::config::ValueRequiredError::add(err, #field_name_str)),
        }}
    } else {
        quote! {
            match #option_arg {
            Ok(value) => match value {
                Some(x) => Ok(Some(x.clone())),
                None => Ok(None),
            },
            Err(err) => Err(::config::ValueRequiredError::add(err, #field_name_str)),
            }
        }
    }
}

fn gen_option_arg(
    typ: &SupportedTypes,
    match_arg: TokenStream,
    span: &Span,
    field_name_str: &LitStr,
) -> TokenStream {
    match typ {
        SupportedTypes::String | SupportedTypes::OtherString => quote! {{
            match #match_arg {
                ::config::SupportedTypes::String(value_arg) => Ok(value_arg.get()),
                _ => panic!("This should never happen"),
            }
        }},
        SupportedTypes::Integer | SupportedTypes::OtherInteger => quote! {{
            match #match_arg {
                ::config::SupportedTypes::Integer(value_arg) => Ok(value_arg.get()),
                _ => panic!("This should never happen"),
            }
        }},
        SupportedTypes::Bool | SupportedTypes::OtherBool => quote! {{
            match #match_arg {
                ::config::SupportedTypes::Bool(value_arg) => Ok(value_arg.get()),
                _ => panic!("This should never happen"),
            }
        }},
        SupportedTypes::Vec(sub_type) => {
            let sub_value = gen_arg(sub_type, quote! {subtype}, span, field_name_str);
            quote! {{
                let a: Result<Vec<#sub_type>, ::config::ValueRequiredError> = match #match_arg {
                    ::config::SupportedTypes::Vec(value_arg) => value_arg
                            .get()
                            .iter()
                            .map(|subtype| {
                                #sub_value
                            })
                            .collect(),
                    _ => panic!("This should never happen"),
                };
                match a {
                    Ok(value) => Ok(Some(value)),
                    Err(err) => Err(err.add("Vec")),
                }
            }}
        }
        SupportedTypes::Struct(ty_path) => {
            let path = &ty_path.path;
            let struct_name_str = LitStr::new(&quote! {#path}.to_string(), span.clone());
            quote! {{
                match #match_arg {
                    ::config::SupportedTypes::Struct(config_struct) => match #ty_path::parse_from_app(config_struct) {
                        Ok(value) => Ok(Some(value)),
                        Err(err) => Err(err.add(#struct_name_str)),
                    },
                    _ => panic!("This should never happen"),
                }
            }}
        }
        SupportedTypes::CheckableStruct(ty_path) => {
            let path = &ty_path.path;
            let struct_name_str = LitStr::new(&quote! {#path}.to_string(), span.clone());
            quote! {{
                match #match_arg {
                    ::config::SupportedTypes::CheckableStruct(config_check_struct) => {
                        if !config_check_struct.is_checked() {
                            Ok(None)
                        } else {
                            match #ty_path::parse_from_app(config_check_struct.get_inner()) {
                                Ok(value) => Ok(Some(value)),
                                Err(err) => Err(err.add(#struct_name_str)),
                            }
                        }
                    },
                    _ => panic!("This should never happen"),
                }
            }}
        }
    }
}
