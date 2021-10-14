use proc_macro::TokenStream;

use quote::quote;
use syn::{AttributeArgs, ItemStruct};
use crate::parameters::Parameters;
use crate::impl_display::DisplayDataStruct;

const MESSAGE: &'static str = "message";

pub fn implement(attr_args: AttributeArgs, item_struct: ItemStruct) -> TokenStream {
    let parameters = Parameters::from_attribute_args(attr_args);
    let message_opt = parameters.string_for_name(MESSAGE);

    let ident = &item_struct.ident;
    let generics = &item_struct.generics;
    let where_clause = &item_struct.generics.where_clause;

    let display_implementation = DisplayDataStruct::new(&item_struct, message_opt).to_display_implementation();

    (quote! {
        #[derive(Debug)] #item_struct
        impl #generics std::error::Error for #ident #generics #where_clause {}
        #display_implementation
    }).into()
}