mod bsn;

#[proc_macro]
pub fn bsn(item: proc_macro::TokenStream) -> proc_macro::TokenStream {
    bsn::bsn(item.into()).into()
}
