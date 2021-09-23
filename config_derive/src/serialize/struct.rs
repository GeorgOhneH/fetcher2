use proc_macro2::{Ident, TokenStream};
use quote::quote;
use syn::{self, LitStr, token::Comma};
use syn::Field;
use syn::punctuated::Punctuated;
use syn::spanned::Spanned;

use crate::utils::{gen_field_name_strs, gen_field_names};

pub fn gen_se(fields: &Punctuated<Field, Comma>, name: &Ident) -> TokenStream {
    let name_str = LitStr::new(&name.to_string(), name.span());
    let field_names = gen_field_names(fields);
    let field_name_strings = gen_field_name_strs(fields);
    let num_fields = field_names.len();

    quote! {
        use serde::ser::SerializeMap as _;
        let mut state = serializer.serialize_map(Some(#num_fields))?;
        #(state.serialize_entry(#field_name_strings, &self.#field_names)?;)*
        state.end()
    }
}
