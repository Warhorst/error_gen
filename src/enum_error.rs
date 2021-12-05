use proc_macro::TokenStream;

use quote::quote;
use syn::{Attribute, AttributeArgs, ItemEnum, Variant};

use crate::common::*;
use crate::impl_display::DisplayDataEnum;
use crate::impl_from::*;
use crate::parameters::{ERROR_ATTRIBUTE, IMPL_FROM, MESSAGE, Parameters};

pub fn implement(attr_args: AttributeArgs, mut item_enum: ItemEnum) -> TokenStream {
    let global_parameters = Parameters::from_attribute_args(attr_args);
    let variants = &mut item_enum.variants;

    let variants_with_parameters = variants
        .iter_mut()
        .map(|var| to_variant_with_parameters(var))
        .collect::<Vec<_>>();

    let mut from_data = FromImplData::new(&item_enum, global_parameters.bool_for_name(IMPL_FROM));
    let mut display_data = DisplayDataEnum::new(&item_enum, global_parameters.string_for_name(MESSAGE));

    for (variant, parameters_opt) in &variants_with_parameters {
        from_data.add_variant(&variant, parameters_opt.as_ref().map(|p| p.bool_for_name(IMPL_FROM)).unwrap_or(false));
        display_data.add_variant(variant, parameters_opt.as_ref().and_then(|p| p.string_for_name(MESSAGE)));
    }

    let ident = &item_enum.ident;
    let generics = &item_enum.generics;
    let (impl_generics, type_generics, where_clause) = generics.split_for_impl();
    let display_implementation = display_data.to_display_implementation();
    let from_implementations = from_data.create_from_implementations();

    (quote! {
        #[derive(Debug)] #item_enum
        impl #impl_generics std::error::Error for #ident #type_generics #where_clause {}

        #(#from_implementations)*

        #display_implementation
    }).into()
}

fn to_variant_with_parameters(variant: &mut Variant) -> (Variant, Option<Parameters>) {
    match extract_error_attribute(&mut variant.attrs) {
        Some(attr) => (variant.clone(), Some(Parameters::from_attribute(attr))),
        None => (variant.clone(), None)
    }
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

pub fn attribute_is_error(attribute: &Attribute) -> bool {
    path_to_name(&attribute.path) == ERROR_ATTRIBUTE
}