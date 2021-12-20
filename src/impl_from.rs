use std::fmt::Formatter;

use quote::quote;
use syn::{FieldsNamed, FieldsUnnamed, ItemEnum, ItemStruct, Variant};
use syn::__private::quote::__private::Ident;
use syn::__private::TokenStream2;
use syn::Fields::*;

use FailedItem::*;
use FromImplementationError::*;

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
            Unit => Err(ParameterOnUnit(Struct(self.item_struct.ident.clone())))
        }
    }

    fn implement_for_named(self, fields: &FieldsNamed) -> Result<TokenStream2, FromImplementationError> {
        if fields.named.len() != 1 {
            return Err(NotExactlyOneField(Struct(self.item_struct.ident.clone())));
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
            return Err(NotExactlyOneField(Struct(self.item_struct.ident.clone())));
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

pub struct EnumFromImplementer<'a> {
    item_enum: &'a ItemEnum,
    enum_parameters: &'a Parameters,
    variants_with_parameters: &'a Vec<(&'a Variant, Option<Parameters>)>,
}

impl<'a> EnumFromImplementer<'a> {
    pub fn new(item_enum: &'a ItemEnum, enum_parameters: &'a Parameters, variants_with_parameters: &'a Vec<(&'a Variant, Option<Parameters>)>) -> Self {
        EnumFromImplementer { item_enum, enum_parameters, variants_with_parameters }
    }

    /// Creates std::convert::From implementations for every enum variant where
    /// From should be implemented (based on parameters).
    ///
    /// The implementations for every variant are merged into a single token stream. If anything fails,
    /// a Vec of errors is returned (even if some implementations could be created).
    ///
    /// An error might occur if
    ///     a variant is a unit and should implement From
    ///     a variant does not have exactly one field
    ///     the enum and one variant are both marked with the parameter 'impl_from'
    pub fn implement(self) -> Result<TokenStream2, Vec<FromImplementationError>> {
        let global_implement = self.enum_parameters.bool_for_name(IMPL_FROM);

        let (implementations, errors) = self.variants_with_parameters
            .into_iter()
            .fold((vec![], vec![]), |(mut implementations, mut errors), (variant, parameters_opt)| {
                let impl_for_variant = parameters_opt.as_ref().map(|p| p.bool_for_name(IMPL_FROM)).unwrap_or(false);

                match (impl_for_variant, global_implement) {
                    (false, false) => implementations.push(quote! {}),
                    (true, true) => errors.push(ParameterOnEnumAndVariant(self.item_enum.ident.clone(), variant.ident.clone())),
                    _ => match self.implement_for_variant(variant) {
                        Ok(implementation) => implementations.push(implementation),
                        Err(e) => errors.push(e)
                    }
                }
                (implementations, errors)
            });

        match errors.is_empty() {
            true => Ok(quote! {#(#implementations)*}),
            false => Err(errors)
        }
    }

    fn implement_for_variant(&self, variant: &Variant) -> Result<TokenStream2, FromImplementationError> {
        match &variant.fields {
            Named(ref fields) => self.implement_for_named(variant, fields),
            Unnamed(ref fields) => self.implement_for_unnamed(variant, fields),
            Unit => Err(ParameterOnUnit(Enum(self.item_enum.ident.clone(), variant.ident.clone())))
        }
    }

    fn implement_for_named(&self, variant: &Variant, fields: &FieldsNamed) -> Result<TokenStream2, FromImplementationError> {
        if fields.named.len() != 1 {
            return Err(NotExactlyOneField(Enum(self.item_enum.ident.clone(), variant.ident.clone())));
        }

        let enum_ident = &self.item_enum.ident;
        let generics = &self.item_enum.generics;
        let (impl_generics, type_generics, where_clause) = generics.split_for_impl();
        let variant_ident = &variant.ident;
        let field = fields.named.first().unwrap();
        let ty = &field.ty;
        let field_ident = field.ident.as_ref().unwrap();

        Ok(quote! {
            impl #impl_generics std::convert::From<#ty> for #enum_ident #type_generics #where_clause {
                fn from(val: #ty) -> Self {
                    #enum_ident::#variant_ident{ #field_ident : val }
                }
            }
        })
    }

    fn implement_for_unnamed(&self, variant: &Variant, fields: &FieldsUnnamed) -> Result<TokenStream2, FromImplementationError> {
        if fields.unnamed.len() != 1 {
            return Err(NotExactlyOneField(Enum(self.item_enum.ident.clone(), variant.ident.clone())));
        }

        let enum_ident = &self.item_enum.ident;
        let generics = &self.item_enum.generics;
        let (impl_generics, type_generics, where_clause) = generics.split_for_impl();
        let variant_ident = &variant.ident;
        let field = fields.unnamed.first().unwrap();
        let ty = &field.ty;

        Ok(quote! {
            impl #impl_generics std::convert::From<#ty> for #enum_ident #type_generics #where_clause {
                fn from(val: #ty) -> Self {
                    #enum_ident::#variant_ident(val)
                }
            }
        })
    }
}

#[derive(Debug)]
pub enum FailedItem {
    Struct(Ident),
    Enum(Ident, Ident),
}

#[derive(Debug)]
pub enum FromImplementationError {
    ParameterOnUnit(FailedItem),
    NotExactlyOneField(FailedItem),
    ParameterOnEnumAndVariant(Ident, Ident),
}

impl std::error::Error for FromImplementationError {}

impl std::fmt::Display for FromImplementationError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            ParameterOnUnit(item) => match item {
                Struct(ident) => write!(f, "'std::convert::From' can not be implemented for unit struct '{}'", ident),
                Enum(enum_ident, variant_ident) => write!(f, "'std::convert::From' can not be implemented for enum variant '{}::{}'", enum_ident, variant_ident)
            }
            NotExactlyOneField(item) => match item {
                Struct(ident) => write!(f, "'std::convert::From' cannot be implemented for struct '{}', as it has not exactly one field.", ident),
                Enum(enum_ident, variant_ident) => write!(f, "'std::convert::From' cannot be implemented for enum variant '{}::{}', as it has not exactly one field.", enum_ident, variant_ident),
            }
            ParameterOnEnumAndVariant(enum_ident, variant_ident) => write!(f, "The '{}' parameter was set on enum '{}' and on its variant {}. Choose only one (enum or variants).", IMPL_FROM, enum_ident, variant_ident)
        }
    }
}