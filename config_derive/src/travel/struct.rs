use proc_macro2::{Ident, TokenStream};
use quote::quote;
use syn::punctuated::Punctuated;
use syn::Field;
use syn::{self, token::Comma};
use syn::spanned::Spanned;
use syn::LitStr;

pub fn gen_travel(fields: &Punctuated<Field, Comma>, _name: &Ident) -> TokenStream {
    let gen_founds = gen_founds(fields);

    quote! {
        use ::config::traveller::TravellerStruct as _;
        let mut state = traveller.found_struct()?;
        #(#gen_founds)*
        state.end()
    }
}

fn gen_founds(fields: &Punctuated<Field, Comma>) -> Vec<TokenStream> {
    fields
        .iter()
        .map(|field| {
            let field_name = field.ident.as_ref().expect("Unreachable");
            let name = LitStr::new(&field_name.to_string(), field.span());
            let field_ty = &field.ty;
            quote! {
                state.found_field::<#field_ty>(#name)?;
            }
        })
        .collect()
}
