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

pub fn bsn(item: TokenStream) -> TokenStream {
    match parse2::<BsnEntity>(item) {
        Ok(bsn) => bsn.into_token_stream(),
        Err(e) => e.to_compile_error(), // TODO: Don't just bail here? Output as much as possible to keep RA happy.
    }
}

pub struct BsnEntity {
    pub patch: BsnPatch,
    pub children: Punctuated<BsnEntity, Token![,]>,
}

impl Parse for BsnEntity {
    fn parse(input: ParseStream) -> Result<Self> {
        let patch = BsnPatch::parse(input)?;

        let children = if input.peek(token::Bracket) {
            let content;
            bracketed![content in input];
            content.parse_terminated(BsnEntity::parse, Token![,])?
        } else {
            Punctuated::new()
        };

        Ok(Self { patch, children })
    }
}

impl ToTokens for BsnEntity {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let cant_wait_for_bsn = syn::Path::from(Ident::new(
            "cant_wait_for_bsn",
            proc_macro2::Span::call_site(),
        ));
        let patch = &self.patch;
        let children = self.children.iter();
        quote! {
            #cant_wait_for_bsn::EntityPatch {
                // inherit: (),
                patch: #patch,
                children: (#(#children,)*),
            }
        }
        .to_tokens(tokens);
    }
}

#[derive(Debug)]
pub enum BsnPatch {
    Patch(Path, Vec<(Member, BsnProp)>),
    Tuple(Punctuated<BsnPatch, Token![,]>),
    // Expr(Expr),
}

impl Parse for BsnPatch {
    fn parse(input: ParseStream) -> Result<BsnPatch> {
        // TODO: DIY parsing to get better autocomplete?

        // TODO: Flatten tuples recursively?
        if input.peek(Paren) {
            let content;
            parenthesized![content in input];
            let tuple = content.parse_terminated(BsnPatch::parse, Token![,])?;
            Ok(BsnPatch::Tuple(tuple))
        } else {
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
        // TODO: Support other expressions?
    }
}

impl ToTokens for BsnPatch {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
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
                .to_tokens(tokens)
            }
            BsnPatch::Tuple(tuple) => quote! {
                (#tuple)
            }
            .to_tokens(tokens),
        }
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
