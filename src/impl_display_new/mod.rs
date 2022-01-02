use std::fmt::Formatter;

use syn::Ident;

use crate::impl_display_new::DisplayImplementationError::*;

pub mod struct_implementor;
pub mod enum_implementor;
pub mod write_implementor;
pub mod variant_struct_implementor;

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