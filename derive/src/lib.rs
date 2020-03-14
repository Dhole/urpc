use proc_macro::TokenStream;

#[proc_macro_attribute]
pub fn urpc_request(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}
