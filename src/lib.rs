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
/// Also, it's possible to generate implementations for std::convert::From for structs and enum variants with a single field.
///
/// The attribute is applicable for struct definitions and enum definitions. If it's used anywhere else,
/// the attribute panics. The only exception are enum variants if the enum definition holds the error attribute.
///
/// # structs
/// ## general usage
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
///         let e = self;
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
/// ## the parameter 'message'
/// The parameter 'message' is of type String. It is optional.
///
/// Providing 'message' will cause an implementation of std::fmt::Display for the struct,
/// while omitting it will allow you to implement it by yourself.
///
/// The message will be used to generate a call of the 'write!' macro. If the message has
/// substrings contained in braces '{...}', these parts will be transformed into expressions
/// in the order they appear. for example
/// ```text
///  message = "This is my val: {e.val}"
///  ```
///  will result in
/// ```text
///  write!(f, "This is my val: {}, e.val")
/// ```
///
/// The braces itself will be lost, so expressions with multiple statements must be contained in
/// an additional pair, like "Complex: {{let mut i = 0; i += 1; i}}".
///
/// To access the error struct itself and its fields/methods, a variable named 'e' will
/// be created, which is just a reference to self. (You could theoretically use self anyway)
///
/// ## the parameter 'impl_from'
/// The parameter 'impl_from' is of type bool. It is optional.
/// Just writing 'impl_from' is equivalent to 'impl_from = true',
/// omitting it is equivalent to 'impl_from = false'.
///
/// When 'impl_from' is true, an implementation of From for the type of
/// the single field of the struct will be created. If the struct has more
/// or less than one field, the attribute panics.
///
/// # enums
/// ## general usage
///
/// Add the attribute to the enum definition like this
/// ``` text
/// #[error(message = "Something went wrong")]
/// enum MyError {
///     Unknown,
///     #[error(message = "Parsing failed in line {line}")]
///     ParsingFailed { line: usize },
///     #[error(message = "Could not read file. Problem: {_0}", impl_from)]
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
///             e @ MyError::ParsingFailed { line } => write!(f, "Parsing failed in line {}", line),
///             e @ MyError::ReadFileFailed(_0) => write!(f, "Could not read file. Problem: {}", e),
///             _ => write!(f, "Something went wrong")
///         }
///     }
/// }
///
/// impl std::convert::From<std::io::Error> for MyError {
///     fn from(e: std::io::Error) -> Self {
///         MyError::ReadFileFailed(e)
///     }
/// }
/// ```
///
/// ## the parameter 'message'
/// The parameter 'message' is of type String. It is optional and can be used on enums and their variants.
///
/// Providing 'message' will cause an implementation of std::fmt::Display for the enum.
/// If neither the enum nor any of its variants has message set, Display will not be implemented.
/// The created implementation has the same capabilities like enums, so you can use expressions
/// to create better messages.
///
/// ### on enums
/// The value of 'message' on the enum itself will be used to generate a default message for every variant
/// without the 'message' parameter set. Just like structs, a variable named 'e' will be created, which
/// is just a reference to self.
///
/// ### on variants
/// A specific match arm in the Display implementation will be created when 'message' is used on a variant. Based
/// on its kind, the fields of the variant will be exposed and can be used in expressions:
///
/// If the variant uses named fields, all names will be usable just by their name. When using tuple like variants,
/// you can use the index of the field beginning with an underscore, like '_0' (as numbers aren't valid identifiers)
///
/// Important: As variants aren't types itself (yet), you cannot call e.field or e.0, as the exposed variable 'e'
/// will be the whole enum.
///
///
/// ## the parameter 'impl_from'
/// The parameter 'impl_from' is of type bool. It is optional.
/// Just writing 'impl_from' is equivalent to 'impl_from = true',
/// omitting it is equivalent to 'impl_from = false'.
///
/// Like for structs, it indicates that std::convert::From should be implemented for
/// an enum.
///
/// ### on enums
/// When used on enums, error_gen tries to create From implementations for every variant of the enum.
/// This only works if every variant has only one field.
///
/// ### on variants
/// When used on a variant, error_gen tries to implement From for the type of the variants single field.
/// This fails if the variant has more or less than one field.
///
/// # Important
/// error_gen will not check if
///  the expressions in your Display messages are correct.
///  OR your chosen items for the From implementation interfere with other code.
/// This might lead to strange compiler errors due to wrong implementations.
#[proc_macro_attribute]
pub fn error(attributes: TokenStream, item: TokenStream) -> TokenStream {
    if let Ok(item_struct) = parse::<ItemStruct>(item.clone()) {
        return struct_error::implement(parse_macro_input!(attributes as AttributeArgs), item_struct);
    }

    if let Ok(item_enum) = parse::<ItemEnum>(item) {
        return enum_error::implement(parse_macro_input!(attributes as AttributeArgs), item_enum);
    }

    panic!("The error attribute is only allowed on structs, enums and enum variants.")
}