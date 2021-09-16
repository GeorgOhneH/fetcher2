use proc_macro2::TokenStream;
use proc_macro_error::abort;
use quote::quote;
use syn::{self, Field, LitStr};
use syn::spanned::Spanned;

use crate::config_type::{ConfigHashType, ConfigType, ConfigWrapperType};

pub fn gen_set(
    typ: &ConfigType,
    field: &Field,
    match_arg: TokenStream,
    set_arg: TokenStream,
) -> TokenStream {
    match typ {
        ConfigType::String(_) => quote! {{
            match #match_arg {
                ::config::CType::String(ref mut cstring) => {
                    cstring.set(#set_arg);
                    Ok(())
                },
                _ => panic!("This should never happen"),
            }
        }},
        ConfigType::OptionString(_) => quote! {{
            match #match_arg {
                ::config::CType::String(ref mut cstring) => {
                    match #set_arg {
                        Some(str) => cstring.set(str),
                        None => cstring.unset(),
                    };
                    Ok(())
                },
                _ => panic!("This should never happen"),
            }
        }},
        ConfigType::Integer(_) => quote! {{
            match #match_arg {
                ::config::CType::Integer(ref mut cint) => {
                    cint.set(#set_arg)
                },
                _ => panic!("This should never happen"),
            }
        }},
        ConfigType::OptionInteger(_) => quote! {{
            match #match_arg {
                ::config::CType::Integer(ref mut cint) => {
                    match #set_arg {
                        Some(int) => cint.set(int),
                        None => {
                            cint.unset();
                            Ok(())
                        }
                    }
                },
                _ => panic!("This should never happen"),
            }
        }},
        ConfigType::Path(_) => quote! {{
            match #match_arg {
                ::config::CType::Path(ref mut cpath) => {
                    cpath.set(#set_arg)
                },
                _ => panic!("This should never happen"),
            }
        }},
        ConfigType::OptionPath(_) => quote! {{
            match #match_arg {
                ::config::CType::Path(ref mut cpath) => {
                    match #set_arg {
                        Some(path) => cpath.set(path),
                        None => Ok(cpath.unset()),
                    }
                },
                _ => panic!("This should never happen"),
            }
        }},
        ConfigType::Bool(_) => quote! {{
            match #match_arg {
                ::config::CType::Bool(ref mut config_arg_bool) => {
                    Ok(config_arg_bool.set(#set_arg))
                },
                _ => panic!("This should never happen"),
            }
        }},
        ConfigType::OptionBool(_) => quote! {{
            match #match_arg {
                ::config::CType::Bool(ref mut cbool) => {
                    match #set_arg {
                        Some(b) => cbool.set(b),
                        None => cbool.unset(),
                    };
                    Ok(())
                },
                _ => panic!("This should never happen"),
            }
        }},
        ConfigType::Wrapper(_, inner_ty, wrapper_ty) => {
            let inner_setter = gen_set(inner_ty, field, quote! {inner}, quote! {value});
            let unwrap_value = match wrapper_ty {
                ConfigWrapperType::Arc => quote! {Arc::try_unwrap(#set_arg).unwrap()},
                ConfigWrapperType::Mutex => quote! {#set_arg.into_inner().unwrap()},
                ConfigWrapperType::RwLock => quote! {#set_arg.into_inner().unwrap()},
            };
            quote! {{
                match #match_arg {
                    ::config::CType::Wrapper(ref mut cwrapper) => {
                        let mut inner = cwrapper.inner_mut();
                        let value = #unwrap_value;
                        #inner_setter
                    },
                    _ => panic!("This should never happen"),
                }
            }}
        }
        ConfigType::Vec(_, sub_type) => {
            let sub_setter = gen_set(sub_type, field, quote! {temp}, quote! {value});
            quote! {{
                let config_vec = match #match_arg {
                    ::config::CType::Vec(ref mut config_vec) => config_vec,
                    _ => panic!("This should never happen"),
                };
                let a: std::result::Result<Vec<::config::CItem>, ::config::InvalidError> = #set_arg
                    .into_iter()
                    .enumerate()
                    .map(| (idx, value) | {
                        let mut temp = config_vec.get_template();
                        match #sub_setter {
                            Ok(_) => Ok(::config::CItem::new(temp, idx)),
                            Err(err) => Err(err),
                        }
                    })
                    .collect();

                match a {
                    Ok(vec) => {
                        config_vec.set(vec.into());
                        Ok(())
                    },
                    Err(err) => Err(err),
                }
            }}
        }
        ConfigType::HashMap(_, key_ty, value_ty) => {
            let key_setter = gen_hash_set(key_ty, field, quote! {key_temp}, quote! {key});
            let value_setter = gen_set(value_ty, field, quote! {value_temp}, quote! {value});
            quote! {{
                let cmap = match #match_arg {
                    ::config::CType::HashMap(ref mut cmap) => cmap,
                    _ => panic!("This should never happen"),
                };
                let a: std::result::Result<std::collections::HashMap<::config::HashKey, ::config::CType>, ::config::InvalidError> = #set_arg
                    .into_iter()
                    .map(| (key, value) | {
                        let mut key_temp = cmap.get_key();
                        let mut value_temp = cmap.get_value();
                        #key_setter?;
                        #value_setter?;
                        Ok((key_temp, value_temp))
                    })
                    .collect();

                match a {
                    Ok(map) => {
                        cmap.set(map.into());
                        Ok(())
                    },
                    Err(err) => Err(err),
                }
            }}
        }
        ConfigType::Struct(path) => {
            let _struct_name_str = LitStr::new(&quote! {#path}.to_string(), field.span());
            quote! {{
                match #match_arg {
                    ::config::CType::CStruct(ref mut config_struct) => {
                        #path::update_app(#set_arg, config_struct)
                    },
                    _ => panic!("This should never happen"),
                }
            }}
        }
        ConfigType::CheckableStruct(path) => {
            let _struct_name_str = LitStr::new(&quote! {#path}.to_string(), field.span());
            quote! {{
                match #match_arg {
                    ::config::CType::CheckableStruct(ref mut config_check_struct) => {
                        match #set_arg {
                            Some(arg) => {
                                config_check_struct.set_checked(true);
                                #path::update_app(arg, config_check_struct.get_inner_mut())
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
        }
        ConfigType::Enum(path) => {
            let _struct_name_str = LitStr::new(&quote! {#path}.to_string(), field.span());
            quote! {{
                match #match_arg {
                    ::config::CType::CEnum(ref mut cenum) => {
                        #path::update_app(#set_arg, cenum)
                    },
                    _ => panic!("This should never happen"),
                }
            }}
        }
        ConfigType::OptionEnum(path) => {
            let _struct_name_str = LitStr::new(&quote! {#path}.to_string(), field.span());
            quote! {{
                match #match_arg {
                    ::config::CType::CEnum(ref mut cenum) => {
                        match #set_arg {
                            Some(h) => #path::update_app(h, cenum),
                            None =>{
                                cenum.unselect();
                                Ok(())
                            },
                        }
                    },
                    _ => panic!("This should never happen"),
                }
            }}
        }
        ConfigType::Skip(_) => abort!(field.span(), "Skip shouldn't be a possible value"),
    }
}

fn gen_hash_set(
    typ: &ConfigHashType,
    _field: &Field,
    match_arg: TokenStream,
    set_arg: TokenStream,
) -> TokenStream {
    match typ {
        ConfigHashType::String => quote! {{
            match #match_arg {
                ::config::HashKey::String(ref mut str) => {
                    *str = #set_arg;
                    Ok(())
                },
                _ => panic!("This should never happen"),
            }
        }},
        ConfigHashType::Path => quote! {{
            match #match_arg {
                ::config::HashKey::Path(ref mut path) => {
                    *path = #set_arg;
                    Ok(())
                },
                _ => panic!("This should never happen"),
            }
        }},
    }
}
