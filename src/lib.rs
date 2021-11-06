use proc_macro::TokenStream;
use syn::{parse, ItemStruct, ItemEnum, parse_macro_input, AttributeArgs};

extern crate syn;

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
/// ```
/// #[error(message = "Something went wrong!")]
/// struct MyError {
///     faulty_value: usize
/// }
/// ```
/// which will create an equivalent implementation to this
/// ```
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
/// ```
///
/// Generics, lifetimes and any other attributes will be preserved.
///
/// As you can see, the 'message' parameter is used to generate the error message.
/// To create more useful error messages, it is possible to use fields. For example
/// ```
/// #[error(message = "The unexpected value '{faulty_value}' was returned!")]
/// struct MyError {
///     faulty_value: usize
/// }
///
/// assert_eq!(MyError { faulty_value: 43 }.to_string(), "The unexpected value '43' was returned!")
/// ```
///
/// The same is true for structs with unnamed parameters, which are used by index. For example
/// ```
/// #[error(message = "The service returned {1} and {0}.")]
/// struct MyError(usize, f32);
///
/// assert_eq!(MyError(42, 43.5).to_string(), "The service returned 43.5 and 42.")
/// ```
///
/// It is also possible to omit the 'message' parameter. This way, it is possible to implement
/// std::fmt::Display manually.
///
///
/// # Usage on enums
///
/// Add the attribute to the enum definition like this
/// ```
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
/// ```
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
/// The 'impl_from' parameter is exclusive to enum variants. If set, a std::convert::From for the
/// variant is created. This is very helpful if you want to use the question mark operator. This only works if the variant has exactly one field.
/// 'impl_from' is interpreted as a boolean and can also be written as 'impl_from = true'.
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