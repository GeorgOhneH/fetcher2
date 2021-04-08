use crate::parse::{parse_clap_attributes, ConfigAttr};
use proc_macro2::Span;
use proc_macro2::TokenStream;
use proc_macro_error::abort_call_site;
use proc_macro_error::proc_macro_error;
use proc_macro_error::{abort, emit_call_site_warning};
use quote::{quote, quote_spanned};
use syn::spanned::Spanned;
use syn::{
    self, parse_macro_input, punctuated::Punctuated, token::Comma, Attribute, Data, DataEnum,
    DataStruct, DeriveInput, Field, Fields, GenericArgument, Ident, LitStr, NestedMeta, Path,
    PathArguments, TypePath,
};
use syn::{AttributeArgs, Meta, Type};
use crate::derives::{SupportedTypes, convert_type};

pub fn gen_build_app_fn(fields: &Punctuated<Field, Comma>) -> TokenStream {
    let augmentation = gen_app_augmentation(fields);
    quote! {
        fn build_app() -> ::config::ConfigStruct {
            let mut app = ::config::ConfigStruct::new();
            #augmentation
            app
        }
    }
}

pub fn gen_app_augmentation(fields: &Punctuated<Field, Comma>) -> TokenStream {
    let data_expanded_members = fields.iter().map(|field| {
        let typ = convert_type(&field.ty);
        gen_arg(field, &typ)
    });

    quote! {
        #(app.arg(#data_expanded_members);)*
    }
}

fn gen_arg(field: &Field, typ: &SupportedTypes) -> TokenStream {
    let field_name = field.ident.as_ref().expect("Unreachable");
    let span = field_name.span();

    if let SupportedTypes::Option(sub_typ) = typ {
        //emit_call_site_warning!(format!("{:#?}", *sub_type));
        let arg = gen_arg(&field, &sub_typ);
        quote_spanned! {span=>
            #arg.required(false)
        }
    } else {
        let config_attrs = parse_clap_attributes(&field.attrs);
        let builder_args = attrs_to_args(&config_attrs);
        let sup_type = gen_type(field, typ, &config_attrs);
        let name = LitStr::new(&field_name.to_string(), span);
        quote_spanned! {span=>
            ConfigArg::new(
                #name.to_string(),
                #sup_type
            )
            #builder_args
        }
    }
}

fn gen_type(field: &Field, typ: &SupportedTypes, config_attrs: &Vec<ConfigAttr>) -> TokenStream {
    let field_name = field.ident.as_ref().expect("Unreachable");
    let span = field_name.span();
    let args = attrs_to_sub_args(config_attrs);
    match typ {
        SupportedTypes::Bool => quote_spanned! {span=>
            ::config::SupportedTypes::Bool(
                ::config::ConfigArgBool::new()
                #args
            )
        },
        SupportedTypes::String => quote_spanned! {span=>
            ::config::SupportedTypes::String(
                ::config::ConfigArgString::new()
                #args
            )
        },
        SupportedTypes::Integer => quote_spanned! {span=>
            ::config::SupportedTypes::Integer(
                ::config::ConfigArgInteger::new()
                #args
            )
        },
        SupportedTypes::Vec(sub_type) => {
            //emit_call_site_warning!(format!("{:#?}", *sub_type));
            let sub_arg = gen_type(field, sub_type, config_attrs);
            quote_spanned! {span=>
                ::config::SupportedTypes::Vec(
                    Box::new(
                        ::config::ConfigVec::new(#sub_arg)
                    )
                )
            }
        }
        SupportedTypes::Other(ty) => {
            if !args.is_empty() {
                abort!(ty, "Sub args are not allowed for ConfigStructs")
            } else {
                quote_spanned! {span=>
                    ::config::SupportedTypes::Struct(Box::new(#ty::build_app()))
                }
            }
        }
        SupportedTypes::Option(_) => abort!(span, "This should not happen"),
        SupportedTypes::None(ty) => abort!(&ty, "Not Supported"),
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
