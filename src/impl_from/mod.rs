use std::fmt::Formatter;

use syn::Ident;

use crate::impl_from::FailedItem::*;
use crate::impl_from::FromImplementationError::*;
use crate::parameters::IMPL_FROM;

pub mod structs;
pub mod enums;

#[derive(Debug)]
pub enum FailedItem {
    Struct(Ident),
    Enum(Ident, Ident),
}

/// Error that might occur when the generation of a std::convert::From implementation
/// fails for structs or enums.
#[derive(Debug)]
pub enum FromImplementationError {
    /// The struct or variant requires exactly one field for From to be implemented.
    NotExactlyOneField(FailedItem),
    /// The parameters::IMPL_FROM parameter was set on an enum and at least one variant.
    /// To keep the code clean, this is considered an error.
    ParameterOnEnumAndVariant(Ident, Ident),
}

impl std::error::Error for FromImplementationError {}

impl std::fmt::Display for FromImplementationError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            NotExactlyOneField(item) => match item {
                Struct(ident) => write!(f, "'std::convert::From' cannot be implemented for struct '{}', as it has not exactly one field.", ident),
                Enum(enum_ident, variant_ident) => write!(f, "'std::convert::From' cannot be implemented for enum variant '{}::{}', as it has not exactly one field.", enum_ident, variant_ident),
            }
            ParameterOnEnumAndVariant(enum_ident, variant_ident) => write!(f, "The '{}' parameter was set on enum '{}' and on its variant {}. Choose only one (enum or variants).", IMPL_FROM, enum_ident, variant_ident)
        }
    }
}