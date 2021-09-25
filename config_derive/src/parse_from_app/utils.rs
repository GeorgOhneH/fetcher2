use proc_macro2::{Span, TokenStream};
use proc_macro_error::abort;
use quote::quote;
use syn::{self, LitStr};

use crate::config_type::{ConfigHashType, ConfigType};

pub fn gen_arg(
    typ: &ConfigType,
    match_arg: TokenStream,
    span: Span,
    field_name: &LitStr,
) -> TokenStream {
    if let ConfigType::Skip(expr) = typ {
        return quote! {
            Ok(#expr)
        };
    }
    let option_arg = gen_option_arg(typ, match_arg, span, field_name);
    match &typ {
        ConfigType::Wrapper(_, _, _) => option_arg,
        _ if typ.is_inside_option() => {
            quote! {
                match #option_arg {
                Ok(value) => match value {
                    Some(x) => Ok(Some(x)),
                    None => Ok(None),
                },
                Err(err) => Err(err),
                }
            }
        }
        _ => {
            quote! {match #option_arg {
                Ok(value) => match value {
                    Some(x) => Ok(x),
                    None => Err(::config::RequiredError::new(#field_name, "Must be Option?")),
                },
                Err(err) => Err(err),
            }}
        }
    }
}

pub fn gen_option_arg(
    typ: &ConfigType,
    match_arg: TokenStream,
    span: Span,
    field_name: &LitStr,
) -> TokenStream {
    match typ {
        ConfigType::String(_) | ConfigType::OptionString(_) => quote! {{
            match #match_arg {
                ::config::CType::String(value_arg) => Ok(value_arg.get().map(|x|x.clone())),
                _ => panic!("This should never happen"),
            }
        }},
        ConfigType::Integer(_) | ConfigType::OptionInteger(_) => quote! {{
            match #match_arg {
                ::config::CType::Integer(value_arg) => Ok(value_arg.get().map(|x|x.clone())),
                _ => panic!("This should never happen"),
            }
        }},
        ConfigType::Float(_) | ConfigType::OptionFloat(_) => quote! {{
            match #match_arg {
                ::config::CType::Float(value_arg) => Ok(value_arg.get().map(|x|x.clone())),
                _ => panic!("This should never happen"),
            }
        }},
        ConfigType::Path(_) | ConfigType::OptionPath(_) => quote! {{
            match #match_arg {
                ::config::CType::Path(cpath) => Ok(cpath.get().map(|x|x.clone())),
                _ => panic!("This should never happen"),
            }
        }},
        ConfigType::Bool(_) | ConfigType::OptionBool(_) => quote! {{
            match #match_arg {
                ::config::CType::Bool(value_arg) => Ok(Some(value_arg.get())),
                _ => panic!("This should never happen"),
            }
        }},
        ConfigType::Wrapper(path, inner_ty, _wrapper_ty) => {
            let inner_token = gen_arg(inner_ty, quote! {inner}, span, field_name);
            let wrapper_name = path.segments[0].ident.to_owned();
            quote! {{
                match #match_arg {
                    ::config::CType::Wrapper(cwrapper) => {
                        let inner = cwrapper.inner();
                        let x = #inner_token?;
                        Ok(#wrapper_name::new(x))
                    },
                    _ => panic!("This should never happen"),
                }
            }}
        }
        ConfigType::Vec(path, sub_type) => {
            let sub_value = gen_arg(sub_type, quote! {subtype}, span, field_name);
            quote! {{
                let a: std::result::Result<#path, ::config::RequiredError> = match #match_arg {
                    ::config::CType::Vec(cvec) => cvec
                            .get()
                            .iter()
                            .map(|citem| {
                                let subtype = &citem.ty;
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
        ConfigType::HashMap(path, key_ty, value_ty) => {
            let real_key = gen_hash_arg(key_ty, quote! {keytype}, span);
            let real_value = gen_arg(value_ty, quote! {valuetype}, span, field_name);
            quote! {{
                let a: std::result::Result<#path, ::config::RequiredError> = match #match_arg {
                    ::config::CType::HashMap(cmap) => cmap
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
        ConfigType::Struct(path) => {
            let name = &path.segments.iter().next().unwrap().ident;
            quote! {{
                match #match_arg {
                    ::config::CType::CStruct(config_struct) => match #name::parse_from_app(config_struct) {
                        Ok(value) => Ok(Some(value)),
                        Err(err) => Err(err),
                    },
                    _ => panic!("This should never happen"),
                }
            }}
        }
        ConfigType::CheckableStruct(path) => {
            let name = &path.segments.iter().next().unwrap().ident;
            quote! {{
                match #match_arg {
                    ::config::CType::CheckableStruct(config_check_struct) => {
                        if !config_check_struct.is_checked() {
                            Ok(None)
                        } else {
                            match #name::parse_from_app(config_check_struct.get_inner()) {
                                Ok(value) => Ok(Some(value)),
                                Err(err) => Err(err),
                            }
                        }
                    },
                    _ => panic!("This should never happen"),
                }
            }}
        }
        ConfigType::Enum(path) | ConfigType::OptionEnum(path) => {
            let _enum_name_str = LitStr::new(&quote! {#path}.to_string(), span.clone());
            quote! {{
                match #match_arg {
                    ::config::CType::CEnum(cenum) => #path::parse_from_app(cenum),
                    _ => panic!("This should never happen"),
                }
            }}
        }
        ConfigType::Skip(_) => abort!(span, "Skip shouldn't be a possible value"),
    }
}

pub fn gen_hash_arg(typ: &ConfigHashType, match_arg: TokenStream, _span: Span) -> TokenStream {
    match typ {
        ConfigHashType::String => quote! {{
            match #match_arg {
                ::config::HashKey::String(str) => str.clone(),
                _ => panic!("This should never happen"),
            }
        }},
        ConfigHashType::Path => quote! {{
            match #match_arg {
                ::config::HashKey::Path(path) => path.clone(),
                _ => panic!("This should never happen"),
            }
        }},
    }
}
