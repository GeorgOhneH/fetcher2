use proc_macro2::TokenStream;

use proc_macro_error::abort;
use quote::quote;
use syn::{
    self, punctuated::Punctuated, token::Comma, DataEnum, Field, Fields, FieldsUnnamed, LitStr,
};

use crate::derives::{parse_type, SupportedTypes, HashType};
use syn::spanned::Spanned;

pub fn gen_struct_update_app_fn(fields: &Punctuated<Field, Comma>) -> TokenStream {
    let augmentation = gen_setter(fields);
    quote! {
        fn update_app(self, app: &mut ::config::CStruct) -> Result<(), ::config::InvalidError> {
            #augmentation
        }
    }
}

pub fn gen_enum_update_app_fn(e: &DataEnum) -> TokenStream {
    let augmentation = gen_carg(e);
    quote! {
        fn update_app(self, cenum: &mut ::config::CEnum) -> Result<(), ::config::InvalidError> {
            #augmentation
        }
    }
}

fn gen_carg(e: &DataEnum) -> TokenStream {
    let data_expanded_members = e.variants.iter().map(|var| {
        let name = &var.ident;
        let name_lit = LitStr::new(&name.to_string(), var.ident.span());
        match &var.fields {
            Fields::Unnamed(FieldsUnnamed { unnamed, .. }) if unnamed.len() == 1 => {
                let field = &unnamed[0];
                let typ = parse_type(&field.ty, &var.attrs);
                if let SupportedTypes::Struct(_) = typ {
                    quote! {
                        Self::#name(cstruct) => {
                            let carg = cenum.set_selected_mut(#name_lit.to_string()).unwrap();
                            #typ::update_app(cstruct, carg.get_mut().unwrap())
                        }
                    }
                } else {
                    abort!(var.fields, "Only Structs are allowed")
                }
            }
            Fields::Unit => {
                quote! {
                    Self::#name => {
                        cenum.set_selected(#name_lit.to_string()).unwrap();
                        Ok(())
                    }
                }
            }
            _ => abort!(var.fields, "Only Structs are allowed"),
        }
    });

    quote! {
        match self {
            #(#data_expanded_members),*
        }
    }
}

