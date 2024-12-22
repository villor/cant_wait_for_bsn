use cant_wait_for_bsn_parse::*;
use proc_macro2::{Ident, TokenStream};
use quote::{quote, ToTokens};
use syn::{
    parse2,
    punctuated::{Pair, Punctuated},
};

pub fn bsn(item: TokenStream) -> TokenStream {
    match parse2::<BsnEntity>(item) {
        Ok(bsn) => bsn.to_token_stream(),
        Err(e) => e.to_compile_error(), // TODO: Don't just bail here? Output as much as possible to keep RA happy.
    }
}

pub fn bsn_hot(item: TokenStream) -> TokenStream {
    let mut out = TokenStream::new();

    let bsn = match parse2::<BsnEntity>(item) {
        Ok(bsn) => bsn,
        Err(e) => return e.to_compile_error(),
    };

    let cant_wait_for_bsn = syn::Path::from(Ident::new(
        "cant_wait_for_bsn",
        proc_macro2::Span::call_site(),
    ));

    let scene = bsn.to_token_stream();
    quote! {{
        #cant_wait_for_bsn::HotReloadableBsnMacro {
            file: file!(),
            line: line!(),
            column: column!(),
            id: #cant_wait_for_bsn::BsnInvocationId::new(file!(), line!(), column!()),
            scene: #scene,
        }
    }}
    .to_tokens(&mut out);

    out
}

trait ToTokensInternal {
    fn to_tokens(&self, tokens: &mut TokenStream);

    fn to_token_stream(&self) -> TokenStream {
        let mut tokens = TokenStream::new();
        self.to_tokens(&mut tokens);
        tokens
    }
}

impl<T, P> ToTokensInternal for Punctuated<T, P>
where
    T: ToTokensInternal,
    P: ToTokens,
{
    fn to_tokens(&self, tokens: &mut TokenStream) {
        for pair in self.pairs() {
            match pair {
                Pair::Punctuated(a, b) => {
                    a.to_tokens(tokens);
                    b.to_tokens(tokens);
                }
                Pair::End(a) => a.to_tokens(tokens),
            }
        }
    }
}

impl ToTokensInternal for BsnEntity {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let cant_wait_for_bsn = syn::Path::from(Ident::new(
            "cant_wait_for_bsn",
            proc_macro2::Span::call_site(),
        ));
        let patch = &self.patch.to_token_stream();
        let inherits = self.inherits.iter().map(|i| i.to_token_stream());
        let children = self.children.iter().map(|i| i.to_token_stream());
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

impl ToTokensInternal for BsnChildren {
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

impl ToTokensInternal for BsnPatch {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let cant_wait_for_bsn = syn::Path::from(Ident::new(
            "cant_wait_for_bsn",
            proc_macro2::Span::call_site(),
        ));
        match self {
            BsnPatch::Patch(path, fields) => {
                let assignments = fields.iter().map(|(member, prop)| {
                    let member = member.to_token_stream();
                    let prop = prop.to_token_stream();
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
            BsnPatch::Tuple(tuple) => {
                let tuple = tuple.to_token_stream();
                quote! {
                    (#tuple)
                }
            }
            BsnPatch::Expr(expr) => quote! {
                #cant_wait_for_bsn::ConstructPatch::new_inferred(move |props| {
                    *props = #expr;
                })
            },
        }
        .to_tokens(tokens);
    }
}

impl ToTokensInternal for BsnProp {
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

impl ToTokensInternal for BsnInherit {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let BsnInherit(path, params) = &self;
        quote! {
            (#path (#params))
        }
        .to_tokens(tokens);
    }
}
