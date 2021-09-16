use proc_macro2::TokenStream;
use quote::quote;
use syn::{self, Field, LitStr, punctuated::Punctuated, token::Comma};

use crate::config_type::parse_type;
use crate::parse_from_app::utils::gen_arg;

pub fn gen_struct_parse_fn(fields: &Punctuated<Field, Comma>) -> TokenStream {
    let augmentation = gen_kwargs(fields);
    quote! {
        fn parse_from_app(app: &::config::CStruct) -> std::result::Result<Self, ::config::RequiredError> {
            #augmentation
        }
    }
}

fn gen_kwargs(fields: &Punctuated<Field, Comma>) -> TokenStream {
    let keywords: Vec<TokenStream> = fields
        .iter()
        .map(|field| {
            let field_name = field.ident.as_ref().expect("Unreachable");
            quote! {#field_name}
        })
        .collect();
    let args: Vec<TokenStream> = fields
        .iter()
        .map(|field| {
            let field_name = field.ident.as_ref().expect("Unreachable");
            let field_name_str = LitStr::new(&field_name.to_string(), field_name.span());
            let match_arg = quote! {app.get_ty(&#field_name_str.to_string()).unwrap()};
            let typ = parse_type(&field.ty, &field.attrs);
            gen_arg(&typ, match_arg, field_name.span(), &field_name_str)
        })
        .collect();

    quote! {
        #(
            let #keywords = #args?;
        )*
        Ok(Self {
            #(
                #keywords,
            )*
        })
    }
}
