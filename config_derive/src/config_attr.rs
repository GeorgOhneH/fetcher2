use proc_macro2::Span;
use proc_macro_error::{abort, ResultExt};
use syn::{
    self,
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
    Attribute, Expr, Ident, LitStr,
};
use syn::{MetaNameValue, Token};

pub struct TravelAttr {
    pub skip: Option<Ident>,
    pub name: Option<(Ident, LitStr)>,
    pub default: Option<(Ident, Expr)>,
}

impl TravelAttr {
    pub fn new() -> Self {
        Self {
            skip: None,
            name: None,
            default: None,
        }
    }

    fn got_default(&mut self, ident: Ident, expr: Expr) {
        if self.default.is_some() {
            abort! {
                ident,
                "default is only allowed once"
            }
        }
        self.default = Some((ident, expr))
    }

    fn got_name(&mut self, ident: Ident, lit_str: LitStr) {
        if self.name.is_some() {
            abort! {
                ident,
                "name is only allowed once"
            }
        }
        self.name = Some((ident, lit_str))
    }

    fn got_skip(&mut self, ident: Ident) {
        if self.skip.is_some() {
            abort! {
                ident,
                "skip is only allowed once"
            }
        }
        self.skip = Some(ident)
    }
}

#[allow(clippy::large_enum_variant)]
pub enum TravelAttrKind {
    DocString(LitStr),
    // single-identifier attributes
    Skip(Ident),
    OtherSingle(Ident),

    // ident = "string literal"
    Name(Ident, LitStr),
    OtherLitStr(Ident, LitStr),

    // ident = arbitrary_expr
    Default(Ident, Expr),
    Other(Ident, Expr),
}

impl Parse for TravelAttrKind {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        use self::TravelAttrKind::*;

        let name: Ident = input.parse()?;
        let name_str = name.to_string();

        if input.peek(Token![=]) {
            // `name = value` attributes.
            let assign_token = input.parse::<Token![=]>()?; // skip '='

            if input.peek(LitStr) {
                let lit: LitStr = input.parse()?;

                match &*name_str {
                    "name" => Ok(Name(name, lit)),

                    _ => Ok(OtherLitStr(name, lit)),
                }
            } else {
                match input.parse::<Expr>() {
                    Ok(expr) => match &*name_str {
                        "default" => Ok(Default(name, expr)),
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
            match &*name_str {
                "skip" => Ok(Skip(name)),
                _ => Ok(OtherSingle(name)),
            }
        }
    }
}

fn push_hint_text_comment(config_attrs: &mut Vec<TravelAttrKind>, attrs: &[Attribute]) {
    use syn::Lit::*;
    use syn::Meta::*;
    let doc_parts: Vec<String> = attrs
        .iter()
        .filter_map(|attr| {
            if let Ok(NameValue(MetaNameValue { lit: Str(s), .. })) = attr.parse_meta() {
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
        config_attrs.push(TravelAttrKind::DocString(LitStr::new(
            &doc_str,
            Span::call_site(),
        )));
    }
}

pub fn parse_config_attributes(all_attrs: &[Attribute]) -> TravelAttr {
    let mut attr = TravelAttr::new();
    for attr_kind in all_attrs
        .iter()
        .filter(|attr| attr.path.is_ident("travel"))
        .flat_map(|attr| {
            attr.parse_args_with(Punctuated::<TravelAttrKind, Token![,]>::parse_terminated)
                .unwrap_or_abort()
        })
    {
        match attr_kind {
            TravelAttrKind::Default(ident, expr) => attr.got_default(ident, expr),
            TravelAttrKind::Name(ident, lit) => attr.got_name(ident, lit),
            TravelAttrKind::Skip(ident) => attr.got_skip(ident),
            TravelAttrKind::Other(ident, _expr) => abort!(ident, "todo: make this sound cool"),
            TravelAttrKind::OtherSingle(ident) => abort!(ident, "todo: make this sound cool"),
            TravelAttrKind::OtherLitStr(ident, _lit) => abort!(ident, "todo: make this sound cool"),
            TravelAttrKind::DocString(_) => (), // TODO
        }
    }

    // push_hint_text_comment(&mut config_attrs, all_attrs);
    attr
}
