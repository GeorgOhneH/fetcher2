use crate::utils::{gen_field_name_strs, gen_field_names};
use proc_macro2::{Ident, TokenStream};
use quote::quote;
use syn::punctuated::Punctuated;
use syn::spanned::Spanned;
use syn::Field;
use syn::{self, token::Comma, LitStr};

pub fn gen_se(fields: &Punctuated<Field, Comma>, name: &Ident) -> TokenStream {
    let name_str = LitStr::new(&name.to_string(), name.span());
    let field_names = gen_field_names(fields);
    let field_name_strings = gen_field_name_strs(fields);
    let num_fields = field_names.len();

    quote! {
        use serde::ser::SerializeStruct as _;
        let mut state = serializer.serialize_struct(#name_str, #num_fields)?;
        #(state.serialize_field(#field_name_strings, &self.#field_names)?;)*
        state.end()
    }
}
