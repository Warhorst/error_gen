use std::collections::HashMap;
use std::fmt::Formatter;

use quote::{format_ident, quote};
use syn::{Field, FieldsNamed, FieldsUnnamed, Index, ItemEnum, ItemStruct, Variant};
use syn::__private::quote::__private::Ident;
use syn::__private::TokenStream2;
use syn::Fields::*;

use DisplayImplementationError::*;

use crate::enum_error::VariantWithParams;
use crate::parameters::{MESSAGE, Parameters};

pub struct StructDisplayImplementor<'a> {
    item_struct: &'a ItemStruct,
    parameters: &'a Parameters
}

impl<'a> StructDisplayImplementor<'a> {
    pub fn new(item_struct: &'a ItemStruct, parameters: &'a Parameters) -> Self {
        StructDisplayImplementor { item_struct, parameters }
    }

    /// Create a std::fmt::Display implementation for the given struct.
    ///
    /// If no message was provided, Display should be implemented manually and an empty
    /// token stream is returned.
    pub fn implement(self) -> TokenStream2 {
        let mut message = match self.parameters.string_for_name(MESSAGE) {
            Some(m) => m,
            None => return quote! {}
        };

        match &self.item_struct.fields {
            Named(fields) => {
                let params = self.create_named_write_parameters(&message, &fields);
                self.create_implementation_with_write_parameters(&message, params)
            },
            Unnamed(fields) => {
                let params = self.create_positional_write_parameters(&mut message, &fields);
                self.create_implementation_with_write_parameters(&message, params)
            },
            Unit => self.create_implementation_with_write_parameters(&message, vec![])
        }
    }

    pub fn create_named_write_parameters(&self, message: &String, fields: &FieldsNamed) -> Vec<TokenStream2> {
        get_used_identifiers_in_string(message, fields)
            .into_iter()
            .map(|ident| quote! {, #ident = self.#ident})
            .collect()
    }

    fn create_positional_write_parameters(&self, message: &mut String, fields: &FieldsUnnamed) -> Vec<TokenStream2> {
        let mut parameters = vec![];
        let mut ignored_fields = 0;

        for i in 0..fields.unnamed.len() {
            let string = format!("{{{}}}", i);

            if message.contains(&string) {
                *message = message.replace(&string, &format!("{{{}}}", i - ignored_fields));
                let index = Index::from(i);
                parameters.push(quote! {, self.#index});
            } else {
                ignored_fields += 1
            }
        }
        parameters
    }

    fn create_implementation_with_write_parameters(&self, message: &String, parameters: Vec<TokenStream2>) -> TokenStream2 {
        let ident = &self.item_struct.ident;
        let generics = &self.item_struct.generics;
        let (impl_generics, type_generics, where_clause) = generics.split_for_impl();
        quote! {
            impl #impl_generics std::fmt::Display for #ident #type_generics #where_clause {
                fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                    write!(f, #message #(#parameters)*)
                }
            }
        }
    }
}

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
            .map(|(v, m)| self.create_match_arm(v, m))
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

    /// Create a match arm for the Display implementation with the given variant and it's message.
    fn create_match_arm(&self, variant: &Variant, message: String) -> TokenStream2 {
        match &variant.fields {
            Named(fields) => MatchArmDataNamed::new(message, &self.item_enum.ident, &variant.ident, fields).to_match_arm(),
            Unnamed(fields) => MatchArmDataUnnamed::new(message, &self.item_enum.ident, &variant.ident, fields).to_match_arm(),
            Unit => MatchArmDataUnit::new(message, &self.item_enum.ident, &variant.ident).to_match_arm()
        }
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
                        #(#match_arms)*
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
            Some(m) => quote! {_ => write!(f, #m)},
            None => quote! {}
        }
    }
}

#[derive(Debug)]
pub enum DisplayImplementationError {
    MissingMessages(Ident),
    UnnecessaryDefaultMessage(Ident)
}

impl std::error::Error for DisplayImplementationError {}

impl std::fmt::Display for DisplayImplementationError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            MissingMessages(ident) => write!(f, "Not all variants of enum '{}' have a Display message. Consider adding a default message at the enum item.", ident),
            UnnecessaryDefaultMessage(ident) => write!(f, "All variants for enum '{}' have a Display message, but a default was provided anyways. Please remove.", ident)
        }
    }
}

struct MatchArmDataNamed<'a> {
    message: String,
    enum_ident: &'a Ident,
    variant_ident: &'a Ident,
    fields: &'a FieldsNamed,
}

impl<'a> MatchArmDataNamed<'a> {
    pub fn new(message: String, enum_ident: &'a Ident, variant_ident: &'a Ident, fields: &'a FieldsNamed) -> Self {
        MatchArmDataNamed { message, enum_ident, variant_ident, fields }
    }

