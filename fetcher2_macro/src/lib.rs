use proc_macro::TokenStream;
use std::sync::Mutex;

use convert_case::{Case, Casing};
use once_cell::sync::Lazy;
use proc_macro2::{Ident, Span};
use quote::quote;
use syn::parse::Parser;
use syn::{self, parse_macro_input, DeriveInput, Field, Fields, Item};

static ENUM_DEFS: Lazy<Mutex<Vec<String>>> = Lazy::new(|| Mutex::new(Vec::new()));

#[proc_macro_attribute]
pub fn login_locks(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let item_c = item.clone();
    match parse_macro_input!(item_c as Item) {
        Item::Enum(item_enum) => {
            let mut enum_vec = ENUM_DEFS.lock().unwrap();
            for variant in item_enum.variants {
                enum_vec.push(variant.ident.to_string());
            }
            item.into()
        }
        Item::Struct(ref mut item_struct) => {
            match &mut item_struct.fields {
                Fields::Named(fields) => {
                    let lock = ENUM_DEFS.lock().unwrap();
                    for x in lock.iter() {
                        let lock_name = Ident::new(&x.to_case(Case::Snake), Span::call_site());
                        fields.named.push(
                            Field::parse_named
                                .parse2(quote! { pub #lock_name: Mutex<LoginState> })
                                .unwrap(),
                        );
                    }
                }
                _ => (),
            };

            quote! {
                #item_struct
            }
        }
        _ => panic!("Must be enum or function"),
    }
    .into()
}

#[proc_macro_derive(LoginLock)]
pub fn derive_login_lock(input: TokenStream) -> TokenStream {
    let _input: DeriveInput = parse_macro_input!(input);

    let arms: Vec<_> = ENUM_DEFS
        .lock()
        .unwrap()
        .iter()
        .map(|x| {
            let lock_name = Ident::new(&x.to_case(Case::Snake), Span::call_site());
            let name = Ident::new(&x, Span::call_site());
            quote! { Module::#name(_) => locks.#lock_name.lock().await }
        })
        .collect();

    let output = quote! {
        impl Module {
            pub async fn get_lock<'a>(&self, locks: &'a LoginLocks) -> MutexGuard<'a, LoginState> {
                match self {
                    #(#arms,)*
                }
            }
        }
    };

    output.into()
}
