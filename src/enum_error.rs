use proc_macro::TokenStream;

use quote::quote;
use syn::{Attribute, ItemEnum, Variant, AttributeArgs};
use crate::parameters::Parameters;
use crate::common::*;
use crate::impl_from::*;
use crate::impl_display::DisplayDataEnum;

const ERROR_ATTRIBUTE: &'static str = "error";
const MESSAGE: &'static str = "message";
const IMPL_FROM: &'static str = "impl_from";

pub fn implement(attr_args: AttributeArgs, mut item_enum: ItemEnum) -> TokenStream {
    let enum_parameters = Parameters::from_attribute_args(attr_args);

    let ident = &item_enum.ident;
    let generics = &item_enum.generics;
    let where_clause = &generics.where_clause;
    let variants = &mut item_enum.variants;


    let variants_with_parameters = variants
        .iter_mut()
        .flat_map(|var| to_variant_with_parameters(var).into_iter())
        .collect::<Vec<_>>();

    let mut from_data = FromImplData::new();
    let mut display_data = DisplayDataEnum::new_empty(&item_enum, enum_parameters.string_for_name(MESSAGE));

    for (variant, parameters) in &variants_with_parameters {
        if *parameters.bool_for_name(IMPL_FROM).get_or_insert(false) {
            from_data.add_data(&item_enum, &variant)
        }

        if let Some(m) = parameters.string_for_name(MESSAGE) {
            display_data.add_match_arm_data(m.clone(), variant);
        }
    }

    let display_implementation = display_data.to_display_implementation();
    let from_implementations = from_data.create_from_implementations();

    let r = quote! {
        #[derive(Debug)] #item_enum
        impl #generics std::error::Error for #ident #generics #where_clause {}

        #(#from_implementations)*

        #display_implementation
    };
    println!("{}", r);
    r.into()
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

pub fn attribute_is_error(attribute: &Attribute) -> bool {
    path_to_name(&attribute.path) == ERROR_ATTRIBUTE
}