fn gen_setter(fields: &Punctuated<Field, Comma>) -> TokenStream {
    let setters: Vec<TokenStream> = fields
        .iter()
        .map(|field| {
            let typ = parse_type(&field.ty, &field.attrs);

            let field_name = field.ident.as_ref().expect("Unreachable");
            let field_name_str = LitStr::new(&field_name.to_string(), field_name.span());
            let match_arg = quote! {app.get_ty_mut(#field_name_str).unwrap()};
            let set_arg = quote! {self.#field_name};
            gen_set(&typ, field, match_arg, set_arg)
        })
        .collect();

    quote! {
        let results: Vec<Result<(), ::config::InvalidError>> = vec![#(
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
                ::config::CTypes::String(ref mut cstring) => {
                    cstring.set(#set_arg);
                    Ok(())
                },
                _ => panic!("This should never happen"),
            }
        }},
        SupportedTypes::OptionString => quote! {{
            match #match_arg {
                ::config::CTypes::String(ref mut cstring) => {
                    match #set_arg {
                        Some(str) => cstring.set(str),
                        None => cstring.unset(),
                    };
                    Ok(())
                },
                _ => panic!("This should never happen"),
            }
        }},
        SupportedTypes::Integer => quote! {{
            match #match_arg {
                ::config::CTypes::Integer(ref mut cint) => {
                    cint.set(#set_arg)
                },
                _ => panic!("This should never happen"),
            }
        }},
        SupportedTypes::OptionInteger => quote! {{
            match #match_arg {
                ::config::CTypes::Integer(ref mut cint) => {
                    match #set_arg {
                        Some(int) => cint.set(int),
                        None => cint.unset(),
                    }
                },
                _ => panic!("This should never happen"),
            }
        }},
        SupportedTypes::Path => quote! {{
            match #match_arg {
                ::config::CTypes::Path(ref mut cpath) => {
                    cpath.set(#set_arg)
                },
                _ => panic!("This should never happen"),
            }
        }},
        SupportedTypes::OptionPath => quote! {{
            match #match_arg {
                ::config::CTypes::Path(ref mut cpath) => {
                    match #set_arg {
                        Some(path) => cpath.set(path),
                        None => cpath.unset(),
                    }
                },
                _ => panic!("This should never happen"),
            }
        }},
        SupportedTypes::Bool => quote! {{
            match #match_arg {
                ::config::CTypes::Bool(ref mut config_arg_bool) => {
                    Ok(config_arg_bool.set(#set_arg))
                },
                _ => panic!("This should never happen"),
            }
        }},
        SupportedTypes::OptionBool => quote! {{
            match #match_arg {
                ::config::CTypes::Bool(ref mut cbool) => {
                    match #set_arg {
                        Some(b) => cbool.set(b),
                        None => cbool.unset(),
                    };
                    Ok(())
                },
                _ => panic!("This should never happen"),
            }
        }},
        SupportedTypes::Vec(sub_type) => {
            let sub_setter = gen_set(sub_type, field, quote! {temp}, quote! {value});
            quote! {{
                let config_vec = match #match_arg {
                    ::config::CTypes::Vec(ref mut config_vec) => config_vec,
                    _ => panic!("This should never happen"),
                };
                let a: Result<Vec<::config::CTypes>, ::config::InvalidError> = #set_arg
                    .into_iter()
                    .map(| value | {
                        let mut temp = config_vec.get_template();
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
        SupportedTypes::HashMap(key_ty, value_ty) => {
            let key_setter = gen_set(key_ty, field, quote! {key_temp}, quote! {key});
            let value_setter = gen_set(value_ty, field, quote! {value_temp}, quote! {value});
            quote! {{
                let cmap = match #match_arg {
                    ::config::CTypes::HashMap(ref mut cmap) => cmap,
                    _ => panic!("This should never happen"),
                };
                let a: Result<HashMap<::config::CTypes, ::config::CTypes>, ::config::InvalidError> = #set_arg
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
                    Ok(map) => cmap.set(map),
                    Err(err) => Err(err),
                }
            }}
        },
        SupportedTypes::Struct(ty_path) => {
            let path = &ty_path.path;
            let _struct_name_str = LitStr::new(&quote! {#path}.to_string(), field.span());
            quote! {{
                match #match_arg {
                    ::config::CTypes::Struct(ref mut config_struct) => {
                        #ty_path::update_app(#set_arg, config_struct)
                    },
                    _ => panic!("This should never happen"),
                }
            }}
        }
        SupportedTypes::CheckableStruct(ty_path) => {
            let path = &ty_path.path;
            let _struct_name_str = LitStr::new(&quote! {#path}.to_string(), field.span());
            quote! {{
                match #match_arg {
                    ::config::CTypes::CheckableStruct(ref mut config_check_struct) => {
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
        }
        SupportedTypes::Enum(ty_path) => {
            let path = &ty_path.path;
            let _struct_name_str = LitStr::new(&quote! {#path}.to_string(), field.span());
            quote! {{
                match #match_arg {
                    ::config::CTypes::Enum(ref mut cenum) => {
                        #ty_path::update_app(#set_arg, cenum)
                    },
                    _ => panic!("This should never happen"),
                }
            }}
        }
        SupportedTypes::OptionEnum(ty_path) => {
            let path = &ty_path.path;
            let _struct_name_str = LitStr::new(&quote! {#path}.to_string(), field.span());
            quote! {{
                match #match_arg {
                    ::config::CTypes::Enum(ref mut cenum) => {
                        match #set_arg {
                            Some(h) => #ty_path::update_app(h, cenum),
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
    }
}

fn gen_set(
    typ: &HashType,
    field: &Field,
    match_arg: TokenStream,
    set_arg: TokenStream,
) -> TokenStream {
    match typ {
        SupportedTypes::String => quote! {{
            match #match_arg {
                ::config::CTypes::String(ref mut cstring) => {
                    cstring.set(#set_arg);
                    Ok(())
                },
                _ => panic!("This should never happen"),
            }
        }},
        SupportedTypes::OptionString => quote! {{
            match #match_arg {
                ::config::CTypes::String(ref mut cstring) => {
                    match #set_arg {
                        Some(str) => cstring.set(str),
                        None => cstring.unset(),
                    };
                    Ok(())
                },
                _ => panic!("This should never happen"),
            }
        }},
    }
}
