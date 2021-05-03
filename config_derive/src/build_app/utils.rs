use crate::config_attr::{parse_config_attributes, ConfigAttr};
use proc_macro2::{TokenStream, Span};

use proc_macro_error::abort;
use quote::{quote, quote_spanned};
use syn::{
    self, punctuated::Punctuated, token::Comma, DataEnum, Field, Fields, FieldsUnnamed, LitStr,
};

use crate::config_type::{parse_type, ConfigHashType, ConfigType};
use syn::spanned::Spanned;



pub fn gen_type(typ: &ConfigType, config_attrs: &[ConfigAttr], span: Span) -> TokenStream {
    let args = attrs_to_sub_args(config_attrs);
    match typ {
        ConfigType::Bool(_) | ConfigType::OptionBool(_) => quote_spanned! {span=>
            ::config::CType::Bool(
                ::config::CBoolBuilder::new()
                #args
                .build()
            )
        },
        ConfigType::String(_) | ConfigType::OptionString(_) => quote_spanned! {span=>
            ::config::CType::String(
                ::config::CStringBuilder::new()
                #args
                .build()
            )
        },
        ConfigType::Integer(_) | ConfigType::OptionInteger(_) => quote_spanned! {span=>
            ::config::CType::Integer(
                ::config::CIntegerBuilder::new()
                #args
                .build()
            )
        },
        ConfigType::Path(_) | ConfigType::OptionPath(_) => quote_spanned! {span=>
            ::config::CType::Path(
                ::config::CPathBuilder::new()
                #args
                .build()
            )
        },
        ConfigType::Vec(_, sub_type) => {
            //emit_call_site_warning!(format!("{:#?}", *sub_type));
            let sub_arg = gen_type(sub_type, config_attrs, span);
            quote_spanned! {span=>
                ::config::CType::Vec(
                    ::config::CVecBuilder::new(|| #sub_arg)
                    .build()
                )
            }
        }
        ConfigType::Wrapper(_, inner_ty, name) => {
            let inner = gen_type(inner_ty, config_attrs, span);
            quote_spanned! {span=>
                ::config::CType::Wrapper(Box::new(
                    ::config::CWrapperBuilder::new(#inner, ::config::CWrapperKind::#name)
                    .build()
                ))
            }
        }
        ConfigType::HashMap(_, key_ty, value_ty) => {
            //emit_call_site_warning!(format!("{:#?}", *sub_type));
            let key_arg = gen_hash_type(key_ty, config_attrs, span);
            let value_arg = gen_type(value_ty, config_attrs, span);
            quote_spanned! {span=>
                ::config::CType::HashMap(
                    ::config::CHashMapBuilder::new(|| #key_arg, || #value_arg)
                    .build()
                )
            }
        }
        ConfigType::Struct(path) => {
            if !args.is_empty() {
                abort!(path, "Sub args are not allowed for ConfigStructs")
            } else {
                quote_spanned! {span=>
                    ::config::CType::Struct(
                        #path::build_app()
                        #args
                    )
                }
            }
        }
        ConfigType::CheckableStruct(path) => {
            quote_spanned! {span=>
                ::config::CType::CheckableStruct(
                    ::config::CCheckableStructBuilder::new(
                        #path::build_app()
                    )
                    #args
                    .build()
                )
            }
        }
        ConfigType::Enum(path) | ConfigType::OptionEnum(path) => {
            if !args.is_empty() {
                abort!(path, "Sub args are not allowed for Enum")
            } else {
                quote_spanned! {span=>
                    ::config::CType::Enum(
                        #path::build_app()
                        #args
                    )
                }
            }
        }
    }
}

pub fn gen_hash_type(typ: &ConfigHashType, config_attrs: &[ConfigAttr], span: Span) -> TokenStream {
    match typ {
        ConfigHashType::String => quote_spanned! {span=>
            ::config::HashKey::String("".to_owned())
        },
        ConfigHashType::Path => quote_spanned! {span=>
            ::config::HashKey::Path(PathBuf::new())
        },
    }
}

pub fn attrs_to_args(config_attrs: &Vec<ConfigAttr>) -> TokenStream {
    use ConfigAttr::*;

    let args: Vec<TokenStream> = config_attrs
        .iter()
        .filter_map(|config_attr| match config_attr {
            GuiName(name, value) => Some(quote! {#name(#value.to_string())}),
            ActiveFn(name, expr) => Some(quote! {#name(#expr)}),
            InactiveBehavior(name, expr) => Some(quote! {#name(#expr)}),
            DocString(str) => Some(quote! {hint_text(#str.to_string())}),
            _ => None,
        })
        .collect();

    quote! {#(.#args)*}
}

pub fn attrs_to_sub_args(config_attrs: &[ConfigAttr]) -> TokenStream {
    let args: Vec<TokenStream> = config_attrs
        .iter()
        .filter_map(|config_attr| match config_attr {
            ConfigAttr::OtherSingle(name) => Some(quote! {#name()}),
            ConfigAttr::OtherLitStr(name, lit) => Some(quote! {#name(#lit.to_string())}),
            ConfigAttr::Other(name, expr) => Some(quote! {#name(#expr)}),
            _ => None,
        })
        .collect();

    quote! {#(.#args)*}
}
