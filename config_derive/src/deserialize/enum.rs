use proc_macro2::{Ident, TokenStream};
use proc_macro_error::abort;
use quote::quote;
use syn::{self, DataEnum, Fields, FieldsUnnamed, LitByteStr, LitStr};

pub fn gen_de_enum(e: &DataEnum, enum_name: &Ident) -> TokenStream {
    let enum_name_str = LitStr::new(&enum_name.to_string(), enum_name.span());
    let counts = 0u64..e.variants.len() as u64;
    let names = e.variants.iter().map(|var| &var.ident).collect::<Vec<_>>();
    let name_strs = e
        .variants
        .iter()
        .map(|var| {
            let name = &var.ident;
            LitStr::new(&name.to_string(), name.span())
        })
        .collect::<Vec<_>>();
    let name_bytes = e
        .variants
        .iter()
        .map(|var| {
            let name = &var.ident;
            LitByteStr::new(name.to_string().as_bytes(), name.span())
        })
        .collect::<Vec<_>>();
    let match_enums = e.variants.iter().map(|var| {
        let name = &var.ident;
        match &var.fields {
            Fields::Unnamed(FieldsUnnamed { unnamed, .. }) if unnamed.len() == 1 => {
                quote! {
                    Ok((Field::#name, var)) => {
                        if let Ok(inner) = var.newtype_variant() {
                            Some(#enum_name::#name(inner))
                        } else {
                            None
                        }
                    }
                }
            }
            Fields::Unit => {
                quote! {
                    Ok((Field::#name, _)) => Some(#enum_name::#name),
                }
            }
            _ => abort!(var.fields, "Only Unit and Single Tuple Enums are allowed"),
        }
    });

    quote! {
        impl<'de> serde::Deserialize<'de> for #enum_name {
            fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                enum Field {
                    #(#names,)*
                    __Nothing,
                }
                const VARIANTS: &'static [&'static str] = &[#(#name_strs,)*];
                impl<'de> serde::Deserialize<'de> for Field {
                    fn deserialize<D>(deserializer: D) -> std::result::Result<Field, D::Error>
                    where
                        D: serde::Deserializer<'de>,
                    {
                        struct FieldVisitor;

                        impl<'de> serde::de::Visitor<'de> for FieldVisitor {
                            type Value = Field;

                            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                                formatter.write_str("`secs` or `nanos`")
                            }

                            fn visit_u64<E>(self, v: u64) -> std::result::Result<Self::Value, E>
                            where
                                E: serde::de::Error,
                            {
                                match v {
                                    #(#counts => Ok(Field::#names),)*
                                    _ => Ok(Field::__Nothing),
                                }
                            }

                            fn visit_str<E>(self, v: &str) -> std::result::Result<Field, E>
                            where
                                E: serde::de::Error,
                            {
                                match v {
                                    #(#name_strs => Ok(Field::#names),)*
                                    _ => Ok(Field::__Nothing),
                                }
                            }

                            fn visit_bytes<E>(self, v: &[u8]) -> std::result::Result<Self::Value, E>
                            where
                                E: serde::de::Error,
                            {
                                match v {
                                    #(#name_bytes => Ok(Field::#names),)*
                                    _ => Ok(Field::__Nothing),
                                }
                            }
                        }

                        deserializer.deserialize_identifier(FieldVisitor)
                    }
                }

                struct _Visitor<'de> {
                    marker: std::marker::PhantomData<#enum_name>,
                    lifetime: std::marker::PhantomData<&'de ()>,
                }
                use serde::de::VariantAccess as _;
                impl<'de> serde::de::Visitor<'de> for _Visitor<'de> {
                    type Value = #enum_name;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                        formatter.write_str("enum ")?;
                        formatter.write_str(#enum_name_str)
                    }

                    fn visit_enum<A>(self, data: A) -> std::result::Result<Self::Value, A::Error>
                    where
                        A: serde::de::EnumAccess<'de>,
                    {
                        let mut cenum: config::CEnum = #enum_name::builder().build();
                        let current_enum = match data.variant() {
                            #(#match_enums)*
                            Ok((Field::__Nothing, _)) => None,
                            Err(_) => None,
                        };

                        if let Some(current_enum) = current_enum {
                            let _ = current_enum.update_app(&mut cenum);
                        }

                        #enum_name::parse_from_app(&cenum)
                            .map_err(|err| serde::de::Error::custom(err.msg))?
                            .ok_or(serde::de::Error::custom("Must be selected"))
                    }
                }

                deserializer.deserialize_enum(
                    #enum_name_str,
                    VARIANTS,
                    _Visitor {
                        marker: std::marker::PhantomData,
                        lifetime: std::marker::PhantomData,
                    },
                )
            }
        }
    }
}
