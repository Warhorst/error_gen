use proc_macro::TokenStream;

use quote::quote;
use syn::{AttributeArgs, ItemStruct};

use crate::impl_display::structs::StructDisplayImplementor;
use crate::impl_from::structs::StructFromImplementer;
use crate::parameters::Parameters;

/// Generate the implementations for a given struct to be a fully qualified and
/// usable error. This means
///
/// - std::error::Error is implemented
/// - std::fmt::Debug and Display are implemented
/// - std::convert::From is implemented (if possible) to allow the usage of the ?-operator
pub fn implement(attr_args: AttributeArgs, item_struct: ItemStruct) -> TokenStream {
    let parameters = Parameters::from_attribute_args(attr_args);

    let ident = &item_struct.ident;
    let generics = &item_struct.generics;
    let (impl_generics, type_generics, where_clause) = generics.split_for_impl();

    let display_implementation = StructDisplayImplementor::new(&item_struct, &parameters).implement();
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