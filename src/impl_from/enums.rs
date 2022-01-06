use quote::quote;
use syn::{FieldsNamed, FieldsUnnamed, ItemEnum, Variant};
use syn::__private::TokenStream2;
use syn::Fields::*;

use crate::enum_error::VariantWithParams;
use crate::impl_from::FailedItem::Enum;
use crate::impl_from::FromImplementationError;
use crate::impl_from::FromImplementationError::{NotExactlyOneField, ParameterOnEnumAndVariant};
use crate::parameters::{IMPL_FROM, Parameters};

pub struct EnumFromImplementer<'a> {
    item_enum: &'a ItemEnum,
    enum_parameters: &'a Parameters,
    variants_with_parameters: &'a Vec<VariantWithParams<'a>>,
}

impl<'a> EnumFromImplementer<'a> {
    pub fn new(item_enum: &'a ItemEnum, enum_parameters: &'a Parameters, variants_with_parameters: &'a Vec<VariantWithParams<'a>>) -> Self {
        EnumFromImplementer { item_enum, enum_parameters, variants_with_parameters }
    }

    /// Creates std::convert::From implementations for every enum variant where
    /// From should be implemented (based on parameters).
    ///
    /// The implementations for every variant are merged into a single token stream. If anything fails,
    /// a Vec of errors is returned (even if some implementations could be created).
    ///
    /// An error might occur if
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
            Unit => Err(NotExactlyOneField(Enum(self.item_enum.ident.clone(), variant.ident.clone())))
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