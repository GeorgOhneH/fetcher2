use proc_macro2::TokenStream;
use quote::quote;
use syn::{self, punctuated::Punctuated, token::Comma, Field, LitStr};

use crate::config_type::{parse_type, ConfigType};
use crate::update_app::utils::gen_set;

pub fn gen_struct_update_app_fn(fields: &Punctuated<Field, Comma>) -> TokenStream {
    let augmentation = gen_setter(fields);
    quote! {
        fn update_app(self, app: &mut ::config::CStruct) -> std::result::Result<(), ::config::InvalidError> {
            #augmentation
        }
    }
}

fn gen_setter(fields: &Punctuated<Field, Comma>) -> TokenStream {
    let setters: Vec<TokenStream> = fields
        .iter()
        .filter_map(|field| {
            let typ = parse_type(&field.ty, &field.attrs);
            if let ConfigType::Skip(_) = typ {
                return None;
            }

            let field_name = field.ident.as_ref().expect("Unreachable");
            let field_name_str = LitStr::new(&field_name.to_string(), field_name.span());
            let match_arg = quote! {app.get_ty_mut(#field_name_str).unwrap()};
            let set_arg = quote! {self.#field_name};
            Some(gen_set(&typ, field, match_arg, set_arg))
        })
        .collect();

    quote! {
        let results: Vec<std::result::Result<(), ::config::InvalidError>> = vec![#(
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
