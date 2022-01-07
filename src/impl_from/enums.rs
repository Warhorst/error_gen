use quote::quote;
use syn::{FieldsNamed, FieldsUnnamed, ItemEnum, Variant};
use syn::__private::TokenStream2;
use syn::Fields::*;

use crate::enum_error::VariantWithParams;
use crate::impl_from::FromImplementationError;
use crate::impl_from::FromImplementationError::{EnumNotExactlyOneField, ParameterOnEnumAndVariant};
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
    pub fn implement(self) -> Result<TokenStream2, FromImplementationError> {
        let global_implement = self.enum_parameters.bool_for_name(IMPL_FROM);
        let impl_from_variants = self.get_impl_from_variants();

        self.validate_impl_from_settings(global_implement, &impl_from_variants)?;

        let implementations = match global_implement {
            true => self.implement_for_variants(self.item_enum.variants.iter()),
            false => self.implement_for_variants(impl_from_variants)
        };

        Ok(quote! {#(#implementations)*})
    }

    /// Return all variants with set IMPL_FROM parameter
    fn get_impl_from_variants(&self) -> Vec<&Variant> {
        self.variants_with_parameters
            .iter()
            .filter_map(|(v, p_opt)| match p_opt {
                Some(p) => Some((v, p)),
                _ => None
            })
            .filter_map(|(v, p)| match p.bool_for_name(IMPL_FROM) {
                true => Some(*v),
                false => None
            })
            .collect()
    }

    /// Check the global IMPL_FROM setting and all variants with impl_from set.
    ///
    /// An error occurs if
    /// the global IMPL_FROM was set and at least one variant has impl_from set
    /// OR
    /// variants with IMPL_FROM set have not exactly one field.
    ///
    /// If the global IMPL_FROM is
    ///     true, all variants are checked
    ///     false, only variants with IMPL_FROM set are checked
    fn validate_impl_from_settings(&self, global_impl_from: bool, impl_from_variants: &Vec<&Variant>) -> Result<(), FromImplementationError> {
        if global_impl_from && impl_from_variants.len() > 0 {
            return Err(ParameterOnEnumAndVariant(self.item_enum.ident.clone()));
        }

        let variant_idents_with_not_one_field = match global_impl_from {
            true => self.item_enum.variants.iter()
                .filter(|v| self.variant_num_fields(v) != 1)
                .map(|v| v.ident.clone())
                .collect::<Vec<_>>(),
            false => impl_from_variants.iter()
                .filter(|v| self.variant_num_fields(v) != 1)
                .map(|v| v.ident.clone())
                .collect::<Vec<_>>()
        };

        match variant_idents_with_not_one_field.len() {
            0 => Ok(()),
            _ => Err(EnumNotExactlyOneField(self.item_enum.ident.clone(), variant_idents_with_not_one_field))
        }
    }

    fn variant_num_fields(&self, variant: &Variant) -> usize {
        match &variant.fields {
            Named(f) => f.named.len(),
            Unnamed(f) => f.unnamed.len(),
            Unit => 0
        }
    }

    fn implement_for_variants<'b, I>(&self, variants: I) -> Vec<TokenStream2>
        where I: IntoIterator<Item=&'b Variant> {
        variants.into_iter()
            .map(|v| self.implement_for_variant(v))
            .collect()
    }

    fn implement_for_variant(&self, variant: &Variant) -> TokenStream2 {
        match &variant.fields {
            Named(ref fields) => self.implement_for_named(variant, fields),
            Unnamed(ref fields) => self.implement_for_unnamed(variant, fields),
            Unit => unreachable!()
        }
    }

    fn implement_for_named(&self, variant: &Variant, fields: &FieldsNamed) -> TokenStream2 {
        let enum_ident = &self.item_enum.ident;
        let generics = &self.item_enum.generics;
        let (impl_generics, type_generics, where_clause) = generics.split_for_impl();
        let variant_ident = &variant.ident;
        let field = fields.named.first().unwrap();
        let ty = &field.ty;
        let field_ident = field.ident.as_ref().unwrap();

        quote! {
            impl #impl_generics std::convert::From<#ty> for #enum_ident #type_generics #where_clause {
                fn from(val: #ty) -> Self {
                    #enum_ident::#variant_ident{ #field_ident : val }
                }
            }
        }
    }

    fn implement_for_unnamed(&self, variant: &Variant, fields: &FieldsUnnamed) -> TokenStream2 {
        let enum_ident = &self.item_enum.ident;
        let generics = &self.item_enum.generics;
        let (impl_generics, type_generics, where_clause) = generics.split_for_impl();
        let variant_ident = &variant.ident;
        let field = fields.unnamed.first().unwrap();
        let ty = &field.ty;

        quote! {
            impl #impl_generics std::convert::From<#ty> for #enum_ident #type_generics #where_clause {
                fn from(val: #ty) -> Self {
                    #enum_ident::#variant_ident(val)
                }
            }
        }
    }
}