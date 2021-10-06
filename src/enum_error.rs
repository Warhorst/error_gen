use proc_macro::TokenStream;

use quote::quote;
use syn::{Attribute, ItemEnum, Variant, AttributeArgs, FieldsNamed, FieldsUnnamed};
use syn::__private::TokenStream2;
use syn::Fields::*;
use crate::parameters::Parameters;
use crate::common::attribute_is_error;

pub fn implement(_attr_args: AttributeArgs, mut item_enum: ItemEnum) -> TokenStream {
    let ident = &item_enum.ident;
    let generics = &item_enum.generics;
    let where_clause = &generics.where_clause;
    let variants = &mut item_enum.variants;

    let mut from_implementations = vec![];

    let variants_with_parameters = variants
        .iter_mut()
        .flat_map(|var| to_variant_with_parameters(var).into_iter())
        .collect::<Vec<_>>();

    for (variant, parameters) in variants_with_parameters {
        if parameters.value_for_name("derive_from").map_or(false, |lit| lit.bool_value()) {
            from_implementations.push(create_from_implementation(&item_enum, variant))
        }
    }

    let gen = quote! {
        #[derive(Debug)] #item_enum
        impl #generics std::error::Error for #ident #generics #where_clause {}

        #(#from_implementations)*
    };
    println!("{}", gen);
    gen.into()
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

fn create_from_implementation(item_enum: &ItemEnum, variant: Variant) -> TokenStream2 {
    match &variant.fields {
        Named(fields) => create_from_for_fields_named(item_enum, &variant, fields),
        Unnamed(fields) => create_from_for_fields_unnamed(item_enum, &variant, fields),
        Unit => panic!("Cannot implement From trait for Unit variants.")
    }
}

fn create_from_for_fields_named(item_enum: &ItemEnum, variant: &Variant, fields: &FieldsNamed) -> TokenStream2 {
    if fields.named.len() != 1 {
        panic!("From trait can only be implemented for variants with one field.")
    }

    let enum_ident = &item_enum.ident;
    let generics = &item_enum.generics;
    let where_clause = &generics.where_clause;
    let variant_ident = &variant.ident;

    let field = fields.named.first().unwrap();
    let ty = &field.ty;
    let field_ident = field.ident.as_ref().unwrap();

    quote! {
        impl #generics std::convert::From<#ty> for #enum_ident #generics #where_clause {
            fn from(val: #ty) -> Self {
                #enum_ident::#variant_ident{ #field_ident : val }
            }
        }
    }
}

fn create_from_for_fields_unnamed(item_enum: &ItemEnum, variant: &Variant, fields: &FieldsUnnamed) -> TokenStream2 {
    if fields.unnamed.len() != 1 {
        panic!("From trait can only be implemented for variants with one field.")
    }

    let enum_ident = &item_enum.ident;
    let generics = &item_enum.generics;
    let where_clause = &generics.where_clause;
    let variant_ident = &variant.ident;

    let field = fields.unnamed.first().unwrap();
    let ty = &field.ty;

    quote! {
        impl #generics std::convert::From<#ty> for #enum_ident #generics #where_clause {
            fn from(val: #ty) -> Self {
                #enum_ident::#variant_ident(val)
            }
        }
    }
}