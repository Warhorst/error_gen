use proc_macro::TokenStream;

use quote::quote;
use syn::{AttributeArgs, Fields, Fields::*, Generics, Ident, Index, ItemStruct};
use syn::__private::TokenStream2;
use crate::common::*;
use crate::parameters::Parameters;

const MESSAGE: &'static str = "message";

pub fn implement(attr_args: AttributeArgs, item_struct: ItemStruct) -> TokenStream {
    let parameters = Parameters::from_attribute_args(attr_args);
    let message_opt = parameters.string_for_name(MESSAGE);

    let ident = &item_struct.ident;
    let fields = &item_struct.fields;
    let generics = &item_struct.generics;
    let where_clause = &item_struct.generics.where_clause;

    let display_implementation = create_display_implementation(message_opt, ident, fields, generics);

    (quote! {
        #[derive(Debug)] #item_struct
        impl #generics std::error::Error for #ident #generics #where_clause {}
        #display_implementation
    }).into()
}

fn create_display_implementation(display_string_opt: Option<String>, ident: &Ident, fields: &Fields, generics: &Generics) -> TokenStream2 {
    let mut display_string = match display_string_opt {
        Some(string) => string,
        None => return quote! {}
    };

    let write_parameters = match fields {
        Named(_) => create_named_write_parameters(&display_string, fields),
        Unnamed(_) => create_positional_write_parameters(&mut display_string, fields),
        Unit => vec![]
    };

    create_implementation_with_write_parameters(display_string, ident, generics, write_parameters)
}

fn create_positional_write_parameters(display_string: &mut String, fields: &Fields) -> Vec<TokenStream2> {
    let mut parameters = vec![];
    let mut ignored_fields = 0;

    for i in 0..fields.len() {
        let string = format!("{{{}}}", i);

        if display_string.contains(&string) {
            *display_string = display_string.replace(&string, &format!("{{{}}}", i - ignored_fields));
            let index = Index::from(i);
            parameters.push(quote! {, self.#index});
        } else {
            ignored_fields += 1
        }
    }
    parameters
}

fn create_implementation_with_write_parameters(display_string: String, ident: &Ident, generics: &Generics, parameters: Vec<TokenStream2>) -> TokenStream2 {
    let where_clause = &generics.where_clause;
    quote! {
        impl #generics std::fmt::Display for #ident #generics #where_clause {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, #display_string #(#parameters)*)
            }
        }
    }
}