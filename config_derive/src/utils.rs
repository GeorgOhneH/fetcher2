use proc_macro2::Span;
use proc_macro2::TokenStream;
use quote::quote;
use syn::spanned::Spanned;
use syn::GenericParam;
use syn::LitStr;
use syn::{
    self, punctuated::Punctuated, token::Comma, AngleBracketedGenericArguments, Field,
    GenericArgument, Generics, Ident, Lifetime, Path, PathArguments, TraitBound, TypeParamBound,
};
use syn::{LifetimeDef, PathSegment};


pub fn bound_generics(mut generics: Generics, bound: TraitBound) -> Generics {
    for param in generics.params.iter_mut() {
        match param {
            GenericParam::Type(ty_param) => {
                ty_param.bounds.push(TypeParamBound::Trait(bound.clone()))
            }
            _ => (),
        }
    }
    generics
}

pub fn create_path(parts: &[&str], span: Span) -> Path {
    let mut path = Path {
        leading_colon: None,
        segments: Default::default(),
    };
    for part in parts {
        let segment = Ident::new(part, span).into();
        path.segments.push(segment);
    }
    path
}

pub fn lifetime_generics(mut generics: Generics, symbol: &str) -> Generics {
    let de_lifetime = GenericParam::Lifetime(LifetimeDef {
        attrs: vec![],
        lifetime: Lifetime::new(symbol, generics.span()),
        colon_token: None,
        bounds: Default::default(),
    });
    generics.params.push(de_lifetime);
    generics
}
