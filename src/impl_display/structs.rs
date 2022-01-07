use quote::quote;
use syn::__private::TokenStream2;
use syn::ItemStruct;

use crate::impl_display::write::WriteImplementor;
use crate::parameters::{MESSAGE, Parameters};

pub struct StructDisplayImplementor<'a> {
    item_struct: &'a ItemStruct,
    parameters: &'a Parameters,
}

impl<'a> StructDisplayImplementor<'a> {
    pub fn new(item_struct: &'a ItemStruct, parameters: &'a Parameters) -> Self {
        StructDisplayImplementor { item_struct, parameters }
    }

    pub fn implement(self) -> TokenStream2 {
        let write_implementation = match self.parameters.string_for_name(MESSAGE) {
            Some(m) => WriteImplementor::new().implement(m),
            None => return quote! {}
        };

        let ident = &self.item_struct.ident;
        let generics = &self.item_struct.generics;
        let (impl_generics, type_generics, where_clause) = generics.split_for_impl();
        quote! {
            impl #impl_generics std::fmt::Display for #ident #type_generics #where_clause {
                fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                    #write_implementation
                }
            }
        }
    }
}