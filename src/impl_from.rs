use quote::quote;
use syn::{FieldsNamed, FieldsUnnamed, ItemEnum, Variant};
use syn::__private::TokenStream2;
use syn::Fields::*;

use crate::parameters::IMPL_FROM;

/// Create an implementation of the std::convert::From trait, based on the type of the enum variant.
///
/// From can only be implemented if the variant is not Unit or the variant only holds one field.
pub struct FromImplData<'a> {
    item_enum: &'a ItemEnum,
    implement_global: bool,
    variants: Vec<&'a Variant>,
    usage_errors: Vec<String>
}

impl<'a> FromImplData<'a> {
    pub fn new(item_enum: &'a ItemEnum, implement_global: bool) -> Self {
        FromImplData { item_enum, implement_global, variants: vec![], usage_errors: vec![] }
    }

    /// Add a variant for code generation.
    ///
    /// The 'From' implementation is only generated if the global or variant setting is set to 'true'.
    /// To keep the code clean, adding '{crate::parameters::IMPL_FROM}' to the enum AND the variant is considered an error and will cause a panic.
    pub fn add_variant(&mut self, variant: &'a Variant, impl_from_for_variant: bool) {
        if self.implement_global && impl_from_for_variant {
            let error = format!("Implementation of std::convert::From is enabled for all variants. Please remove parameter '{}' from variant '{}'.", IMPL_FROM, variant.ident);
            self.usage_errors.push(error);
            return;
        }

        if self.implement_global || impl_from_for_variant {
            self.variants.push(variant)
        }
    }

    pub fn to_from_implementations(self) -> Vec<TokenStream2> {
        if !self.usage_errors.is_empty() {
            panic!("{}", self.usage_errors.into_iter().collect::<String>())
        }

        self.variants
            .iter()
            .map(|variant| match &variant.fields {
                Named(fields) => self.create_for_fields_named(variant, fields),
                Unnamed(fields) => self.create_for_fields_unnamed(variant, fields),
                Unit => panic!("Cannot implement From trait for Unit variants.")
            })
            .collect()
    }

    fn create_for_fields_named(&self, variant: &Variant, fields: &FieldsNamed) -> TokenStream2 {
        if fields.named.len() != 1 {
            panic!("From trait can only be implemented for variants with one field.")
        }

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

    fn create_for_fields_unnamed(&self, variant: &Variant, fields: &FieldsUnnamed) -> TokenStream2 {
        if fields.unnamed.len() != 1 {
            panic!("From trait can only be implemented for variants with one field.")
        }

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