use crate::config_attr::{parse_clap_attributes, ConfigAttr};

use proc_macro2::{Ident, Span, TokenStream};

use proc_macro_error::abort;
use quote::{quote, quote_spanned};
use syn::{self, punctuated::Punctuated, token::Comma, Field, LitStr};

use crate::derives::{convert_type, SupportedTypes};
use syn::spanned::Spanned;

pub fn gen_update_app_fn(fields: &Punctuated<Field, Comma>) -> TokenStream {
    let augmentation = gen_setter(fields);
    quote! {
        fn update_app(self, app: &mut ::config::ConfigStruct) -> Result<(), ::config::ValueError> {
            #augmentation
        }
    }
}

fn gen_setter(fields: &Punctuated<Field, Comma>) -> TokenStream {
    let setters: Vec<TokenStream> = fields
        .iter()
        .map(|field| {
            let typ = convert_type(&field.ty);

            let field_name = field.ident.as_ref().expect("Unreachable");
            let field_name_str = LitStr::new(&field_name.to_string(), field_name.span());
            let match_arg = quote! {app.get_ty_mut(#field_name_str).unwrap()};
            let set_arg = quote! {self.#field_name};
            gen_set(&typ, field, match_arg, set_arg)
        })
        .collect();

    quote! {
        let results = vec![#(
            #setters
        ),*];
        for result in results {
            if let Err(err) = result {
                return Err(err)
            }
        }
        Ok(())
    }
}

fn gen_set(
    typ: &SupportedTypes,
    field: &Field,
    match_arg: TokenStream,
    set_arg: TokenStream,
) -> TokenStream {
    match typ {
        SupportedTypes::String => quote! {{
            match #match_arg {
                ::config::SupportedTypes::String(ref mut config_arg_string) => {
                    Ok(config_arg_string.set(Some(#set_arg)))
                },
                _ => panic!("This should never happen"),
            }
        }},
        SupportedTypes::OtherString => quote! {{
            match #match_arg {
                ::config::SupportedTypes::String(ref mut config_arg_string) => {
                    Ok(config_arg_string.set(#set_arg))
                },
                _ => panic!("This should never happen"),
            }
        }},
        SupportedTypes::Integer => quote! {{
            match #match_arg {
                ::config::SupportedTypes::Integer(ref mut config_arg_int) => {
                    config_arg_int.set(Some(#set_arg))
                },
                _ => panic!("This should never happen"),
            }
        }},
        SupportedTypes::OtherInteger => quote! {{
            match #match_arg {
                ::config::SupportedTypes::Integer(ref mut config_arg_int) => {
                    config_arg_int.set(#set_arg)
                },
                _ => panic!("This should never happen"),
            }
        }},
        SupportedTypes::Bool => quote! {{
            match #match_arg {
                ::config::SupportedTypes::Bool(ref mut config_arg_bool) => {
                    Ok(config_arg_bool.set(Some(#set_arg)))
                },
                _ => panic!("This should never happen"),
            }
        }},
        SupportedTypes::OtherBool => quote! {{
            match #match_arg {
                ::config::SupportedTypes::Bool(ref mut config_arg_bool) => {
                    Ok(config_arg_bool.set(#set_arg))
                },
                _ => panic!("This should never happen"),
            }
        }},
        SupportedTypes::Vec(sub_type) => {
            let sub_setter = gen_set(sub_type, field, quote! {temp}, quote! {value});
            quote! {{
                let mut config_vec = match #match_arg {
                    ::config::SupportedTypes::Vec(ref mut config_vec) => config_vec,
                    _ => panic!("This should never happen"),
                };
                let a: Result<Vec<::config::SupportedTypes>, ::config::ValueError> = #set_arg
                    .into_iter()
                    .map(| value | {
                        let mut temp = config_vec.get_template().clone();
                        match #sub_setter {
                            Ok(_) => Ok(temp),
                            Err(err) => Err(err),
                        }
                    })
                    .collect();

                match a {
                    Ok(vec) => config_vec.set(vec),
                    Err(err) => Err(err),
                }
            }}
        },
        SupportedTypes::Struct(ty_path) => {
            let path = &ty_path.path;
            let struct_name_str = LitStr::new(&quote! {#path}.to_string(), field.span());
            quote! {{
                match #match_arg {
                    ::config::SupportedTypes::Struct(ref mut config_struct) => {
                        #ty_path::update_app(#set_arg, config_struct)
                    },
                    _ => panic!("This should never happen"),
                }
            }}
        },
        SupportedTypes::CheckableStruct(ty_path) => {
            let path = &ty_path.path;
            let struct_name_str = LitStr::new(&quote! {#path}.to_string(), field.span());
            quote! {{
                match #match_arg {
                    ::config::SupportedTypes::CheckableStruct(ref mut config_check_struct) => {
                        match #set_arg {
                            Some(arg) => {
                                config_check_struct.set_checked(true);
                                #ty_path::update_app(arg, config_check_struct.get_inner_mut())
                            },
                            None => {
                                config_check_struct.set_checked(false);
                                Ok(())
                            }
                        }
                    },
                    _ => panic!("This should never happen"),
                }
            }}
        },
    }
}
