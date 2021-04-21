use crate::config_attr::{parse_config_attributes, ConfigAttr};

use proc_macro2::TokenStream;

use proc_macro_error::abort;
use quote::{quote, quote_spanned};
use syn::{
    self, punctuated::Punctuated, token::Comma, DataEnum, Field, Fields, FieldsUnnamed, LitStr,
};

use crate::derives::{parse_type, SupportedTypes};
use syn::spanned::Spanned;

pub fn gen_struct_build_app_fn(fields: &Punctuated<Field, Comma>) -> TokenStream {
    let augmentation = gen_app_augmentation(fields);
    quote! {
        fn build_app() -> ::config::CStruct {
            ::config::CStructBuilder::new()
            #augmentation
            .build()
        }
    }
}

fn gen_app_augmentation(fields: &Punctuated<Field, Comma>) -> TokenStream {
    let data_expanded_members = fields.iter().map(|field| {
        let typ = parse_type(&field.ty, &field.attrs);
        gen_arg(field, &typ)
    });

    quote! {
        #(.arg(#data_expanded_members))*
    }
}

fn gen_arg(field: &Field, typ: &SupportedTypes) -> TokenStream {
    let field_name = field.ident.as_ref().expect("Unreachable");
    let span = field_name.span();
    let config_attrs = parse_config_attributes(&field.attrs);
    let builder_args = attrs_to_args(&config_attrs);
    let sup_type = gen_type(field, typ, &config_attrs);
    let name = LitStr::new(&field_name.to_string(), span);
    let is_required = typ.is_inside_option();
    quote_spanned! {span=>
        ::config::CKwargBuilder::new(
            #name.to_string(),
            #sup_type
        )
        .required(#is_required)
        #builder_args
        .build()
    }
}

pub fn gen_enum_build_app_fn(e: &DataEnum) -> TokenStream {
    let augmentation = gen_enum_augmentation(e);
    quote! {
        fn build_app() -> ::config::CEnum {
            ::config::CEnumBuilder::new()
            #augmentation
            .build()
        }
    }
}

fn gen_enum_augmentation(e: &DataEnum) -> TokenStream {
    let data_expanded_members = e.variants.iter().map(|var| {
        let name = LitStr::new(&var.ident.to_string(), var.ident.span());
        let struct_arg = match &var.fields {
            Fields::Unnamed(FieldsUnnamed { unnamed, .. }) if unnamed.len() == 1 => {
                let field = &unnamed[0];
                let typ = parse_type(&field.ty, &var.attrs);
                if let SupportedTypes::Struct(_) = typ {
                    quote! {
                    .value(#typ::build_app())
                        }
                } else {
                    abort!(var.fields, "Only Structs are allowed")
                }
            }
            Fields::Unit => {
                quote! {}
            }
            _ => abort!(var.fields, "Only Structs are allowed"),
        };

        quote_spanned! {var.span()=>
            ::config::CArgBuilder::new(
                #name.to_string(),
            )
            #struct_arg
            .build()
        }
    });

    quote! {
        #(.arg(#data_expanded_members))*
    }
}

fn gen_enum_arg(field: &Field, typ: &SupportedTypes) -> TokenStream {
    let field_name = field.ident.as_ref().expect("Unreachable");
    let span = field_name.span();
    let config_attrs = parse_config_attributes(&field.attrs);
    let builder_args = attrs_to_args(&config_attrs);
    let sup_type = gen_type(field, typ, &config_attrs);
    let name = LitStr::new(&field_name.to_string(), span);
    let is_required = typ.is_inside_option();
    quote_spanned! {span=>
        ::config::CKwargBuilder::new(
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
        SupportedTypes::Bool | SupportedTypes::OptionBool => quote_spanned! {span=>
            ::config::CTypes::Bool(
                ::config::CBoolBuilder::new()
                #args
                .build()
            )
        },
        SupportedTypes::String | SupportedTypes::OptionString => quote_spanned! {span=>
            ::config::CTypes::String(
                ::config::CStringBuilder::new()
                #args
                .build()
            )
        },
        SupportedTypes::Integer | SupportedTypes::OptionInteger => quote_spanned! {span=>
            ::config::CTypes::Integer(
                ::config::CIntegerBuilder::new()
                #args
                .build()
            )
        },
        SupportedTypes::Vec(sub_type) => {
            //emit_call_site_warning!(format!("{:#?}", *sub_type));
            let sub_arg = gen_type(field, sub_type, config_attrs);
            quote_spanned! {span=>
                ::config::CTypes::Vec(
                    ::config::CVecBuilder::new(|| #sub_arg)
                    .build()
                )
            }
        }
        SupportedTypes::Struct(ty) => {
            if !args.is_empty() {
                abort!(ty, "Sub args are not allowed for ConfigStructs")
            } else {
                quote_spanned! {span=>
                    ::config::CTypes::Struct(
                        #ty::build_app()
                        #args
                    )
                }
            }
        }
        SupportedTypes::CheckableStruct(ty) => {
            quote_spanned! {span=>
                ::config::CTypes::CheckableStruct(
                    ::config::CCheckableStructBuilder::new(
                        #ty::build_app()
                    )
                    #args
                    .build()
                )
            }
        }
        SupportedTypes::Enum(ty) | SupportedTypes::OptionEnum(ty) => {
            if !args.is_empty() {
                abort!(ty, "Sub args are not allowed for Enum")
            } else {
                quote_spanned! {span=>
                    ::config::CTypes::Enum(
                        #ty::build_app()
                        #args
                    )
                }
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
