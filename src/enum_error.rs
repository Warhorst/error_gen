use proc_macro::TokenStream;

use quote::quote;
use syn::{Attribute, AttributeArgs, ItemEnum, Variant};

use crate::common::*;
use crate::impl_display::DisplayDataEnum;
use crate::impl_from::*;
use crate::parameters::{ERROR_ATTRIBUTE, IMPL_FROM, MESSAGE, Parameters};

pub fn implement(attr_args: AttributeArgs, mut item_enum: ItemEnum) -> TokenStream {
    let global_parameters = Parameters::from_attribute_args(attr_args);
    let mut display_data = DisplayDataEnum::new(&item_enum, global_parameters.string_for_name(MESSAGE));
    let mut from_data = FromImplData::new(&item_enum, global_parameters.bool_for_name(IMPL_FROM));

    item_enum.variants
        .iter()
        .map(to_variant_with_parameters)
        .for_each(|(variant, parameters_opt)| {
            display_data.add_variant(variant, &parameters_opt);
            from_data.add_variant(variant, &parameters_opt);
        });

    let display_implementation = display_data.to_display_implementation();
    let from_implementations = from_data.to_from_implementations();

    remove_variant_attributes(&mut item_enum);

    let ident = &item_enum.ident;
    let generics = &item_enum.generics;
    let (impl_generics, type_generics, where_clause) = generics.split_for_impl();

    (quote! {
        #[derive(Debug)] #item_enum
        impl #impl_generics std::error::Error for #ident #type_generics #where_clause {}

        #(#from_implementations)*

        #display_implementation
    }).into()
}

fn to_variant_with_parameters(variant: &Variant) -> (&Variant, Option<Parameters>) {
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

// TODO: it might be unnecessary to remove the attributes. Compiler error: "expected non-macro attribute, found attribute macro `error`".
//  Currently I always get an error when adding any kind of attribute to an enum variant. But if the attribute recognizes anything else than the
//  error attribute, it could just return the TokenStream.
//  ^
//  |
//  Update: Attributes on non items seem to be only allowed as helper attributes in custom derives
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

pub fn attribute_is_error(attribute: &Attribute) -> bool {
    path_to_name(&attribute.path) == ERROR_ATTRIBUTE
}