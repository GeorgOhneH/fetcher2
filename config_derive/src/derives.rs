

use proc_macro2::TokenStream;
use proc_macro_error::abort_call_site;

use proc_macro_error::{abort};
use quote::{quote};

use syn::{
    self, punctuated::Punctuated, token::Comma, Attribute, Data,
    DataStruct, DeriveInput, Field, Fields, GenericArgument, Ident,
    PathArguments, TypePath,
};
use syn::{Type};
use crate::build_app::gen_build_app_fn;

pub enum SupportedTypes {
    String,
    Integer,
    Bool,
    Vec(Box<SupportedTypes>),
    Option(Box<SupportedTypes>),
    Other(TypePath),
    None(Type),
}

impl SupportedTypes {
    fn can_be_in_option(&self) -> bool {
        match self {
            Self::Vec(_) | Self::Option(_) | Self::Other(_) | Self::None(_) => false,
            _ => true,
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
        }

    }
}

pub fn convert_type(ty: &Type) -> SupportedTypes {
    if let Some((name, inner_ty)) = extract_type_from_bracket(ty) {
        //emit_call_site_warning!(name.to_string());
        match &*name.to_string() {
            "Vec" => SupportedTypes::Vec(Box::new(convert_type(inner_ty))),
            "Option" => {
                let inner_supported_type = convert_type(inner_ty);
                if !inner_supported_type.can_be_in_option() {
                    abort!(&inner_ty, "This can not be in a Option")
                }
                SupportedTypes::Option(Box::new(convert_type(inner_ty)))
            }
            _ => SupportedTypes::None(ty.clone()),
        }
    } else {
        match ty {
            Type::Path(type_path) if type_path.path.get_ident().is_some() => {
                match &*type_path.path.get_ident().unwrap().to_string() {
                    "String" => SupportedTypes::String,
                    "i32" => SupportedTypes::Integer,
                    "bool" => SupportedTypes::Bool,
                    _ => SupportedTypes::Other(type_path.clone()),
                }
            }
            _ => SupportedTypes::None(ty.clone()),
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
