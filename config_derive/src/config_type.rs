use lazy_static::lazy_static;
use proc_macro_error::abort;
use regex::Regex;

use crate::config_attr::{parse_config_attributes, ConfigAttr};
use proc_macro2::Span;
use syn::spanned::Spanned;
use syn::Type;
use syn::{self, Attribute, Expr, GenericArgument, Path, PathArguments, TypePath};

pub enum ConfigHashType {
    String,
    Path,
}

pub enum ConfigWrapperType {
    Mutex,
    RwLock,
    Arc,
}

pub enum ConfigType {
    String(Path),
    OptionString(Path),
    Integer(Path),
    OptionInteger(Path),
    Bool(Path),
    OptionBool(Path),
    Path(Path),
    OptionPath(Path),
    Vec(Path, Box<ConfigType>),
    HashMap(Path, ConfigHashType, Box<ConfigType>),
    Struct(Path),
    CheckableStruct(Path), // aka OptionStruct
    Enum(Path),
    OptionEnum(Path),
    Wrapper(Path, Box<ConfigType>, ConfigWrapperType),
    Skip(Expr),
}

impl ConfigType {
    pub fn is_inside_option(&self) -> bool {
        use ConfigType::*;
        match self {
            String(_)
            | Integer(_)
            | Bool(_)
            | Vec(_, _)
            | Struct(_)
            | Enum(_)
            | Path(_)
            | HashMap(_, _, _)
            | Skip(_)
            | Wrapper(_, _, _) => false,
            OptionString(_) | OptionInteger(_) | OptionBool(_) | CheckableStruct(_)
            | OptionEnum(_) | OptionPath(_) => true,
        }
    }
}

pub fn parse_type(ty: &Type, attrs: &[Attribute]) -> ConfigType {
    let config_attrs = parse_config_attributes(attrs);

    if let Some(ConfigAttr::Skip(expr)) = config_attrs
        .iter()
        .find(|attr| matches!(attr, ConfigAttr::Skip(_)))
    {
        return ConfigType::Skip(expr.clone());
    }

    let type_annots = config_attrs.iter().find_map(|config_attr| {
        if let ConfigAttr::Type(_, lit) = config_attr {
            Some(extract_value_from_ty_string(&lit.value(), ty.span()))
        } else {
            None
        }
    });

    _parse_type(ty, type_annots)
}

fn _parse_type(ty: &Type, type_annots: Option<TypeAnnotations>) -> ConfigType {
    if let Some((path, inner_types)) = extract_type_from_bracket(ty) {
        let name = if let Some(TypeAnnotations {
            name: Some(name), ..
        }) = &type_annots
        {
            name.clone()
        } else {
            path.segments[0].ident.to_string()
        };

        match (&*name, &inner_types[..]) {
            ("Vec", [inner]) => {
                let inner_type_annot =
                    type_annots.and_then(|type_annots| type_annots.into_inner_1());
                let inner_ty = _parse_type(inner, inner_type_annot);
                if inner_ty.is_inside_option() {
                    abort!(ty, "Option can not be in Vec")
                }
                ConfigType::Vec(path.clone(), Box::new(inner_ty))
            }
            ("HashMap", [key, value]) => {
                let (key_annot, value_annot) = if let Some(type_annots) = type_annots {
                    type_annots.into_inner_2()
                } else {
                    (None, None)
                };
                let key_ty = parse_hash_type(key, key_annot);
                let value_ty = _parse_type(value, value_annot);
                if value_ty.is_inside_option() {
                    abort!(ty, "Option can not be in Hashmap")
                }
                ConfigType::HashMap(path.clone(), key_ty, Box::new(value_ty))
            }
            ("Option", [inner]) => {
                let inner_type_annot =
                    type_annots.and_then(|type_annots| type_annots.into_inner_1());
                let inner_supported_type = _parse_type(inner, inner_type_annot);
                match inner_supported_type {
                    ConfigType::Struct(path) => ConfigType::CheckableStruct(path),
                    ConfigType::String(path) => ConfigType::OptionString(path),
                    ConfigType::Integer(path) => ConfigType::OptionInteger(path),
                    ConfigType::Bool(path) => ConfigType::OptionBool(path),
                    ConfigType::Path(path) => ConfigType::OptionPath(path),
                    ConfigType::Enum(path) => ConfigType::OptionEnum(path),
                    _ => abort!(ty, "Can not be inside an Option"),
                }
            }
            ("Mutex", [inner]) => {
                let inner_type_annot =
                    type_annots.and_then(|type_annots| type_annots.into_inner_1());
                let inner_ty = _parse_type(inner, inner_type_annot);
                ConfigType::Wrapper(path.clone(), Box::new(inner_ty), ConfigWrapperType::Mutex)
            }
            ("RwLock", [inner]) => {
                let inner_type_annot =
                    type_annots.and_then(|type_annots| type_annots.into_inner_1());
                let inner_ty = _parse_type(inner, inner_type_annot);
                ConfigType::Wrapper(path.clone(), Box::new(inner_ty), ConfigWrapperType::RwLock)
            }
            ("Arc", [inner]) => {
                let inner_type_annot =
                    type_annots.and_then(|type_annots| type_annots.into_inner_1());
                let inner_ty = _parse_type(inner, inner_type_annot);
                ConfigType::Wrapper(path.clone(), Box::new(inner_ty), ConfigWrapperType::Arc)
            }
            ("String", []) => ConfigType::String(path.clone()),
            ("isize", []) => ConfigType::Integer(path.clone()),
            ("bool", []) => ConfigType::Bool(path.clone()),
            ("PathBuf", []) => ConfigType::Path(path.clone()),
            ("Struct", []) => ConfigType::Struct(path.clone()),
            ("Enum", []) => ConfigType::Enum(path.clone()),
            (x, _) => abort!(ty, "{} is not supported", x),
        }
    } else {
        abort!(ty, "Not Supported type1")
    }
}

