use proc_macro2::{Ident, TokenStream};
use quote::{quote, ToTokens};
use syn::{
    braced, bracketed, parenthesized,
    parse::{Parse, ParseStream},
    parse2,
    punctuated::Punctuated,
    token::{self, Brace, Paren},
    Expr, FieldValue, Member, Path, Result, Token,
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
    Patch(Path, Vec<(Member, Expr)>),
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
                    .parse_terminated(Expr::parse, Token![,])?
                    .iter()
                    .enumerate()
                    .map(|(i, expr)| (Member::from(i), expr.clone())) // TODO: Avoid clone?
                    .collect()
            } else if input.peek(Brace) {
                // Struct (braced)
                let content;
                braced![content in input];
                content
                    .parse_terminated(FieldValue::parse, Token![,])?
                    .iter()
                    .map(|field_value| (field_value.member.clone(), field_value.expr.clone())) // TODO: Avoid clone?
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
                let assignments = fields.iter().map(|(member, expr)| {
                    quote! {
                        props.#member = (#expr).into();
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
