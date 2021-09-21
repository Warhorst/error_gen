use proc_macro::TokenStream;

pub fn implement(_attributes: TokenStream, item: TokenStream) -> TokenStream {
    item
}