    pub fn to_match_arm(self) -> TokenStream2 {
        let enum_ident = &self.enum_ident;
        let variant_ident = &self.variant_ident;
        let message = &self.message;
        let match_patterns = self.create_match_patterns();
        let write_parameters = self.create_write_parameters();

        quote! {
            #enum_ident::#variant_ident{#(#match_patterns)*} => write!(f, #message #(#write_parameters)*),
        }
    }

    fn create_match_patterns(&self) -> Vec<TokenStream2> {
        let identifiers = get_used_identifiers_in_string(&self.message, self.fields);
        let field_amount = self.fields.named.len();

        let mut tokens = identifiers.into_iter()
            .enumerate()
            .map(|(i, ident)| match i {
                0 => quote! {#ident},
                _ => quote! {, #ident}
            })
            .collect::<Vec<_>>();

        match tokens.len() {
            0 => vec![quote! {..}],
            len if len == field_amount => tokens,
            _ => {
                tokens.extend(vec![quote! {, ..}]);
                tokens
            }
        }
    }

    fn create_write_parameters(&self) -> Vec<TokenStream2> {
        get_used_identifiers_in_string(&self.message, self.fields)
            .into_iter()
            .map(|ident| quote! {, #ident = #ident})
            .collect()
    }
}

struct MatchArmDataUnnamed<'a> {
    message: String,
    enum_ident: &'a Ident,
    variant_ident: &'a Ident,
    fields: &'a FieldsUnnamed,
}

impl<'a> MatchArmDataUnnamed<'a> {
    pub fn new(message: String, enum_ident: &'a Ident, variant_ident: &'a Ident, fields: &'a FieldsUnnamed) -> Self {
        MatchArmDataUnnamed { message, enum_ident, variant_ident, fields }
    }

    pub fn to_match_arm(mut self) -> TokenStream2 {
        let parameter_map = self.get_unnamed_parameters_in_message();
        let mut match_params = vec![];
        let mut write_params = vec![];

        for i in 0..self.fields.unnamed.len() {
            let parameter = parameter_map.get(&i).unwrap();
            if let Some(l) = parameter.letter {
                self.message = self.message.replace(&format!("{{{}}}", i), &format!("{{{}}}", l))
            }
            match_params.push(parameter.pattern.clone());
            write_params.push(parameter.write_param.clone());
        }

        let enum_ident = &self.enum_ident;
        let variant_ident = &self.variant_ident;
        let message = self.message;
        quote! {
            #enum_ident::#variant_ident(#(#match_params)*) => write!(f, #message #(#write_params)*),
        }
    }

    fn get_unnamed_parameters_in_message(&self) -> HashMap<usize, UnnamedEnumParamter> {
        let field_amount = self.fields.unnamed.len();
        (0..field_amount).into_iter()
            .map(|i| match self.message.contains(&format!("{{{}}}", i)) {
                true => {
                    let letter = char::from_u32((i as u32) + 97).unwrap();
                    let ident = format_ident!("{}", letter);
                    let pattern = if i < field_amount - 1 { quote! {#ident,} } else { quote! {#ident} };
                    let write_param = quote! {,#ident = #ident};
                    (i, UnnamedEnumParamter::new(Some(letter), pattern, write_param))
                }
                false => {
                    let pattern = if i < field_amount - 1 { quote! {_,} } else { quote! {_} };
                    let write_param = quote! {};
                    (i, UnnamedEnumParamter::new(None, pattern, write_param))
                }
            })
            .collect()
    }
}

struct UnnamedEnumParamter {
    letter: Option<char>,
    pattern: TokenStream2,
    write_param: TokenStream2,
}

impl UnnamedEnumParamter {
    pub fn new(letter: Option<char>, pattern: TokenStream2, write_param: TokenStream2) -> Self {
        UnnamedEnumParamter { letter, pattern, write_param }
    }
}

struct MatchArmDataUnit<'a> {
    message: String,
    enum_ident: &'a Ident,
    variant_ident: &'a Ident,
}

impl<'a> MatchArmDataUnit<'a> {
    pub fn new(message: String, enum_ident: &'a Ident, variant_ident: &'a Ident) -> Self {
        MatchArmDataUnit { message, enum_ident, variant_ident }
    }

    pub fn to_match_arm(self) -> TokenStream2 {
        let enum_ident = &self.enum_ident;
        let variant_ident = &self.variant_ident;
        let message = &self.message;
        quote! {
            #enum_ident::#variant_ident => write!(f, #message),
        }
    }
}

pub fn get_used_identifiers_in_string<'a>(display_string: &String, fields: &'a FieldsNamed) -> Vec<&'a Ident> {
    fields.named.iter()
        .map(|field| {
            let ident = get_field_ident(field);
            if display_string.contains(&format!("{{{}}}", ident.to_string())) {
                return Some(ident);
            }
            None
        })
        .flat_map(Option::into_iter)
        .collect()
}

fn get_field_ident(field: &Field) -> &Ident {
    field.ident.as_ref().unwrap()
}