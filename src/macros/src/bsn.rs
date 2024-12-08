use proc_macro2::{Ident, TokenStream};
use quote::{quote, ToTokens};
use syn::{
    braced, bracketed, parenthesized,
    parse::{Parse, ParseStream},
    parse2,
    punctuated::Punctuated,
    token::{self, Brace, Paren},
    Expr, Member, Path, Result, Token,
};

// TODO: Better rust-analyzer support

// TODO: Support nested constructs? E.g:
// bsn! {
//    ConstructOuter {
//        some_prop: ConstructInner {
//            inner_prop: @"asset.txt",
//        }
//    }
// Might be the reason that braces are required for expressions in the first place.

pub fn bsn(item: TokenStream) -> TokenStream {
    match parse2::<BsnEntity>(item) {
        Ok(bsn) => bsn.into_token_stream(),
        Err(e) => e.to_compile_error(), // TODO: Don't just bail here? Output as much as possible to keep RA happy.
    }
}

pub struct BsnEntity {
    pub inherits: Punctuated<BsnInherit, Token![,]>,
    pub patch: BsnPatch,
    pub children: Punctuated<BsnChildren, Token![,]>,
}

impl Parse for BsnEntity {
    fn parse(input: ParseStream) -> Result<Self> {
        let mut inherits = Punctuated::new();
        let patch;
        if input.peek(Paren) {
            let content;
            parenthesized![content in input];

            let mut patch_tuple = Punctuated::new();

            loop {
                if content.is_empty() {
                    break;
                }

                if content.peek(Token![:]) {
                    content.parse::<Token![:]>()?;
                    inherits = content.parse_terminated(BsnInherit::parse, Token![,])?;
                    break;
                }

                let patch = content.parse::<BsnPatch>()?;
                patch_tuple.push_value(patch);
                if content.is_empty() {
                    break;
                }

                if content.peek(Token![:]) || (content.peek(Token![,]) && content.peek2(Token![:]))
                {
                    content.parse::<Token![,]>().ok();
                    content.parse::<Token![:]>()?;
                    inherits = content.parse_terminated(BsnInherit::parse, Token![,])?;
                    break;
                }

                let punct = content.parse()?;
                patch_tuple.push_punct(punct);
            }

            patch = BsnPatch::Tuple(patch_tuple);
        } else {
            patch = BsnPatch::parse(input)?;
        }

        let children = if input.peek(token::Bracket) {
            let content;
            bracketed![content in input];
            content.parse_terminated(BsnChildren::parse, Token![,])?
        } else {
            Punctuated::new()
        };

        Ok(Self {
            inherits,
            patch,
            children,
        })
    }
}

impl ToTokens for BsnEntity {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let cant_wait_for_bsn = syn::Path::from(Ident::new(
            "cant_wait_for_bsn",
            proc_macro2::Span::call_site(),
        ));
        let patch = &self.patch;
        let inherits = self.inherits.iter();
        let children = self.children.iter();
        quote! {
            #cant_wait_for_bsn::EntityPatch {
                inherit: (#(#inherits,)*),
                patch: #patch,
                children: (#(#children,)*),
            }
        }
        .to_tokens(tokens);
    }
}

pub enum BsnChildren {
    Entity(BsnEntity),
    Spread(Expr),
}

impl Parse for BsnChildren {
    fn parse(input: ParseStream) -> Result<Self> {
        if input.peek(Token![.]) && input.peek2(Token![.]) {
            // Parse as spread
            input.parse::<Token![.]>()?;
            input.parse::<Token![.]>()?;
            Ok(BsnChildren::Spread(input.parse::<Expr>()?))
        } else {
            Ok(BsnChildren::Entity(input.parse::<BsnEntity>()?))
        }
    }
}

impl ToTokens for BsnChildren {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let cant_wait_for_bsn = syn::Path::from(Ident::new(
            "cant_wait_for_bsn",
            proc_macro2::Span::call_site(),
        ));
        match self {
            BsnChildren::Entity(entity) => entity.to_tokens(tokens),
            BsnChildren::Spread(expr) => quote! {
                #cant_wait_for_bsn::SceneIter::new(#expr)
            }
            .to_tokens(tokens),
        }
    }
}

#[derive(Debug)]
pub enum BsnPatch {
    Patch(Path, Vec<(Member, BsnProp)>),
    Tuple(Punctuated<BsnPatch, Token![,]>),
    Expr(Expr),
}

