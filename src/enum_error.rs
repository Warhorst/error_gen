use proc_macro::{TokenStream, Span};
use quote::quote;
use syn::{parse_macro_input, ItemEnum, AttributeArgs, Variant, MetaList, Path, Attribute, NestedMeta, Meta};
use syn::spanned::Spanned;
use syn::punctuated::Punctuated;
use syn::token::Token;
use std::convert::TryFrom;
use syn::Lit;
use syn::Lit::*;

pub fn implement(_attributes: TokenStream, item: TokenStream) -> TokenStream {
    let mut item_enum = parse_macro_input!(item as ItemEnum);

    let ident = &item_enum.ident;
    let generics = &item_enum.generics;
    let where_clause = &generics.where_clause;
    let variants = &mut item_enum.variants;

    for variant in variants {
        parse_variant_attributes(&mut variant.attrs);
    }

    // foo(variants);


    let gen = quote! {
        #[derive(Debug)] #item_enum
        impl #generics std::error::Error for #ident #generics #where_clause {}
    };
    println!("{}", gen);
    gen.into()
}

fn foo(variants: &Punctuated<Variant, Token![,]>) {
    for variant in variants {
        for att in &variant.attrs {
            println!("{}", att.path.get_ident().expect("foo"));
            let meta = att.parse_meta().unwrap();

            // Use meta to get the parameters
            match meta {
                syn::Meta::List(_) => println!("list"),
                syn::Meta::Path(_) => println!("path"),
                syn::Meta::NameValue(_) => println!("name_value")
            }
        }
    }
}

fn parse_variant_attributes(attributes: &mut Vec<Attribute>) -> Option<Parameters> {
    let att_index = attributes
        .iter()
        .cloned()
        .enumerate()
        .find(|(_, att)| path_is_error_attribute(&att.path))?;
    attributes.remove(att_index.0);

    let meta = att_index.1.parse_meta().unwrap();

    if let syn::Meta::List(list) = meta {
        let parameters = Parameters::from_nested_metas(&list.nested);
    } else {
        panic!("Expected meta list for attribute e_error")
    }


    None
}

fn path_is_error_attribute(path: &Path) -> bool {
    path
        .get_ident()
        .map_or(false, |ident| &ident.to_string() == "e_error")
}

#[derive(Default, Debug, Eq, PartialEq)]
struct Parameters {
    description: Option<String>,
    derive_from: Option<bool>,
}

impl Parameters {
    const DESCRIPTION_IDENTIFIER: &'static str = "description";
    const DERIVE_FROM_IDENTIFIER: &'static str = "derive_from";

    /// Parse the nested meta to parameters. If none of the expected parameters were provided, nothing is returned.
    fn from_nested_metas(nested_metas: &Punctuated<NestedMeta, Token![,]>) -> Option<Self> {
        let mut result = Self::default();

        let name_values = nested_metas
            .iter()
            .filter_map(|nested| match nested {
                syn::NestedMeta::Meta(meta) => Some(meta),
                syn::NestedMeta::Lit(_) => panic!("Unexpected literal in meta list")
            })
            .filter_map(Self::meta_to_name_value);

        name_values.into_iter().for_each(|(name, value)| {
            if name == Self::DESCRIPTION_IDENTIFIER.to_string() {
                result.description = Some(value.get_string())
            }

            if name == Self::DERIVE_FROM_IDENTIFIER.to_string() {
                result.derive_from = Some(value.get_bool())
            }
        });

        match result == Self::default() {
            true => None,
            false => Some(result)
        }
    }

    /// Parse meta to name value tuples. The value of a NameValue is the
    /// value of the corresponding literal. The value of a path is always true.
    /// Meta lists are ignored.
    fn meta_to_name_value(meta: &Meta) -> Option<(String, LitValue)> {
        match meta {
            syn::Meta::NameValue(name_value) => {
                let name = path_to_name(&name_value.path);
                let value = LitValue::from(&name_value.lit);
                Some((name, value))
            }
            syn::Meta::Path(path) => {
                let name = path_to_name(path);
                let value = LitValue::Boolean(true);
                Some((name, value))
            }
            _ => None
        }
    }
}

fn path_to_name(path: &Path) -> String {
    path.get_ident().map(|ident| ident.to_string()).expect("The given path was not an identifier.")
}

enum LitValue {
    String(String),
    Boolean(bool),
}

impl LitValue {
    fn get_bool(&self) -> bool {
        if let LitValue::Boolean(b) = self {
            return *b
        }

        panic!("Expected boolean")
    }

    fn get_string(&self) -> String {
        if let LitValue::String(s) = self {
            return s.clone()
        }

        panic!("Expected string")
    }
}

impl From<&Lit> for LitValue {
    fn from(lit: &syn::Lit) -> Self {
        match lit {
            Str(lit_str) => LitValue::String(lit_str.value()),
            Bool(lit_bool) => LitValue::Boolean(lit_bool.value),
            _ => panic!("Unexpected literal value")
        }
    }
}