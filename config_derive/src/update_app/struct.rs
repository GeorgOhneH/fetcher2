use proc_macro2::TokenStream;

use proc_macro_error::abort;
use quote::quote;
use syn::{
    self, punctuated::Punctuated, token::Comma, DataEnum, Field, Fields, FieldsUnnamed, LitStr,
};

use crate::config_type::{parse_type, ConfigHashType, ConfigType};
use crate::update_app::utils::gen_set;
use syn::spanned::Spanned;

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
