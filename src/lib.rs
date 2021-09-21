use proc_macro::TokenStream;

mod struct_error;
mod enum_error;

#[proc_macro_attribute]
pub fn error(attributes: TokenStream, item: TokenStream) -> TokenStream {
    struct_error::implement(attributes, item)
}

#[proc_macro_attribute]
pub fn e_error(attributes: TokenStream, item: TokenStream) -> TokenStream{
    enum_error::implement(attributes, item)
}