impl Parse for BsnPatch {
    fn parse(input: ParseStream) -> Result<BsnPatch> {
        // TODO: Flatten tuples recursively?
        if input.peek(Paren) {
            // Tuple
            let content;
            parenthesized![content in input];
            let tuple = content.parse_terminated(BsnPatch::parse, Token![,])?;
            Ok(BsnPatch::Tuple(tuple))
        } else if input.peek(Brace) {
            // Expression
            let content;
            braced![content in input];
            let expr = content.parse::<Expr>()?;
            Ok(BsnPatch::Expr(expr))
        } else {
            // TODO: Maybe also support fallback-to-expression for maybe-structs that don't turn out to be parsable as struct
            // Another idea is to treat paths where last segment is lowercase (probably function call) as expr by default. (bit weird, but should be good a nice DX)
            let path = input.parse::<Path>()?;

            let fields = if input.peek(Paren) {
                // Tuple struct
                let content;
                parenthesized![content in input];
                content
                    .parse_terminated(BsnProp::parse, Token![,])?
                    .iter()
                    .enumerate()
                    .map(|(i, prop)| (Member::from(i), prop.clone())) // TODO: Avoid clone?
                    .collect()
            } else if input.peek(Brace) {
                // Struct (braced)
                let content;
                braced![content in input];
                content
                    .parse_terminated(
                        |input| {
                            let member: Member = input.parse()?;
                            let _colon_token: Token![:] = input.parse()?;
                            let prop: BsnProp = input.parse()?;
                            Ok((member, prop))
                        },
                        Token![,],
                    )?
                    .iter()
                    .cloned() // TODO: Avoid clone?
                    .collect()
            } else {
                Vec::new()
            };

            Ok(BsnPatch::Patch(path, fields))
        }
    }
}

impl ToTokens for BsnPatch {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let cant_wait_for_bsn = syn::Path::from(Ident::new(
            "cant_wait_for_bsn",
            proc_macro2::Span::call_site(),
        ));
        match self {
            BsnPatch::Patch(path, fields) => {
                let assignments = fields.iter().map(|(member, prop)| {
                    quote! {
                        props.#member = #prop;
                    }
                });
                quote! {
                    #path::patch(move |props| {
                        #(#assignments)*
                    })
                }
            }
            BsnPatch::Tuple(tuple) => quote! {
                (#tuple)
            },
            BsnPatch::Expr(expr) => quote! {
                #cant_wait_for_bsn::ConstructPatch::new_inferred(move |props| {
                    *props = #expr;
                })
            },
        }
        .to_tokens(tokens);
    }
}

#[derive(Debug, Clone)]
pub enum BsnProp {
    Value(Expr),
    Prop(Expr),
}

impl Parse for BsnProp {
    fn parse(input: ParseStream) -> Result<BsnProp> {
        let is_prop = input.parse::<Token![@]>().is_ok();
        let expr = input.parse::<Expr>()?;
        match is_prop {
            true => Ok(BsnProp::Prop(expr)),
            false => Ok(BsnProp::Value(expr)),
        }
    }
}

impl ToTokens for BsnProp {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let cant_wait_for_bsn = syn::Path::from(Ident::new(
            "cant_wait_for_bsn",
            proc_macro2::Span::call_site(),
        ));
        match self {
            BsnProp::Value(expr) => quote! {
                (#expr).into()
            },
            BsnProp::Prop(expr) => quote! {
                #cant_wait_for_bsn::ConstructProp::Prop((#expr).into())
            },
        }
        .to_tokens(tokens);
    }
}

#[derive(Debug, Clone)]
pub struct BsnInherit(Path, Punctuated<Expr, Token![,]>);

impl Parse for BsnInherit {
    fn parse(input: ParseStream) -> Result<BsnInherit> {
        let path = input.parse::<Path>()?;

        // Optional params
        let params = if input.peek(Paren) {
            let content;
            parenthesized![content in input];
            content.parse_terminated(Expr::parse, Token![,])?
        } else {
            Punctuated::new()
        };
        Ok(BsnInherit(path, params))
    }
}

impl ToTokens for BsnInherit {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let BsnInherit(path, params) = &self;
        quote! {
            (#path (#params))
        }
        .to_tokens(tokens);
    }
}
