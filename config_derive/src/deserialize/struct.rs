use proc_macro2::{Ident, TokenStream};
use quote::quote;
use syn::{self, Field, LitStr, punctuated::Punctuated, token::Comma};
use syn::spanned::Spanned;

use crate::config_type::{ConfigType, ConfigWrapperType, parse_type};

pub fn gen_field(fields: &Punctuated<Field, Comma>) -> TokenStream {
    let field_names = fields
        .iter()
        .filter_map(|field| {
            let typ = parse_type(&field.ty, &field.attrs);
            if let ConfigType::Skip(_) = typ {
                None
            } else {
                Some(gen_field_name(field))
            }
        })
        .collect::<Vec<_>>();

    let field_name_strings = fields
        .iter()
        .filter_map(|field| {
            let typ = parse_type(&field.ty, &field.attrs);
            if let ConfigType::Skip(_) = typ {
                None
            } else {
                Some(gen_field_name_string(field))
            }
        })
        .collect::<Vec<_>>();

    quote! {
        const FIELDS: &'static [&'static str] = &[#(#field_name_strings,)*];
        #[allow(non_camel_case_types)]
        enum Field { #(#field_names,)* __Nothing }
        impl<'de> serde::Deserialize<'de> for Field {
            fn deserialize<D>(deserializer: D) -> std::result::Result<Field, D::Error>
                where
                    D: serde::Deserializer<'de>,
            {
                struct FieldVisitor;

                impl<'de> serde::de::Visitor<'de> for FieldVisitor {
                    type Value = Field;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                        formatter.write_str("not valid field found")
                    }

                    fn visit_str<E>(self, value: &str) -> std::result::Result<Field, E>
                        where
                            E: serde::de::Error,
                    {
                        match value {
                            #(#field_name_strings => Ok(Field::#field_names),)*
                            _ => Ok(Field::__Nothing),
                        }
                    }
                }

                deserializer.deserialize_identifier(FieldVisitor)
            }
        }
    }
}

