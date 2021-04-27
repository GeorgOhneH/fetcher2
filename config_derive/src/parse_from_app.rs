use proc_macro2::{Span, TokenStream};

use proc_macro_error::abort;
use quote::quote;
use syn::{
    self, punctuated::Punctuated, token::Comma, DataEnum, Field, Fields, FieldsUnnamed, LitStr,
};

use crate::derives::{parse_type, SupportedTypes, HashType};

pub fn gen_struct_parse_fn(fields: &Punctuated<Field, Comma>) -> TokenStream {
    let augmentation = gen_kwargs(fields);
    quote! {
        fn parse_from_app(app: &::config::CStruct) -> Result<Self, ::config::RequiredError> {
            #augmentation
        }
    }
}

pub fn gen_enum_parse_fn(e: &DataEnum) -> TokenStream {
    let augmentation = gen_carg(e);
    quote! {
        fn parse_from_app(cenum: &::config::CEnum) -> Result<Option<Self>, ::config::RequiredError> {
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
                let typ = parse_type(&field.ty, &var.attrs);
                if let SupportedTypes::Struct(_) = typ {
                    quote! {
                        #name_lit => {
                            let config = #typ::parse_from_app(carg.get().unwrap());
                            match config {
                                Ok(s) => Ok(Some(Self::#name(s))),
                                Err(err) => Err(err),
                            }
                        }
                    }
                } else {
                    abort!(var.fields, "Only Structs are allowed")
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
            let typ = parse_type(&field.ty, &field.attrs);
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
                None => Err(::config::RequiredError::new(#field_name_str, "Must be Option?")),
            },
            Err(err) => Err(err),
        }}
    } else {
        quote! {
            match #option_arg {
            Ok(value) => match value {
                Some(x) => Ok(Some(x.clone())),
                None => Ok(None),
            },
            Err(err) => Err(err),
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
        SupportedTypes::String | SupportedTypes::OptionString => quote! {{
            match #match_arg {
                ::config::CTypes::String(value_arg) => Ok(value_arg.get()),
                _ => panic!("This should never happen"),
            }
        }},
        SupportedTypes::Integer | SupportedTypes::OptionInteger => quote! {{
            match #match_arg {
                ::config::CTypes::Integer(value_arg) => Ok(value_arg.get()),
                _ => panic!("This should never happen"),
            }
        }},
        SupportedTypes::Path | SupportedTypes::OptionPath => quote! {{
            match #match_arg {
                ::config::CTypes::Path(cpath) => Ok(cpath.get()),
                _ => panic!("This should never happen"),
            }
        }},
        SupportedTypes::Bool | SupportedTypes::OptionBool => quote! {{
            match #match_arg {
                ::config::CTypes::Bool(value_arg) => Ok(value_arg.get()),
                _ => panic!("This should never happen"),
            }
        }},
        SupportedTypes::Vec(sub_type) => {
            let sub_value = gen_arg(sub_type, quote! {subtype}, span, field_name_str);
            quote! {{
                let a: Result<Vec<#sub_type>, ::config::RequiredError> = match #match_arg {
                    ::config::CTypes::Vec(cvec) => cvec
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
                    Err(err) => Err(err),
                }
            }}
        }
        SupportedTypes::HashMap(key_ty, value_ty) => {
            let real_key = gen_hash_arg(key_ty, quote! {keytype}, span, field_name_str);
            let real_value = gen_arg(value_ty, quote! {valuetype}, span, field_name_str);
            quote! {{
                let a: Result<HashMap<#key_ty, #value_ty>, ::config::RequiredError> = match #match_arg {
                    ::config::CTypes::HashMap(cmap) => cmap
                            .get()
                            .iter()
                            .map(|(keytype, valuetype)| {
                                let x = #real_key;
                                let y = #real_value?;
                                Ok((x, y))
                            })
                            .collect(),
                    _ => panic!("This should never happen"),
                };
                match a {
                    Ok(value) => Ok(Some(value)),
                    Err(err) => Err(err),
                }
            }}
        }
        SupportedTypes::Struct(ty_path) => {
            let path = &ty_path.path;
            let struct_name_str = LitStr::new(&quote! {#path}.to_string(), span.clone());
            quote! {{
                match #match_arg {
                    ::config::CTypes::Struct(config_struct) => match #ty_path::parse_from_app(config_struct) {
                        Ok(value) => Ok(Some(value)),
                        Err(err) => Err(err),
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
                    ::config::CTypes::CheckableStruct(config_check_struct) => {
                        if !config_check_struct.is_checked() {
                            Ok(None)
                        } else {
                            match #ty_path::parse_from_app(config_check_struct.get_inner()) {
                                Ok(value) => Ok(Some(value)),
                                Err(err) => Err(err),
                            }
                        }
                    },
                    _ => panic!("This should never happen"),
                }
            }}
        }
        SupportedTypes::Enum(ty_path) | SupportedTypes::OptionEnum(ty_path) => {
            let path = &ty_path.path;
            let enum_name_str = LitStr::new(&quote! {#path}.to_string(), span.clone());
            quote! {{
                match #match_arg {
                    ::config::CTypes::Enum(cenum) => match #ty_path::parse_from_app(cenum) {
                        Ok(value) => Ok(value),
                        Err(err) => Err(err),
                    },
                    _ => panic!("This should never happen"),
                }
            }}
        }
    }
}


fn gen_hash_arg(
    typ: &HashType,
    match_arg: TokenStream,
    span: &Span,
    field_name_str: &LitStr,
) -> TokenStream {
    match typ {
        HashType::String => quote! {{
            match #match_arg {
                ::config::HashKey::String(str) => str.clone(),
                _ => panic!("This should never happen"),
            }
        }},
        HashType::Path => quote! {{
            match #match_arg {
                ::config::HashKey::Path(path) => path.clone(),
                _ => panic!("This should never happen"),
            }
        }},
    }
}