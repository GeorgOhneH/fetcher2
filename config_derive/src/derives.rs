use proc_macro_error::abort_call_site;

use proc_macro2::{Span, TokenStream};
use quote::{quote, ToTokens, TokenStreamExt};

use proc_macro_error::abort;

use crate::build_app::{gen_enum_build_app_fn, gen_struct_build_app_fn};
use crate::config_attr::{parse_config_attributes, ConfigAttr};
use syn::{
    self, punctuated::Punctuated, token::Comma, Attribute, Data, DataStruct, DeriveInput, Field,
    Fields, GenericArgument, Ident, PathArguments, TypePath,
};
use syn::{DataEnum, Type};

pub enum HashType {
    String,
    Path,
}


impl ToTokens for HashType {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        use HashType::*;
        match self {
            String => tokens.append(Ident::new("String", Span::call_site())),
            Path => tokens.append(Ident::new("PathBuf", Span::call_site())),
        }
    }
}

pub enum SupportedTypes {
    String,
    OptionString,
    Integer,
    OptionInteger,
    Bool,
    OptionBool,
    Path,
    OptionPath,
    Vec(Box<SupportedTypes>),
    HashMap(HashType, Box<SupportedTypes>),
    Struct(TypePath),
    CheckableStruct(TypePath), // aka OptionStruct
    Enum(TypePath),
    OptionEnum(TypePath),
}

impl SupportedTypes {
    pub fn is_inside_option(&self) -> bool {
        use SupportedTypes::*;
        match self {
            String | Integer | Bool | Vec(_) | Struct(_) | Enum(_) | Path | HashMap(_, _) => false,
            OptionString | OptionInteger | OptionBool | CheckableStruct(_) | OptionEnum(_)
            | OptionPath => true,
        }
    }
}

impl ToTokens for SupportedTypes {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        use SupportedTypes::*;
        match self {
            String => tokens.append(Ident::new("String", Span::call_site())),
            OptionString => tokens.append(Ident::new("Option<String>", Span::call_site())),
            Integer => tokens.append(Ident::new("isize", Span::call_site())),
            OptionInteger => tokens.append(Ident::new("Option<isize>", Span::call_site())),
            Bool => tokens.append(Ident::new("bool", Span::call_site())),
            OptionBool => tokens.append(Ident::new("Other<bool>", Span::call_site())),
            Path => tokens.append(Ident::new("PathBuf", Span::call_site())),
            OptionPath => tokens.append(Ident::new("Other<PathBuf>", Span::call_site())),
            Vec(sup_typ) => {
                tokens.append(Ident::new("Vec<", Span::call_site()));
                sup_typ.to_tokens(tokens);
                tokens.append(Ident::new(">", Span::call_site()));
            }
            HashMap(key, value) => {
                tokens.append(Ident::new("HashMap<", Span::call_site()));
                key.to_tokens(tokens);
                tokens.append(Ident::new(", ", Span::call_site()));
                value.to_tokens(tokens);
                tokens.append(Ident::new(">", Span::call_site()));
            }
            Struct(type_path) => type_path.to_tokens(tokens),
            CheckableStruct(type_path) => type_path.to_tokens(tokens),
            Enum(type_path) => type_path.to_tokens(tokens),
            OptionEnum(type_path) => type_path.to_tokens(tokens),
        }
    }
}

pub fn derive_config(input: &DeriveInput) -> TokenStream {
    let ident = &input.ident;

    match input.data {
        Data::Struct(DataStruct {
            fields: Fields::Named(ref fields),
            ..
        }) => gen_for_struct(ident, &fields.named, &input.attrs),
        Data::Struct(DataStruct {
            fields: Fields::Unit,
            ..
        }) => gen_for_struct(ident, &Punctuated::<Field, Comma>::new(), &input.attrs),
        Data::Enum(ref e) => gen_for_enum(ident, &input.attrs, e),
        _ => abort_call_site!("`#[derive(Config)]` only supports non-tuple structs and enums"),
    }
}

fn gen_for_struct(
    name: &Ident,
    fields: &Punctuated<Field, Comma>,
    _attrs: &[Attribute],
) -> TokenStream {
    let build_app_fn = gen_struct_build_app_fn(fields);
    let parse_fn = crate::parse_from_app::gen_struct_parse_fn(fields);
    let update_app_fn = crate::update_app::gen_struct_update_app_fn(fields);

    quote! {
        #[allow(dead_code, unreachable_code, unused_variables)]
        #[allow(
            clippy::style,
            clippy::complexity,
            clippy::pedantic,
            clippy::restriction,
            clippy::perf,
            clippy::deprecated,
            clippy::nursery,
            clippy::cargo
        )]
        #[deny(clippy::correctness)]
        impl ::config::Config for #name {
            #build_app_fn
            #parse_fn
            #update_app_fn
        }

    }
}

