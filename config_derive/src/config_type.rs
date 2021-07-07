




use proc_macro_error::abort;


use crate::config_attr::{parse_config_attributes, ConfigAttr};
use syn::{
    self, Attribute, GenericArgument, Ident, Path, PathArguments, TypePath,
};
use syn::{Type};

pub enum ConfigHashType {
    String,
    Path,
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
    Wrapper(Path, Box<ConfigType>, Ident),
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
            | Wrapper(_, _, _) => false,
            OptionString(_) | OptionInteger(_) | OptionBool(_) | CheckableStruct(_)
            | OptionEnum(_) | OptionPath(_) => true,
        }
    }
}

pub fn parse_type(ty: &Type, attrs: &[Attribute]) -> ConfigType {
    _parse_type(ty, attrs, false)
}
fn _parse_type(ty: &Type, attrs: &[Attribute], inner: bool) -> ConfigType {
    if let Some((path, inner_types)) = extract_type_from_bracket(ty) {
        let type_args: Vec<String> = parse_config_attributes(attrs)
            .iter()
            .filter_map(|config_attr| {
                if inner {
                    if let ConfigAttr::InnerType(_, lit) = config_attr {
                        Some(lit.value())
                    } else {
                        None
                    }
                } else {
                    if let ConfigAttr::Type(_, lit) = config_attr {
                        Some(lit.value())
                    } else {
                        None
                    }
                }
            })
            .collect();
        let name = match type_args.len() {
            1 => type_args[0].to_owned(),
            0 => path.segments[0].ident.to_string(),
            _ => abort!(ty, "Can't have more then 1 inner type attribute"),
        };
        match (&*name, &inner_types[..]) {
            ("Vec", [inner]) => {
                let inner_ty = _parse_type(inner, attrs, true);
                if inner_ty.is_inside_option() {
                    abort!(ty, "Option can not be in Vec")
                }
                ConfigType::Vec(path.clone(), Box::new(inner_ty))
            }
            ("HashMap", [key, value]) => {
                let key_ty = parse_hash_type(key, attrs);
                let value_ty = _parse_type(value, attrs, true);
                if value_ty.is_inside_option() {
                    abort!(ty, "Option can not be in Hashmap")
                }
                ConfigType::HashMap(path.clone(), key_ty, Box::new(value_ty))
            }
            ("Option", [inner]) => {
                let inner_supported_type = _parse_type(inner, attrs, true);
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
                let inner_ty = _parse_type(inner, attrs, true);
                ConfigType::Wrapper(
                    path.clone(),
                    Box::new(inner_ty),
                    path.segments[0].ident.to_owned(),
                )
            }
            ("RwLock", [inner]) => {
                let inner_ty = _parse_type(inner, attrs, true);
                ConfigType::Wrapper(
                    path.clone(),
                    Box::new(inner_ty),
                    path.segments[0].ident.to_owned(),
                )
            }
            ("Arc", [inner]) => {
                let inner_ty = _parse_type(inner, attrs, true);
                ConfigType::Wrapper(
                    path.clone(),
                    Box::new(inner_ty),
                    path.segments[0].ident.to_owned(),
                )
            }
            ("String", []) => ConfigType::String(path.clone()),
            ("isize", []) => ConfigType::Integer(path.clone()),
            ("bool", []) => ConfigType::Bool(path.clone()),
            ("PathBuf", []) => ConfigType::Path(path.clone()),
            (_, []) if type_args.len() == 1 => match &*type_args[0] {
                "struct" => ConfigType::Struct(path.clone()),
                "enum" => ConfigType::Enum(path.clone()),
                _ => abort!(ty, "Not Supported type. Use 'struct' or 'enum'"),
            },
            (x, _) => abort!(ty, "{} is not supported", x),
        }
    } else {
        abort!(ty, "Not Supported type1")
    }
}

pub fn parse_hash_type(ty: &Type, _attrs: &[Attribute]) -> ConfigHashType {
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
