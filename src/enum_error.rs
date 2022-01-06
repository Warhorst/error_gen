use quote::quote;
use syn::{Attribute, AttributeArgs, ItemEnum, Variant};
use syn::__private::TokenStream2;

use crate::common::*;
use crate::impl_display::enums::EnumDisplayImplementor;
use crate::impl_from::enums::EnumFromImplementer;
use crate::parameters::Parameters;

pub type VariantWithParams<'a> = (&'a Variant, Option<Parameters>);

/// Generate the implementations for a given enum to be a fully qualified and
/// usable error. This means
///
/// - std::error::Error is implemented
/// - std::fmt::Debug and Display are implemented
/// - std::convert::From is implemented (if possible) to allow the usage of the ?-operator
pub fn implement(attr_args: AttributeArgs, mut item_enum: ItemEnum) -> TokenStream2 {
    let enum_parameters = Parameters::from_attribute_args(attr_args);

    let variants_with_parameters = item_enum.variants
        .iter()
        .map(to_variant_with_parameters)
        .collect::<Vec<_>>();

    let display_implementation = match EnumDisplayImplementor::new(&item_enum, &enum_parameters, &variants_with_parameters).implement() {
        Ok(implementation) => implementation,
        Err(e) => panic!("An error occurred while implementing std::fmt::Display for enum: {}", e)
    };
    let from_implementations = match EnumFromImplementer::new(&item_enum, &enum_parameters, &variants_with_parameters).implement() {
        Ok(implementations) => implementations,
        Err(errors) => panic!("Some errors occurred while implementing std::convert::From for enum: {}", errors.into_iter().map(|e| format!("{},", e.to_string())).collect::<String>())
    };

    remove_variant_attributes(&mut item_enum);

    let ident = &item_enum.ident;
    let generics = &item_enum.generics;
    let (impl_generics, type_generics, where_clause) = generics.split_for_impl();

    quote! {
        #[derive(Debug)] #item_enum
        impl #impl_generics std::error::Error for #ident #type_generics #where_clause {}

        #from_implementations

        #display_implementation
    }
}

fn to_variant_with_parameters(variant: &Variant) -> VariantWithParams {
    match get_error_attribute(&variant.attrs) {
        Some(attr) => (variant, Some(Parameters::from_attribute(attr))),
        None => (variant, None)
    }
}

fn get_error_attribute(attributes: &Vec<Attribute>) -> Option<&Attribute> {
    let index = attributes
        .iter()
        .enumerate()
        .find_map(|(i, att)| match attribute_is_error(att) {
            true => Some(i),
            false => None
        })?;
    attributes.get(index)
}

//  Attributes on non items seem to be only allowed as helper attributes in custom derives
//  (https://doc.rust-lang.org/reference/procedural-macros.html#derive-macro-helper-attributes). proc_macro_aatributes on the other hand are only allowed
//  on items (https://doc.rust-lang.org/reference/items.html) and need to be removed manually.
fn remove_variant_attributes(item_enum: &mut ItemEnum) {
    item_enum.variants
        .iter_mut()
        .for_each(remove_error_attribute_from_variant)
}

/// Search the index of the error attribute in the given variants attributes.
/// If the index could be found, remove the entry from the variants attributes.
fn remove_error_attribute_from_variant(variant: &mut Variant) {
    let index_opt = variant.attrs
        .iter()
        .enumerate()
        .find_map(|(i, att)| match attribute_is_error(att) {
            true => Some(i),
            false => None
        });

    if let Some(i) = index_opt {
        variant.attrs.remove(i);
    }
}