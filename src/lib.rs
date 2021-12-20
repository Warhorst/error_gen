extern crate syn;

use proc_macro::TokenStream;

use syn::{AttributeArgs, ItemEnum, ItemStruct, parse, parse_macro_input};


mod struct_error;
mod enum_error;
mod parameters;
mod common;
mod impl_from;
mod impl_display;

/// Create fully qualified errors with the "error" attribute.
///
/// A fully qualified error means:
/// - std::error::Error is implemented
/// - std::fmt::Debug is implemented
/// - std::fmt::Display is implemented
/// Also, it's possible to generate implementations for std::convert::From for enum variants.
///
/// The attribute is applicable for struct definitions and enum definitions. If it's used anywhere else,
/// the attribute panics. The only exception are enum variants if the enum definition holds the error attribute.
///
/// # Usage on structs
/// Add the attribute to the struct definition like this
/// ``` text
/// #[error(message = "Something went wrong!", impl_from)]
/// struct MyError {
///     faulty_value: usize
/// }
/// ```
/// which will create an equivalent implementation to this
/// ``` text
/// use std::fmt::Formatter;
///
/// #[derive(Debug)]
/// struct MyError {
///     faulty_value: usize
/// }
///
/// impl std::error::Error for MyError {}
///
/// impl std::fmt::Display for MyError {
///     fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
///         write!(f, "Something went wrong!")
///     }
/// }
///
/// impl std::convert::From for MyError {
///     fn from(val: usize) {
///         MyError { faulty_value: val }
///     }
/// }
/// ```
///
/// Generics, lifetimes and any other attributes will be preserved.
///
/// As you can see, the 'message' parameter is used to generate the error message.
/// To create more useful error messages, it is possible to use fields. For example
/// ``` text
/// #[error(message = "The unexpected value '{faulty_value}' was returned!")]
/// struct MyError {
///     faulty_value: usize
/// }
///
/// assert_eq!(MyError { faulty_value: 43 }.to_string(), "The unexpected value '43' was returned!")
/// ```
///
/// The same is true for structs with unnamed parameters, which are used by index. For example
/// ``` text
/// #[error(message = "The service returned {1} and {0}.")]
/// struct MyError(usize, f32);
///
/// assert_eq!(MyError(42, 43.5).to_string(), "The service returned 43.5 and 42.")
/// ```
///
/// It is also possible to omit the 'message' parameter. This way, it is possible to implement
/// std::fmt::Display manually.
///
/// The 'impl_from' parameter can be used to generate implementations for std::convert::From.
/// It is applicable for non unit structs with exactly one field.
///
/// # Usage on enums
///
/// Add the attribute to the enum definition like this
/// ``` text
/// #[error(message = "Something went wrong")]
/// enum MyError {
///     Unknown,
///     #[error(message = "Parsing failed in line {line}")]
///     ParsingFailed { line: usize },
///     #[error(message = "Could not read file. Problem: {0}", impl_from)]
///     ReadFileFailed(std::io::Error)
/// }
/// ```
/// which will create an equivalent implementation to this
/// ``` text
/// use std::fmt::Formatter;
/// use std::io::Error;
///
/// #[derive(Debug)]
/// enum MyError {
///     Unknown,
///     ParsingFailed { line: usize },
///     ReadFileFailed(std::io::Error)
/// }
///
/// impl std::error::Error for MyError {}
///
/// impl std::fmt::Display for MyError {
///     fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
///         match self {
///             MyError::ParsingFailed { line } => write!(f, "Parsing failed in line {}", line),
///             MyError::ReadFileFailed(e) => write!(f, "Could not read file. Problem: {}", e),
///             _ => write!(f, "Something went wrong")
///         }
///     }
/// }
///
/// impl std::convert::From<std::io::Error> for MyError {
///     fn from(e: Error) -> Self {
///         MyError::ReadFileFailed(e)
///     }
/// }
/// ```
///
/// The enum and any of it's variants can have the 'message' parameter. The value in the enum attribute is used for
/// any variant without a custom message. It can only be omitted if every variant has a custom message.
/// Enum messages have the same templating features like struct messages.
///
/// The 'impl_from' parameter can be added either to enums or variants. If set to a variant, a std::convert::From for this
/// variant will be created. If it's added to the whole enum, error_gen tries to implement from for every variant. If at least
/// one variant has more than one field, this fails. Its also invalid to add 'impl_from' to a variant and the whole enum
/// and will create a panic, choose one.
#[proc_macro_attribute]
pub fn error(attributes: TokenStream, item: TokenStream) -> TokenStream {
    if let Ok(item_struct) = parse::<ItemStruct>(item.clone()) {
        return struct_error::implement(parse_macro_input!(attributes as AttributeArgs), item_struct)
    }

    if let Ok(item_enum) = parse::<ItemEnum>(item) {
        return enum_error::implement(parse_macro_input!(attributes as AttributeArgs), item_enum)
    }

    panic!("The error attribute is only allowed on structs, enums and enum variants.")
}