use proc_macro::TokenStream;

use quote::quote;
use syn::{Attribute, ItemEnum, Variant, AttributeArgs, FieldsNamed, FieldsUnnamed};
use syn::__private::TokenStream2;
use syn::Fields::*;
use crate::parameters::{Parameters, LitValue};
use crate::common::*;
use syn::__private::quote::__private::Ident;
use crate::impl_from::*;

pub fn implement(attr_args: AttributeArgs, mut item_enum: ItemEnum) -> TokenStream {
    let enum_parameters = Parameters::from_attribute_args(attr_args);

    let ident = &item_enum.ident;
    let generics = &item_enum.generics;
    let where_clause = &generics.where_clause;
    let variants = &mut item_enum.variants;

    let mut from_implementations = vec![];
    let mut display_match_arms = vec![];

    let variants_with_parameters = variants
        .iter_mut()
        .flat_map(|var| to_variant_with_parameters(var).into_iter())
        .collect::<Vec<_>>();

    for (variant, parameters) in variants_with_parameters {
        if parameters.value_for_name("derive_from").map_or(false, LitValue::bool_value) {
            from_implementations.push(create_from_implementation(&item_enum, &variant))
        }

        if let Some(val) = parameters.value_for_name("description") {
            display_match_arms.push(create_display_match_arm(val.string_value(), &item_enum.ident, &variant))
        }
    }

    let display_implementation = match display_match_arms.len() == item_enum.variants.len() {
        true => create_display_implementation(&item_enum, display_match_arms, None),
        false => match enum_parameters.value_for_name("description") {
            Some(val) => create_display_implementation(&item_enum, display_match_arms, Some(val.string_value())),
            None => panic!("Not all enum variants have a display message. Provide a default message at the enum definition.")
        }
    };

    (quote! {
        #[derive(Debug)] #item_enum
        impl #generics std::error::Error for #ident #generics #where_clause {}

        #(#from_implementations)*

        #display_implementation
    }).into()
}

fn to_variant_with_parameters(variant: &mut Variant) -> Option<(Variant, Parameters)> {
    let error_attribute = extract_error_attribute(&mut variant.attrs)?;
    let parameters = Parameters::from_attribute(error_attribute);

    if parameters.is_empty() { return None; }

    Some((variant.clone(), parameters))
}

fn extract_error_attribute(attributes: &mut Vec<Attribute>) -> Option<Attribute> {
    let index = attributes
        .iter()
        .enumerate()
        .find_map(|(i, att)| match attribute_is_error(att) {
            true => Some(i),
            false => None
        })?;
    Some(attributes.remove(index))
}

fn create_display_match_arm(description: String, ident: &Ident, variant: &Variant) -> TokenStream2 {
    match &variant.fields {
        Unit => create_unit_match_arm(description, ident, variant),
        Unnamed(fields) => create_unnamed_match_arm(description, ident, variant, fields),
        Named(fields) => create_named_match_arm(description, ident, variant, fields)
    }
}

fn create_unit_match_arm(description: String, ident: &Ident, variant: &Variant) -> TokenStream2 {
    let variant_ident = &variant.ident;
    quote! {
        #ident::#variant_ident => write!(f, #description),
    }
}

fn create_unnamed_match_arm(description: String, ident: &Ident, variant: &Variant, fields: &FieldsUnnamed) -> TokenStream2 {
    create_unnamed_variant_match_arm(description, fields, ident, variant)
}

fn create_named_match_arm(description: String, ident: &Ident, variant: &Variant, fields: &FieldsNamed) -> TokenStream2 {
    let variant_ident = &variant.ident;
    let write_parameters = create_named_write_parameters_enum(&description, &fields.named);
    let match_arm_parameters = create_named_enum_variant_match_arm_parameters(&description, fields);

    quote! {
        #ident::#variant_ident{#(#match_arm_parameters)*} => write!(f, #description #(#write_parameters)*),
    }
}

fn create_display_implementation(item_enum: &ItemEnum, match_arms: Vec<TokenStream2>, default_message: Option<String>) -> TokenStream2 {
    let ident = &item_enum.ident;
    let generics = &item_enum.generics;
    let where_clause = &generics.where_clause;
    let default = match default_message {
        Some(m) => quote! {_ => write!(f, #m)},
        None => quote! {}
    };
    quote! {
        impl #generics std::fmt::Display for #ident #generics #where_clause {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                match self {
                    #(#match_arms)*
                    #default
                }
            }
        }
    }
}