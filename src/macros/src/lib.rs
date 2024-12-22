use proc_macro::TokenStream;

mod bsn;
mod derive_construct;

#[proc_macro]
pub fn bsn(item: TokenStream) -> TokenStream {
    bsn::bsn(item.into()).into()
}

#[proc_macro]
pub fn bsn_hot(item: TokenStream) -> TokenStream {
    bsn::bsn_hot(item.into()).into()
}

#[proc_macro_derive(Construct, attributes(construct))]
pub fn derive_construct(item: TokenStream) -> TokenStream {
    derive_construct::derive_construct(item.into()).into()
}
