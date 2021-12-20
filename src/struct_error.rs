use proc_macro::TokenStream;

use quote::quote;
use syn::{AttributeArgs, ItemStruct};

use crate::impl_display::DisplayDataStruct;
use crate::parameters::{MESSAGE, Parameters};
use crate::impl_from::StructFromImplementer;

pub fn implement(attr_args: AttributeArgs, item_struct: ItemStruct) -> TokenStream {
    let parameters = Parameters::from_attribute_args(attr_args);
    let message_opt = parameters.string_for_name(MESSAGE);

    let ident = &item_struct.ident;
    let generics = &item_struct.generics;
    let (impl_generics, type_generics, where_clause) = generics.split_for_impl();

    let display_implementation = DisplayDataStruct::new(&item_struct, message_opt).to_display_implementation();
    let from_implementation = match StructFromImplementer::new(&item_struct, &parameters).implement() {
        Ok(implementation) => implementation,
        Err(e) => panic!("Error while generating From for struct: {}", e)
    };

    (quote! {
        #[derive(Debug)] #item_struct
        impl #impl_generics std::error::Error for #ident #type_generics #where_clause {}
        #display_implementation
        #from_implementation
    }).into()
}