pub fn parse_hash_type(ty: &Type, _attrs: Option<TypeAnnotations>) -> ConfigHashType {
    if let Some((path, inner_types)) = extract_type_from_bracket(ty) {
        match (&*path.segments[0].ident.to_string(), &inner_types[..]) {
            ("String", []) => ConfigHashType::String,
            ("PathBuf", []) => ConfigHashType::Path,
            _ => abort!(ty, "Not Supported type"),
        }
    } else {
        abort!(ty, "Not Supported type")
    }
}

fn extract_type_from_bracket(ty: &Type) -> Option<(&Path, Vec<&Type>)> {
    if let Type::Path(TypePath { path, qself: None }) = ty {
        let type_params = &path.segments.iter().last().unwrap().arguments;
        let inner_types: Vec<_> = match type_params {
            PathArguments::AngleBracketed(params) => params
                .args
                .iter()
                .map(|arg| match arg {
                    GenericArgument::Type(ty) => Some(ty),
                    _ => None,
                })
                .collect::<Option<_>>()?,
            PathArguments::None => vec![],
            _ => return None,
        };
        Some((path, inner_types))
    } else {
        None
    }
}

#[derive(Debug)]
pub struct TypeAnnotations {
    pub name: Option<String>,
    pub inner: Option<Vec<TypeAnnotations>>,
    pub span: Span,
}

impl TypeAnnotations {
    pub fn new(raw_name: &str, inner: Vec<TypeAnnotations>, span: Span) -> Self {
        let raw_name = raw_name.trim();
        if raw_name == "_" {
            Self {
                name: None,
                inner: Some(inner),
                span,
            }
        } else {
            Self {
                name: Some(raw_name.to_owned()),
                inner: Some(inner),
                span,
            }
        }
    }

    pub fn raw_name(raw_name: &str, span: Span) -> Self {
        let raw_name = raw_name.trim();
        if raw_name == "_" {
            Self {
                name: None,
                inner: None,
                span,
            }
        } else {
            Self {
                name: Some(raw_name.to_owned()),
                inner: None,
                span,
            }
        }
    }

    pub fn into_inner_1(self) -> Option<Self> {
        if let Some(annots) = self.inner {
            if annots.len() != 1 {
                abort!(self.span, "Not Blabla")
            }
            Some(annots.into_iter().next().unwrap())
        } else {
            None
        }
    }

    pub fn into_inner_2(self) -> (Option<Self>, Option<Self>) {
        if let Some(annots) = self.inner {
            if annots.len() != 2 {
                abort!(self.span, "Not Blabla")
            }
            let mut iter = annots.into_iter();
            let first = iter.next().unwrap();
            let second = iter.next().unwrap();
            (Some(first), Some(second))
        } else {
            (None, None)
        }
    }
}

// ty = "_(_, Vec(_)>)"
fn extract_value_from_ty_string(ty_str: &str, span: Span) -> TypeAnnotations {
    lazy_static! {
        static ref BRACKET_RE: Regex = Regex::new(r"<(.+)>").unwrap();
    }
    let name = if let Some(idx) = ty_str.find("<") {
        &ty_str[0..idx]
    } else {
        return TypeAnnotations::raw_name(&ty_str.trim(), span);
    };

    if let Some(caps) = BRACKET_RE.captures(ty_str) {
        let inner_ty: &str = caps.get(1).unwrap().as_str();
        let inner: Vec<_> = inner_ty
            .split(",")
            .map(|raw_inner| raw_inner.trim())
            .map(|inner| extract_value_from_ty_string(inner, span))
            .collect();
        TypeAnnotations::new(name, inner, span)
    } else {
        abort!(span, "Not valid formatting")
    }
}
