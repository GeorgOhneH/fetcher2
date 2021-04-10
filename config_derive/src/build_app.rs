use crate::config_attr::{parse_clap_attributes, ConfigAttr};

use proc_macro2::TokenStream;

use proc_macro_error::abort;
use quote::{quote, quote_spanned};
use syn::{self, punctuated::Punctuated, token::Comma, Field, LitStr};

use crate::derives::{convert_type, SupportedTypes};

pub fn gen_build_app_fn(fields: &Punctuated<Field, Comma>) -> TokenStream {
    let augmentation = gen_app_augmentation(fields);
    quote! {
        fn build_app() -> ::config::ConfigStruct {
            ::config::ConfigStructBuilder::new()
            #augmentation
            .build()
        }
    }
}

pub fn gen_app_augmentation(fields: &Punctuated<Field, Comma>) -> TokenStream {
    let data_expanded_members = fields.iter().map(|field| {
        let typ = convert_type(&field.ty);
        gen_arg(field, &typ)
    });

    quote! {
        #(.arg(#data_expanded_members))*
    }
}

fn gen_arg(field: &Field, typ: &SupportedTypes) -> TokenStream {
    let field_name = field.ident.as_ref().expect("Unreachable");
    let span = field_name.span();
    let config_attrs = parse_clap_attributes(&field.attrs);
    let builder_args = attrs_to_args(&config_attrs);
    let sup_type = gen_type(field, typ, &config_attrs);
    let name = LitStr::new(&field_name.to_string(), span);
    let is_required = typ.is_inside_option();
    quote_spanned! {span=>
        ::config::ConfigArgBuilder::new(
            #name.to_string(),
            #sup_type
        )
        .required(#is_required)
        #builder_args
        .build()
    }
}

fn gen_type(field: &Field, typ: &SupportedTypes, config_attrs: &Vec<ConfigAttr>) -> TokenStream {
    let field_name = field.ident.as_ref().expect("Unreachable");
    let span = field_name.span();
    let args = attrs_to_sub_args(config_attrs);
    match typ {
        SupportedTypes::Bool | SupportedTypes::OtherBool => quote_spanned! {span=>
            ::config::SupportedTypes::Bool(
                ::config::ConfigArgBoolBuilder::new()
                #args
                .build()
            )
        },
        SupportedTypes::String | SupportedTypes::OtherString => quote_spanned! {span=>
            ::config::SupportedTypes::String(
                ::config::ConfigArgStringBuilder::new()
                #args
                .build()
            )
        },
        SupportedTypes::Integer | SupportedTypes::OtherInteger => quote_spanned! {span=>
            ::config::SupportedTypes::Integer(
                ::config::ConfigArgIntegerBuilder::new()
                #args
                .build()
            )
        },
        SupportedTypes::Vec(sub_type) => {
            //emit_call_site_warning!(format!("{:#?}", *sub_type));
            let sub_arg = gen_type(field, sub_type, config_attrs);
            quote_spanned! {span=>
                ::config::SupportedTypes::Vec(
                    Box::new(
                        ::config::ConfigVecBuilder::new(#sub_arg)
                        .build()
                    )
                )
            }
        }
        SupportedTypes::Struct(ty) => {
            if !args.is_empty() {
                abort!(ty, "Sub args are not allowed for ConfigStructs")
            } else {
                quote_spanned! {span=>
                    ::config::SupportedTypes::Struct(Box::new(
                        #ty::build_app()
                        #args
                    ))
                }
            }
        }
        SupportedTypes::CheckableStruct(ty) => {
            quote_spanned! {span=>
                ::config::SupportedTypes::CheckableStruct(Box::new(
                    ::config::ConfigCheckableStructBuilder::new(
                        #ty::build_app()
                    )
                    #args
                    .build()
                ))
            }
        }
    }
}

fn attrs_to_sub_args(config_attrs: &Vec<ConfigAttr>) -> TokenStream {
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
fn attrs_to_args(config_attrs: &Vec<ConfigAttr>) -> TokenStream {
    let args: Vec<TokenStream> = config_attrs
        .iter()
        .filter_map(|config_attr| match config_attr {
            ConfigAttr::GuiName(name, value) => Some(quote! {#name(#value.to_string())}),
            ConfigAttr::ActiveFn(name, expr) => Some(quote! {#name(#expr)}),
            ConfigAttr::InactiveBehavior(name, expr) => Some(quote! {#name(#expr)}),
            _ => None,
        })
        .collect();

    quote! {#(.#args)*}
}
