use proc_macro2::TokenStream;

use proc_macro_error::abort;
use quote::quote;
use syn::{
    self, DataEnum, Fields, FieldsUnnamed, LitStr,
};

use crate::config_type::{parse_type};
use crate::update_app::utils::gen_set;

pub fn gen_enum_update_app_fn(e: &DataEnum) -> TokenStream {
    let augmentation = gen_carg(e);
    quote! {
        fn update_app(self, cenum: &mut ::config::CEnum) -> std::result::Result<(), ::config::InvalidError> {
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
                let config_type = parse_type(&field.ty, &var.attrs);
                let config = gen_set(
                    &config_type,
                    field,
                    quote! {carg.get_mut().unwrap()},
                    quote! {ctype},
                );
                quote! {
                    Self::#name(ctype) => {
                        let carg = cenum.set_selected_mut(#name_lit.to_string()).unwrap();
                        #config
                    }
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