fn gen_for_enum(name: &Ident, _attrs: &[Attribute], e: &DataEnum) -> TokenStream {
    let build_app_fn = gen_enum_build_app_fn(e);
    let parse_fn = crate::parse_from_app::gen_enum_parse_fn(e);
    let update_app_fn = crate::update_app::gen_enum_update_app_fn(e);

    quote! {
        #[allow(dead_code, unreachable_code, unused_variables)]
        #[allow(
            clippy::style,
            clippy::complexity,
            clippy::pedantic,
            clippy::restriction,
            clippy::perf,
            clippy::deprecated,
            clippy::nursery,
            clippy::cargo
        )]
        #[deny(clippy::correctness)]
        impl ::config::ConfigEnum for #name {
            #build_app_fn
            #parse_fn
            #update_app_fn
        }

    }
}

pub fn parse_type(ty: &Type, attrs: &[Attribute]) -> SupportedTypes {
    if let Some((name, inner_types)) = extract_type_from_bracket(ty) {
        match (&*name.to_string(), &inner_types[..]) {
            ("Vec", [inner]) => {
                let inner_ty = parse_type(inner, attrs);
                if inner_ty.is_inside_option() {
                    abort!(ty, "Option can not be in Vec")
                }
                SupportedTypes::Vec(Box::new(inner_ty))
            }
            ("HashMap", [key, value]) => {
                let key_ty = parse_hash_type(key, attrs);
                let value_ty = parse_type(value, attrs);
                if value_ty.is_inside_option() {
                    abort!(ty, "Option can not be in Hashmap")
                }
                SupportedTypes::HashMap(key_ty, Box::new(value_ty))
            }
            ("Option", [inner]) => {
                let inner_supported_type = parse_type(inner, attrs);
                match inner_supported_type {
                    SupportedTypes::Struct(type_path) => SupportedTypes::CheckableStruct(type_path),
                    SupportedTypes::String => SupportedTypes::OptionString,
                    SupportedTypes::Integer => SupportedTypes::OptionInteger,
                    SupportedTypes::Bool => SupportedTypes::OptionBool,
                    SupportedTypes::Path => SupportedTypes::OptionPath,
                    SupportedTypes::Enum(ty_path) => SupportedTypes::OptionEnum(ty_path),
                    _ => abort!(ty, "Can not be inside an Option"),
                }
            }
            _ => abort!(ty, "Not Supported type"),
        }
    } else {
        match ty {
            Type::Path(type_path) if type_path.path.get_ident().is_some() => {
                match &*type_path.path.get_ident().unwrap().to_string() {
                    "String" => SupportedTypes::String,
                    "isize" => SupportedTypes::Integer,
                    "bool" => SupportedTypes::Bool,
                    "PathBuf" => SupportedTypes::Path,
                    _ => {
                        let type_args: Vec<String> = parse_config_attributes(attrs)
                            .iter()
                            .filter_map(|config_attr| {
                                if let ConfigAttr::Type(_, lit) = config_attr {
                                    Some(lit.value())
                                } else {
                                    None
                                }
                            })
                            .collect();
                        if type_args.len() != 1 {
                            abort!(ty, "Field must have exactly one type attribute")
                        }
                        match &*type_args[0] {
                            "struct" => SupportedTypes::Struct(type_path.clone()),
                            "enum" => SupportedTypes::Enum(type_path.clone()),
                            _ => abort!(ty, "Not Supported type. Use 'struct' or 'enum'"),
                        }
                    }
                }
            }
            _ => abort!(ty, "Not Supported type"),
        }
    }
}

pub fn parse_hash_type(ty: &Type, attrs: &[Attribute]) -> HashType {
    if let Some((name, inner_types)) = extract_type_from_bracket(ty) {
        abort!(ty, "Not Supported type")
    } else {
        match ty {
            Type::Path(type_path) if type_path.path.get_ident().is_some() => {
                match &*type_path.path.get_ident().unwrap().to_string() {
                    "String" => HashType::String,
                    "PathBuf" => HashType::Path,
                    _ => abort!(ty, "Not Supported type"),
                }
            }
            _ => abort!(ty, "Not Supported type"),
        }
    }
}

fn extract_type_from_bracket(ty: &Type) -> Option<(&Ident, Vec<&Type>)> {
    if let Type::Path(type_path) = ty {
        let bracket_name = path_get_bracket_name(type_path)?;
        let type_params = &type_path.path.segments.iter().next().unwrap().arguments;
        let generic_arg: Option<Vec<_>> = match type_params {
            PathArguments::AngleBracketed(params) => params
                .args
                .iter()
                .map(|arg| match arg {
                    GenericArgument::Type(ty) => Some(ty),
                    _ => None,
                })
                .collect(),
            _ => None,
        };
        generic_arg.map(|types| (bracket_name, types))
    } else {
        None
    }
}

fn path_get_bracket_name(type_path: &TypePath) -> Option<&Ident> {
    if type_path.path.leading_colon.is_none() && type_path.path.segments.len() == 1 {
        Some(&type_path.path.segments.iter().next().unwrap().ident)
    } else {
        None
    }
}
