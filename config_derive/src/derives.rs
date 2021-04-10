use proc_macro_error::abort_call_site;

use proc_macro2::{Span, TokenStream};
use quote::{quote, ToTokens, TokenStreamExt};

use proc_macro_error::abort;

use crate::build_app::gen_build_app_fn;
use syn::Type;
use syn::{
    self, punctuated::Punctuated, token::Comma, Attribute, Data, DataStruct, DeriveInput, Field,
    Fields, GenericArgument, Ident, PathArguments, TypePath,
};

pub enum SupportedTypes {
    String,
    OtherString,
    Integer,
    OtherInteger,
    Bool,
    OtherBool,
    Vec(Box<SupportedTypes>),
    Struct(TypePath),
    CheckableStruct(TypePath), // aka OtherStruct
}

impl SupportedTypes {
    pub fn is_inside_option(&self) -> bool {
        use SupportedTypes::*;
        match self {
            String | Integer | Bool | Vec(_) | Struct(_) => false,
            OtherString | OtherInteger | OtherBool | CheckableStruct(_) => true,
        }
    }
}

impl ToTokens for SupportedTypes {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        use SupportedTypes::*;
        match self {
            String => tokens.append(Ident::new("String", Span::call_site())),
            OtherString => tokens.append(Ident::new("Option<String>", Span::call_site())),
            Integer => tokens.append(Ident::new("isize", Span::call_site())),
            OtherInteger => tokens.append(Ident::new("Option<isize>", Span::call_site())),
            Bool => tokens.append(Ident::new("bool", Span::call_site())),
            OtherBool => tokens.append(Ident::new("Other<bool>", Span::call_site())),
            Vec(sup_typ) => {
                tokens.append(Ident::new("Vec<", Span::call_site()));
                sup_typ.to_tokens(tokens);
                tokens.append(Ident::new(">", Span::call_site()));
            }
            Struct(type_path) => type_path.to_tokens(tokens),
            CheckableStruct(type_path) => type_path.to_tokens(tokens),
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
        _ => abort_call_site!("`#[derive(Config)]` only supports non-tuple structs"),
    }
}

fn gen_for_struct(
    name: &Ident,
    fields: &Punctuated<Field, Comma>,
    _attrs: &[Attribute],
) -> TokenStream {
    let build_app_fn = gen_build_app_fn(fields);
    let parse_fn = crate::parse_from_app::gen_parse_fn(fields);
    let update_app_fn = crate::update_app::gen_update_app_fn(fields);

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

pub fn convert_type(ty: &Type) -> SupportedTypes {
    use SupportedTypes::*;
    if let Some((name, inner_ty)) = extract_type_from_bracket(ty) {
        //emit_call_site_warning!(name.to_string());
        match &*name.to_string() {
            "Vec" => {
                let inner_ty = convert_type(inner_ty);
                if inner_ty.is_inside_option() {
                    abort!(ty, "Option can not be in Vec")
                }
                Vec(Box::new(inner_ty))
            }
            "Option" => {
                let inner_supported_type = convert_type(inner_ty);
                match inner_supported_type {
                    Struct(type_path) => CheckableStruct(type_path),
                    String => OtherString,
                    Integer => OtherInteger,
                    Bool => OtherBool,
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
                    _ => SupportedTypes::Struct(type_path.clone()),
                }
            }
            _ => abort!(ty, "Not Supported type"),
        }
    }
}

fn extract_type_from_bracket(ty: &Type) -> Option<(&Ident, &Type)> {
    match ty {
        Type::Path(type_path) => {
            match path_get_bracket_name(type_path) {
                Some(bracket_name) => {
                    let type_params = &type_path.path.segments.iter().next().unwrap().arguments;
                    // It should have only on angle-bracketed param ("<String>"):
                    let generic_arg = match type_params {
                        PathArguments::AngleBracketed(params) => {
                            Some(params.args.iter().next().unwrap())
                        }
                        _ => None,
                    };
                    // This argument must be a type:
                    match generic_arg {
                        Some(GenericArgument::Type(ty)) => Some((bracket_name, ty)),
                        _ => None,
                    }
                }
                _ => None,
            }
        }
        _ => None,
    }
}

fn path_get_bracket_name(type_path: &TypePath) -> Option<&Ident> {
    if type_path.path.leading_colon.is_none() && type_path.path.segments.len() == 1 {
        Some(&type_path.path.segments.iter().next().unwrap().ident)
    } else {
        None
    }
}
