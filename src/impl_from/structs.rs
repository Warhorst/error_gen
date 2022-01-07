use quote::quote;
use syn::{FieldsNamed, FieldsUnnamed, ItemStruct};
use syn::__private::TokenStream2;
use syn::Fields::*;

use crate::impl_from::FromImplementationError;
use crate::impl_from::FromImplementationError::StructNotExactlyOneField;
use crate::parameters::{IMPL_FROM, Parameters};

pub struct StructFromImplementer<'a> {
    item_struct: &'a ItemStruct,
    struct_parameters: &'a Parameters,
}

impl<'a> StructFromImplementer<'a> {
    pub fn new(item_struct: &'a ItemStruct, struct_parameters: &'a Parameters) -> Self {
        StructFromImplementer { item_struct, struct_parameters }
    }

    /// Create the std::convert::From implementation for a struct.
    ///
    /// If the struct should not implement From, return an empty token stream.
    /// Returns Result::Err if the struct is an unit or has not exactly one field.
    pub fn implement(self) -> Result<TokenStream2, FromImplementationError> {
        if self.struct_parameters.bool_for_name(IMPL_FROM) == false {
            return Ok(quote! {});
        }

        match &self.item_struct.fields {
            Named(ref fields) => self.implement_for_named(fields),
            Unnamed(ref fields) => self.implement_for_unnamed(fields),
            Unit => Err(StructNotExactlyOneField(self.item_struct.ident.clone()))
        }
    }

    fn implement_for_named(self, fields: &FieldsNamed) -> Result<TokenStream2, FromImplementationError> {
        if fields.named.len() != 1 {
            return Err(StructNotExactlyOneField(self.item_struct.ident.clone()));
        }

        let struct_ident = &self.item_struct.ident;
        let generics = &self.item_struct.generics;
        let (impl_generics, type_generics, where_clause) = generics.split_for_impl();
        let field = fields.named.first().unwrap();
        let ty = &field.ty;
        let field_ident = field.ident.as_ref().unwrap();

        Ok(quote! {
            impl #impl_generics std::convert::From<#ty> for #struct_ident #type_generics #where_clause {
                fn from(val: #ty) -> Self {
                    #struct_ident{ #field_ident : val }
                }
            }
        })
    }

    fn implement_for_unnamed(self, fields: &FieldsUnnamed) -> Result<TokenStream2, FromImplementationError> {
        if fields.unnamed.len() != 1 {
            return Err(StructNotExactlyOneField(self.item_struct.ident.clone()));
        }

        let struct_ident = &self.item_struct.ident;
        let generics = &self.item_struct.generics;
        let (impl_generics, type_generics, where_clause) = generics.split_for_impl();
        let field = fields.unnamed.first().unwrap();
        let ty = &field.ty;

        Ok(quote! {
            impl #impl_generics std::convert::From<#ty> for #struct_ident #type_generics #where_clause {
                fn from(val: #ty) -> Self {
                    #struct_ident(val)
                }
            }
        })
    }
}