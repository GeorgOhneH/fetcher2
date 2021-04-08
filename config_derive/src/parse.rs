use std::iter::FromIterator;

use proc_macro_error::{abort, ResultExt};
use quote::ToTokens;
use syn::Token;
use syn::{
    self, parenthesized,
    parse::{Parse, ParseBuffer, ParseStream},
    punctuated::Punctuated,
    Attribute, Expr, ExprLit, Ident, Lit, LitBool, LitStr,
};

#[allow(clippy::large_enum_variant)]
pub enum ConfigAttr {
    // single-identifier attributes
    OtherSingle(Ident),

    // ident = "string literal"
    GuiName(Ident, LitStr),
    OtherLitStr(Ident, LitStr),

    // ident = arbitrary_expr
    ActiveFn(Ident, Expr),
    InactiveBehavior(Ident, Expr),
    Other(Ident, Expr),
}

impl Parse for ConfigAttr {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        use self::ConfigAttr::*;

        let name: Ident = input.parse()?;
        let name_str = name.to_string();

        if name_str == "required" {
            abort! {
                name,
                "use Option<> to set required to false"
            }
        }

        if input.peek(Token![=]) {
            // `name = value` attributes.
            let assign_token = input.parse::<Token![=]>()?; // skip '='

            if input.peek(LitStr) {
                let lit: LitStr = input.parse()?;

                match &*name_str {
                    "gui_name" => Ok(GuiName(name, lit)),

                    _ => Ok(OtherLitStr(name, lit)),
                }
            } else {
                match input.parse::<Expr>() {
                    Ok(expr) => match &*name_str {
                        "active_fn" => Ok(ActiveFn(name, expr)),
                        "inactive_behavior" => Ok(InactiveBehavior(name, expr)),
                        _ => Ok(Other(name, expr)),
                    },

                    Err(_) => abort! {
                        assign_token,
                        "expected `string literal` or `expression` after `=`"
                    },
                }
            }
        } else if input.peek(syn::token::Paren) {
            // `name(...)` attributes.
            abort!(name, "nested attributes are not valid")
        } else {
            // Attributes represented with a sole identifier.
            match name_str {
                _ => Ok(OtherSingle(name)),
            }
        }
    }
}

pub fn parse_clap_attributes(all_attrs: &[Attribute]) -> Vec<ConfigAttr> {
    all_attrs
        .iter()
        .filter(|attr| attr.path.is_ident("config"))
        .flat_map(|attr| {
            attr.parse_args_with(Punctuated::<ConfigAttr, Token![,]>::parse_terminated)
                .unwrap_or_abort()
        })
        .collect()
}
