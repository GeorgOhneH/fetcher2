use proc_macro2::TokenStream;
use proc_macro_error::abort;
use quote::{quote, quote_spanned};
use syn::spanned::Spanned;
use syn::{self, DataEnum, Fields, FieldsUnnamed, LitStr};

use crate::build_app::utils::gen_type;
use crate::config_attr::parse_config_attributes;
use crate::config_type::parse_type;

pub fn gen_enum_build_app_fn(e: &DataEnum) -> TokenStream {
    let augmentation = gen_enum_augmentation(e);
    quote! {
        fn builder() -> ::config::CEnumBuilder {
            ::config::CEnumBuilder::new()
            #augmentation
        }
    }
}

fn gen_enum_augmentation(e: &DataEnum) -> TokenStream {
    let data_expanded_members = e.variants.iter().map(|var| {
        let name = LitStr::new(&var.ident.to_string(), var.ident.span());
        let struct_arg = match &var.fields {
            Fields::Unnamed(FieldsUnnamed { unnamed, .. }) if unnamed.len() == 1 => {
                let field = &unnamed[0];
                let config_type = parse_type(&field.ty, &var.attrs);
                let config_attrs = parse_config_attributes(&field.attrs);
                let sup_type = gen_type(&config_type, &config_attrs, field.span(), None);
                quote! { .value(#sup_type) }
            }
            Fields::Unit => {
                quote! {}
            }
            _ => abort!(var.fields, "Only Unit and Single Tuple Enums are allowed"),
        };

        quote_spanned! {var.span()=>
            ::config::CArgBuilder::new(
                #name.to_string(),
            )
            #struct_arg
            .build()
        }
    });

    quote! {
        #(.arg(#data_expanded_members))*
    }
}
