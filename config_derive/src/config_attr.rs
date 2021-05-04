use proc_macro_error::{abort, ResultExt};

use proc_macro2::Span;
use syn::{
    self,
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
    Attribute, Expr, Ident, LitStr,
};
use syn::{MetaNameValue, Token};

#[allow(clippy::large_enum_variant)]
pub enum ConfigAttr {
    DocString(LitStr),

    // single-identifier attributes
    OtherSingle(Ident),

    // ident = "string literal"
    GuiName(Ident, LitStr),
    Type(Ident, LitStr),
    InnerType(Ident, LitStr),
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
                    "ty" => Ok(Type(name, lit)),
                    "inner_ty" => Ok(InnerType(name, lit)),

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

fn push_hint_text_comment(config_attrs: &mut Vec<ConfigAttr>, attrs: &[Attribute]) {
    use syn::Lit::*;
    use syn::Meta::*;
    let doc_parts: Vec<String> = attrs
        .iter()
        .filter_map(|attr| {
            if let Ok(NameValue(MetaNameValue { lit: Str(s), .. })) = attr.parse_meta() {
                //emit_call_site_warning! { " efigef"}
                Some(s.value().trim().to_string())
            } else {
                // non #[doc = "..."] attributes are not our concern
                // we leave them for rustc to handle
                None
            }
        })
        .collect();

    let doc_str = doc_parts.join("\n").trim().to_string();
    if !doc_str.is_empty() {
        config_attrs.push(ConfigAttr::DocString(LitStr::new(
            &doc_str,
            Span::call_site(),
        )));
    }
}

pub fn parse_config_attributes(all_attrs: &[Attribute]) -> Vec<ConfigAttr> {
    let mut config_attrs: Vec<ConfigAttr> = all_attrs
        .iter()
        .filter(|attr| attr.path.is_ident("config"))
        .flat_map(|attr| {
            attr.parse_args_with(Punctuated::<ConfigAttr, Token![,]>::parse_terminated)
                .unwrap_or_abort()
        })
        .collect();

    push_hint_text_comment(&mut config_attrs, all_attrs);
    config_attrs
}
