use proc_macro2::Span;
use proc_macro2::TokenStream;
use quote::quote;
use syn::{
    self, AngleBracketedGenericArguments,
    Field, GenericArgument, Generics, Ident, Lifetime, Path, PathArguments, punctuated::Punctuated,
    token::Comma, TraitBound, TypeParamBound,
};
use syn::{LifetimeDef, PathSegment};
use syn::GenericParam;
use syn::LitStr;
use syn::spanned::Spanned;


use crate::config_type::{ConfigType, parse_type};

pub fn gen_field_names(fields: &Punctuated<Field, Comma>) -> Vec<TokenStream> {
    fields
        .iter()
        .filter_map(|field| {
            let typ = parse_type(&field.ty, &field.attrs);
            if let ConfigType::Skip(_) = typ {
                None
            } else {
                Some(gen_field_name(field))
            }
        })
        .collect()
}

pub fn gen_field_name_strs(fields: &Punctuated<Field, Comma>) -> Vec<TokenStream> {
    fields
        .iter()
        .filter_map(|field| {
            let typ = parse_type(&field.ty, &field.attrs);
            if let ConfigType::Skip(_) = typ {
                None
            } else {
                Some(gen_field_name_string(field))
            }
        })
        .collect()
}

pub fn gen_field_name(field: &Field) -> TokenStream {
    let field_name = field.ident.as_ref().expect("Unreachable");
    quote! { #field_name }
}

pub fn gen_field_name_string(field: &Field) -> TokenStream {
    let field_name = field.ident.as_ref().expect("Unreachable");
    let name = LitStr::new(&field_name.to_string(), field.span());
    quote! { #name }
}

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

pub fn create_path(parts: &[(&str, Option<&str>)], span: Span) -> Path {
    let mut path = Path {
        leading_colon: None,
        segments: Default::default(),
    };
    for (part, lifetime) in parts {
        let segment = if let Some(lifetime) = lifetime {
            let lifetime =
                GenericArgument::Lifetime(Lifetime::new(lifetime, span));
            let mut p = Punctuated::new();
            p.push(lifetime);
            let x = AngleBracketedGenericArguments {
                colon2_token: None,
                lt_token: Default::default(),
                args: p,
                gt_token: Default::default(),
            };
            PathSegment {
                ident: Ident::new("Deserialize", span),
                arguments: PathArguments::AngleBracketed(x),
            }
        } else {
            Ident::new(part, span).into()
        };
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
