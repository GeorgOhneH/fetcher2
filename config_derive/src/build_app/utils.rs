use crate::config_attr::ConfigAttr;
use proc_macro2::{Span, TokenStream};

use proc_macro_error::abort;
use quote::{quote, quote_spanned};

use syn::LitStr;

use crate::config_type::{ConfigHashType, ConfigType};

pub fn gen_type(
    typ: &ConfigType,
    config_attrs: &[ConfigAttr],
    span: Span,
    name: Option<&LitStr>,
) -> TokenStream {
    let args = attrs_to_sub_args(config_attrs);
    let gui_fn = if let Some(name) = name {
        quote! {.name(#name.to_string())}
    } else {
        quote! {}
    };
    match typ {
        ConfigType::Bool(_) | ConfigType::OptionBool(_) => quote_spanned! {span=>
            ::config::CType::Bool(
                ::config::CBoolBuilder::new()
                #gui_fn
                #args
                .build()
            )
        },
        ConfigType::String(_) | ConfigType::OptionString(_) => quote_spanned! {span=>
            ::config::CType::String(
                ::config::CStringBuilder::new()
                #gui_fn
                #args
                .build()
            )
        },
        ConfigType::Integer(_) | ConfigType::OptionInteger(_) => quote_spanned! {span=>
            ::config::CType::Integer(
                ::config::CIntegerBuilder::new()
                #gui_fn
                #args
                .build()
            )
        },
        ConfigType::Path(_) | ConfigType::OptionPath(_) => quote_spanned! {span=>
            ::config::CType::Path(
                ::config::CPathBuilder::new()
                #gui_fn
                #args
                .build()
            )
        },
        ConfigType::Vec(_, sub_type) => {
            //emit_call_site_warning!(format!("{:#?}", *sub_type));
            let sub_arg = gen_type(sub_type, config_attrs, span, name);
            quote_spanned! {span=>
                ::config::CType::Vec(
                    ::config::CVecBuilder::new(|| #sub_arg)
                    #gui_fn
                    .build()
                )
            }
        }
        ConfigType::Wrapper(path, inner_ty, _wrapper_ty) => {
            let inner = gen_type(inner_ty, config_attrs, span, name);
            let wrapper_name = path.segments[0].ident.to_owned();
            quote_spanned! {span=>
                ::config::CType::Wrapper(Box::new(
                    ::config::CWrapperBuilder::new(#inner, ::config::CWrapperKind::#wrapper_name)
                    .build()
                ))
            }
        }
        ConfigType::HashMap(_, key_ty, value_ty) => {
            //emit_call_site_warning!(format!("{:#?}", *sub_type));
            let key_arg = gen_hash_type(key_ty, config_attrs, span);
            let value_arg = gen_type(value_ty, config_attrs, span, name);
            quote_spanned! {span=>
                ::config::CType::HashMap(
                    ::config::CHashMapBuilder::new(|| #key_arg, || #value_arg)
                    #gui_fn
                    .build()
                )
            }
        }
        ConfigType::Struct(path) => {
            quote_spanned! {span=>
                ::config::CType::CStruct(
                    #path::builder()
                    #gui_fn
                    #args
                    .build()
                )
            }
        }
        ConfigType::CheckableStruct(path) => {
            quote_spanned! {span=>
                ::config::CType::CheckableStruct(
                    ::config::CCheckableStructBuilder::new(
                        #path::builder()
                    )
                    #gui_fn
                    #args
                    .build()
                )
            }
        }
        ConfigType::Enum(path) | ConfigType::OptionEnum(path) => {
            quote_spanned! {span=>
                ::config::CType::CEnum(
                    #path::builder()
                    #gui_fn
                    #args
                    .build()
                )
            }
        }
        ConfigType::Skip(_) => abort!(span, "Skip shouldn't be a possible value"),
    }
}

pub fn gen_hash_type(
    typ: &ConfigHashType,
    _config_attrs: &[ConfigAttr],
    span: Span,
) -> TokenStream {
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
            ConfigAttr::Name(name, value) => Some(quote! {#name(#value.to_string())}),
            _ => None,
        })
        .collect();

    quote! {#(.#args)*}
}
