use proc_macro::TokenStream;

use quote::quote;
use syn::{Attribute, ItemEnum, parse_macro_input, Path, Variant, AttributeArgs};
use syn::__private::TokenStream2;
use crate::parameters::Parameters;
use crate::common::attribute_is_error;

pub fn implement(_attr_args: AttributeArgs, mut item_enum: ItemEnum) -> TokenStream {
    let ident = &item_enum.ident;
    let generics = &item_enum.generics;
    let where_clause = &generics.where_clause;
    let variants = &mut item_enum.variants;

    for variant in variants {
        let error_attribute = match extract_error_attribute(&mut variant.attrs) {
            Some(att) => att,
            None => continue
        };

        let parameters = Parameters::from_attribute(error_attribute);
        if parameters.is_empty() { continue; }

        if parameters.value_for_name("derive_from").map_or(false, |lit| lit.bool_value()) {
            // create from implementation
        }

        if let Some(val) =  parameters.value_for_name("description") {
            // create match arm for display
        }
    }

    let gen = quote! {
        #[derive(Debug)] #item_enum
        impl #generics std::error::Error for #ident #generics #where_clause {}
    };
    println!("{}", gen);
    gen.into()
}

fn extract_error_attribute(attributes: &mut Vec<Attribute>) -> Option<Attribute> {
    let index = attributes
        .iter()
        .enumerate()
        .find_map(|(i, att)| match attribute_is_error(att) {
            true => Some((i)),
            false => None
        })?;
    Some(attributes.remove(index))
}

fn path_is_error_attribute(path: &Path) -> bool {
    path
        .get_ident()
        .map_or(false, |ident| &ident.to_string() == "error")
}