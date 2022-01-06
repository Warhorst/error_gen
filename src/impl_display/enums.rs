use quote::quote;
use syn::{ItemEnum, Variant};
use syn::__private::TokenStream2;

use crate::enum_error::VariantWithParams;
use crate::impl_display::DisplayImplementationError;
use crate::impl_display::DisplayImplementationError::*;
use crate::impl_display::match_arm::MatchArmImplementor;
use crate::parameters::{MESSAGE, Parameters};

pub struct EnumDisplayImplementor<'a> {
    item_enum: &'a ItemEnum,
    enum_parameters: &'a Parameters,
    variants_with_parameters: &'a Vec<VariantWithParams<'a>>
}

impl<'a> EnumDisplayImplementor<'a> {
    pub fn new(item_enum: &'a ItemEnum, enum_parameters: &'a Parameters, variants_with_parameters: &'a Vec<VariantWithParams<'a>>) -> Self {
        EnumDisplayImplementor { item_enum, enum_parameters, variants_with_parameters }
    }

    /// Create the std::fmt::Display implementation for the given enum and its variants.
    ///
    /// If neither the variants nor the enum itself provides a message, Display will not be implemented.
    ///
    /// This might fail if
    ///  not every variant has a message set and no default was set
    ///  OR every variant has a message and a default was set (this is an error to keep the code clean from useless parameters).
    pub fn implement(self) -> Result<TokenStream2, DisplayImplementationError> {
        let variants_with_message = self.get_variants_with_message();

        if self.display_should_not_be_implemented(&variants_with_message) {
            return Ok(quote! {})
        }

        self.check_set_messages_are_valid(&variants_with_message)?;

        let match_arms = variants_with_message
            .into_iter()
            .map(|(v, m)| MatchArmImplementor::new(&self.item_enum.ident, &m).implement_for(v))
            .collect::<Vec<_>>();

        Ok(self.create_implementation(match_arms))
    }

    /// Return a Vec of all variants witch a set Display message.
    fn get_variants_with_message(&self) -> Vec<(&Variant, String)> {
        self.variants_with_parameters
            .iter()
            .filter_map(|(v, p_opt)| match p_opt {
                Some(p) => Some((v, p)),
                _ => None
            })
            .filter_map(|(v, p)| match p.string_for_name(MESSAGE) {
                Some(m) => Some((*v, m)),
                _ => None
            })
            .collect()
    }

    /// If
    ///  our enum does not have a Display message in it's parameters
    ///  AND none of our variants has a Display message set
    /// Display should not be implemented
    fn display_should_not_be_implemented(&self, variants_with_message: &Vec<(&Variant, String)>) -> bool {
        !self.enum_parameters.has_parameter(MESSAGE) && variants_with_message.is_empty()
    }

    /// Check the Display messages on all variants and the default one. It's an error if
    ///  not every variant has a message and no default message was set
    ///  OR
    ///  all variants have a message, but a default message was provided anyways.
    fn check_set_messages_are_valid(&self, variants_with_message: &Vec<(&Variant, String)>) -> Result<(), DisplayImplementationError> {
        let num_variants = self.item_enum.variants.len();
        let num_set_messages = variants_with_message.len();
        let default_message_set = self.enum_parameters.has_parameter(MESSAGE);

        if !default_message_set && num_variants != num_set_messages {
            return Err(MissingMessages(self.item_enum.ident.clone()))
        }

        if default_message_set && num_variants == num_set_messages {
            return Err(UnnecessaryDefaultMessage(self.item_enum.ident.clone()))
        }

        Ok(())
    }

    fn create_implementation(&self, match_arms: Vec<TokenStream2>) -> TokenStream2 {
        let ident = &self.item_enum.ident;
        let generics = &self.item_enum.generics;
        let (impl_generics, type_generics, where_clause) = generics.split_for_impl();
        let default_match_arm = self.create_default_match_arm();

        quote! {
            impl #impl_generics std::fmt::Display for #ident #type_generics #where_clause {
                fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                    match self {
                        #(#match_arms,)*
                        #default_match_arm
                    }
                }
            }
        }
    }

    /// Create the default match arm for the Display implementation, which is necessary
    /// if not all variants have a message set.
    ///
    /// Note: EnumDisplayImplementor::check_set_messages_are_valid verifies if
    /// some messages are missing and a default is set, so it's not done here again.
    fn create_default_match_arm(&self) -> TokenStream2 {
        match self.enum_parameters.string_for_name(MESSAGE) {
            Some(m) => MatchArmImplementor::new(&self.item_enum.ident, &m).implement_default(),
            None => quote! {}
        }
    }
}