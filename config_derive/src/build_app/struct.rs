use crate::config_attr::{parse_config_attributes, ConfigAttr};
use crate::build_app::utils::{attrs_to_args, attrs_to_sub_args};
use crate::build_app::utils::gen_type;
use proc_macro2::TokenStream;

use proc_macro_error::abort;
use quote::{quote, quote_spanned};
use syn::{
    self, punctuated::Punctuated, token::Comma, DataEnum, Field, Fields, FieldsUnnamed, LitStr,
};

use crate::config_type::{parse_type, ConfigHashType, ConfigType};
use syn::spanned::Spanned;

pub fn gen_struct_build_app_fn(fields: &Punctuated<Field, Comma>) -> TokenStream {
    let augmentation = gen_app_augmentation(fields);
    quote! {
        fn build_app() -> ::config::CStruct {
            ::config::CStructBuilder::new()
            #augmentation
            .build()
        }
    }
}

fn gen_app_augmentation(fields: &Punctuated<Field, Comma>) -> TokenStream {
    let data_expanded_members = fields.iter().map(|field| {
        let typ = parse_type(&field.ty, &field.attrs);
        gen_arg(field, &typ)
    });

    quote! {
        #(.arg(#data_expanded_members))*
    }
}

fn gen_arg(field: &Field, typ: &ConfigType) -> TokenStream {
    let field_name = field.ident.as_ref().expect("Unreachable");
    let span = field_name.span();
    let config_attrs = parse_config_attributes(&field.attrs);
    let builder_args = attrs_to_args(&config_attrs);
    let sup_type = gen_type(typ, &config_attrs, field.span());
    let name = LitStr::new(&field_name.to_string(), span);
    let is_required = typ.is_inside_option();
    quote_spanned! {span=>
        ::config::CKwargBuilder::new(
            #name.to_string(),
            #sup_type
        )
        .required(#is_required)
        #builder_args
        .build()
    }
}



