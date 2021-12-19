use proc_macro::TokenStream;

use quote::quote;
use syn::{AttributeArgs, ItemStruct};

use crate::impl_display::DisplayDataStruct;
use crate::parameters::{MESSAGE, Parameters, IMPL_FROM};
use crate::impl_from::StructFromImplData;

pub fn implement(attr_args: AttributeArgs, item_struct: ItemStruct) -> TokenStream {
    let parameters = Parameters::from_attribute_args(attr_args);
    let message_opt = parameters.string_for_name(MESSAGE);
    let impl_from = parameters.bool_for_name(IMPL_FROM);

    let ident = &item_struct.ident;
    let generics = &item_struct.generics;
    let (impl_generics, type_generics, where_clause) = generics.split_for_impl();

    let display_implementation = DisplayDataStruct::new(&item_struct, message_opt).to_display_implementation();
    let from_implementation = match impl_from {
        true => StructFromImplData::new(&item_struct).to_from_implementation(),
        false => quote! {}
    };

    (quote! {
        #[derive(Debug)] #item_struct
        impl #impl_generics std::error::Error for #ident #type_generics #where_clause {}
        #display_implementation
        #from_implementation
    }).into()
}