fn gen_field_name(field: &Field) -> TokenStream {
    let field_name = field.ident.as_ref().expect("Unreachable");
    quote! { #field_name }
}

fn gen_field_name_string(field: &Field) -> TokenStream {
    let field_name = field.ident.as_ref().expect("Unreachable");
    let name = LitStr::new(&field_name.to_string(), field.span());
    quote! { #name }
}

pub fn gen_visitor(fields: &Punctuated<Field, Comma>, name: &Ident) -> TokenStream {
    let name_str = LitStr::new(&name.to_string(), name.span());
    let field_names = fields
        .iter()
        .filter_map(|field| {
            let typ = parse_type(&field.ty, &field.attrs);
            if let ConfigType::Skip(_) = typ {
                None
            } else {
                Some(gen_field_name(field))
            }
        })
        .collect::<Vec<_>>();

    let field_name_strings = fields
        .iter()
        .filter_map(|field| {
            let typ = parse_type(&field.ty, &field.attrs);
            if let ConfigType::Skip(_) = typ {
                None
            } else {
                Some(gen_field_name_string(field))
            }
        })
        .collect::<Vec<_>>();

    let c_struct_setter = fields
        .iter()
        .filter_map(|field| {
            let typ = parse_type(&field.ty, &field.attrs);
            if let ConfigType::Skip(_) = typ {
                None
            } else {
                let field_name = field.ident.as_ref().expect("Unreachable");
                let field_str = LitStr::new(&field_name.to_string(), field.span());
                Some(gen_c_setter(
                    field,
                    typ,
                    quote! {cstruct.get_ty_mut(#field_str).unwrap()},
                    quote! {value},
                ))
            }
        })
        .collect::<Vec<_>>();

    quote! {
        struct DurationVisitor;

        impl<'de> serde::de::Visitor<'de> for DurationVisitor {
            type Value = #name;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("struct #name")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<Self::Value, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut cstruct: config::CStruct = #name::builder().build();
                #(let mut #field_names = None;)*
                while let Ok(Some(key)) = map.next_key() {
                    match key {
                        #(
                            Field::#field_names => {
                                if #field_names.is_some() {
                                    return Err(serde::de::Error::duplicate_field(#field_name_strings));
                                }
                                if let Ok(value) = map.next_value() {
                                    #field_names = Some(value);
                                }
                            }
                        )*
                            Field::__Nothing => {
                                let _: std::result::Result<(),_> = map.next_value();
                            }
                    }
                }
                #(
                    if let Some(value) = #field_names {
                        #c_struct_setter
                    }
                )*
                #name::parse_from_app(&cstruct).map_err(|err| serde::de::Error::custom(err.msg))
            }
        }

        deserializer.deserialize_struct(#name_str, FIELDS, DurationVisitor)
    }
}

fn gen_c_setter(
    field: &Field,
    ty: ConfigType,
    ctype: TokenStream,
    field_name: TokenStream,
) -> TokenStream {
    match ty {
        ConfigType::String(_) => quote! {
            #ctype.string_mut().unwrap().set(#field_name);
        },
        ConfigType::OptionString(_) => quote! {
            #ctype.string_mut().unwrap().set_raw(#field_name);
        },
        ConfigType::Integer(_) => quote! {
            let _ = #ctype.int_mut().unwrap().set(#field_name);
        },
        ConfigType::OptionInteger(_) => quote! {
            let _ = #ctype.int_mut().unwrap().set_raw(#field_name);
        },
        ConfigType::Bool(_) => quote! {
            #ctype.bool_mut().unwrap().set(#field_name);
        },
        ConfigType::OptionBool(_) => quote! {
            #ctype.bool_mut().unwrap().set_option(#field_name);
        },
        ConfigType::Path(_) => quote! {
            let _ = #ctype.path_mut().unwrap().set::<String>(#field_name);
        },
        ConfigType::OptionPath(_) => quote! {
            let _ = #ctype.path_mut().unwrap().set_raw::<String>(#field_name);
        },
        ConfigType::Vec(_, inner_ty) => {
            let inner_setter = gen_c_setter(field, *inner_ty, quote! {ctype}, quote! {inner_value});
            quote! {
                let value_hint: Vec<_> = #field_name;
                let cvec = #ctype.vec_mut().unwrap();
                let new = value_hint.into_iter().enumerate().map(|(i, inner_value)| {
                    let mut ctype = cvec.get_template();
                    #inner_setter;
                    config::CItem::new(ctype, i)
                }).collect();
                cvec.set(new);
            }
        }
        ConfigType::HashMap(_path, _key_ty, inner_ty) => {
            let inner_setter = gen_c_setter(field, *inner_ty, quote! {cvalue}, quote! {value});
            quote! {
                let value_hint: HashMap<String, _> = #field_name;
                let cmap = #ctype.map_mut().unwrap();
                let new = value_hint.into_iter().map(|(key, value)| {
                    let mut ckey = cmap.get_key();
                    ckey.set(key);
                    let mut cvalue = cmap.get_value();
                    #inner_setter
                    (ckey, cvalue)
                }).collect();
                cmap.set(new);
            }
        }
        ConfigType::Struct(path) => {
            quote! {
                let value_hint: #path = #field_name;
                let _ = value_hint.update_app(&mut #ctype.struct_mut().unwrap());
            }
        }
        ConfigType::CheckableStruct(path) => {
            quote! {
                let value_hint: Option<#path> = #field_name;
                let c_ceck_struct = #ctype.check_struct_mut().unwrap();
                match value_hint {
                    Some(inner_value) => {
                        c_ceck_struct.set_checked(true);
                        let _ = inner_value.update_app(c_ceck_struct.get_inner_mut());
                    }
                    None => c_ceck_struct.set_checked(false),
                }
            }
        }
        ConfigType::Enum(path) => {
            quote! {
                let value_hint: #path = #field_name;
                let _ = value_hint.update_app(&mut #ctype.enum_mut().unwrap());
            }
        }
        ConfigType::OptionEnum(path) => {
            quote! {
                let value_hint: Option<#path> = #field_name;
                let e_enum = #ctype.enum_mut().unwrap();
                match value_hint {
                    Some(inner_value) => {
                        let _ = inner_value.update_app(e_enum);
                    }
                    None => e_enum.unselect(),
                }
            }
        }
        ConfigType::Wrapper(path, inner_ty, wrapper_ty) => {
            let inner_setter =
                gen_c_setter(field, *inner_ty, quote! {c_inner}, quote! {inner_value});
            let inner_value_quote = match wrapper_ty {
                ConfigWrapperType::Arc => quote! {
                    Arc::try_unwrap(value_hint).unwrap()
                },
                ConfigWrapperType::Mutex => quote! {
                    value_hint.into_inner().unwrap()
                },
                ConfigWrapperType::RwLock => quote! {
                    value_hint.into_inner().unwrap()
                },
            };
            quote! {
                let value_hint: #path = #field_name;
                let c_inner = #ctype.wrapper_mut().unwrap().inner_mut();
                let inner_value = #inner_value_quote;
                #inner_setter
            }
        }
        ConfigType::Skip(_) => unreachable!(),
    }
}