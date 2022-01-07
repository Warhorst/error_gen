use std::fmt::Formatter;

use syn::Ident;

use crate::impl_from::FromImplementationError::*;
use crate::parameters::IMPL_FROM;

pub mod structs;
pub mod enums;

/// Error that might occur when the generation of a std::convert::From implementation
/// fails for structs or enums.
#[derive(Debug)]
pub enum FromImplementationError {
    /// A struct requires exactly one field for From to be implemented.
    StructNotExactlyOneField(Ident),
    /// An enum variant requires exactly one field for From to be implemented.
    /// Every failed variant is listed here.
    EnumNotExactlyOneField(Ident, Vec<Ident>),
    /// The parameters::IMPL_FROM parameter was set on an enum and at least one variant.
    /// To keep the code clean, this is considered an error.
    ParameterOnEnumAndVariant(Ident),
}

impl std::error::Error for FromImplementationError {}

impl std::fmt::Display for FromImplementationError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            StructNotExactlyOneField(ident) => write!(f, "'std::convert::From' cannot be implemented for struct '{}', as it has not exactly one field.", ident),
            EnumNotExactlyOneField(enum_ident, idents) => {
                let idents_string: String = idents.iter()
                    .enumerate()
                    .map(|(i, ident)| if i < idents.len() - 1 { format!("{},", ident) } else { format!("{}", ident) })
                    .collect();
                write!(f, "'std::convert::From' cannot be implemented for enum '{}'. The following variants don't have exactly one field: {}", enum_ident, idents_string)
            }
            ParameterOnEnumAndVariant(ident) => write!(f, "The '{}' parameter was set on enum '{}' and at least one of its variants. Choose only one (enum or variants).", IMPL_FROM, ident)
        }
    }
}