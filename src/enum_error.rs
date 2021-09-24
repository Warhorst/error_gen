use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, ItemEnum, AttributeArgs};
use syn::spanned::Spanned;

pub fn implement(_attributes: TokenStream, item: TokenStream) -> TokenStream {
    let item_enum = parse_macro_input!(item as ItemEnum);

    let variants = &item_enum.variants;

    let variant = variants.first().unwrap();
    let attrs = &variant.attrs;

    let att = attrs.first().unwrap();
    let meta = att.parse_meta().unwrap();
    /// works

    let gen = quote! {
        #item_enum
    };
    println!("{}", gen);
    gen.into()
}