use proc_macro::TokenStream;
use syn::{parse, ItemStruct, ItemEnum, parse_macro_input, AttributeArgs};

extern crate syn;

mod struct_error;
mod enum_error;
mod parameters;
mod common;
mod impl_from;
mod impl_display;

#[proc_macro_attribute]
pub fn error(attributes: TokenStream, item: TokenStream) -> TokenStream {
    if let Ok(item_struct) = parse::<ItemStruct>(item.clone()) {
        return struct_error::implement(parse_macro_input!(attributes as AttributeArgs), item_struct)
    }

    if let Ok(item_enum) = parse::<ItemEnum>(item) {
        return enum_error::implement(parse_macro_input!(attributes as AttributeArgs), item_enum)
    }

    panic!("The error attribute is only allowed on structs, enums and enum variants.")
}