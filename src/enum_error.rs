use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, ItemEnum, AttributeArgs, Variant};
use syn::spanned::Spanned;
use syn::punctuated::Punctuated;
use syn::token::Token;

pub fn implement(_attributes: TokenStream, item: TokenStream) -> TokenStream {
    let item_enum = parse_macro_input!(item as ItemEnum);

    let ident = &item_enum.ident;
    let generics = &item_enum.generics;
    let where_clause = &generics.where_clause;
    let variants = &item_enum.variants;
    //foo(variants);


    let gen = quote! {
        #[derive(Debug)] #item_enum
        impl #generics std::error::Error for #ident #generics #where_clause {}
    };
    println!("{}", gen);
    gen.into()
}

fn foo(variants: &Punctuated<Variant, Token![,]>) {
    let variant = variants.first().unwrap();
    let attrs = &variant.attrs;

    let att = attrs.first().unwrap();
    let meta = att.parse_meta().unwrap();
}