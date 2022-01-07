# error_gen
A crate to generate boilerplate code necessary for fully qualified errors in Rust.

## Motivation
Imagine a complex method with multiple possible failures, like parsing an external resource. You need to
- receive the resource from an external server
- parse the resource to a domain object
- read a necessary value and check if it is valid

A typical approach to this task is to create an error for every single operation, like so
``` rust
struct NetworkError { pub response_code: usize }

struct ParseError { pub line: usize }

struct InvalidValueError(pub f32)
```

To create an error for the complex operation, an enum is created with its variants holding the single error values, like so

``` rust
enum ComplexError {
    Network(NetworkError),
    Parsing(ParseError),
    InvalidValue(InvalidValueError)
}
``` 

And the final method would look like this
``` rust
fn complex_function() -> Result<f32, ComplexError> {
    let file = receive_file()?; // maybe a NetworkError
    let the_object = parse_file(file)?; // maybe a ParseError
    let value = check_and_get_value(the_object)?; // maybe an InvalidValueError
    value
}
```

The problem: If we want a fully qualified error, every of our errors should implement the std::error::Error trait, which in turn requires std::fmt::Debug and std::fmt::Display. 'Debug' can be derived, but 'Error' and 'Display' must be implemented manually. Also, to use the question mark operator like above, we need three extra std::convert::From implementations for 'ComplexError'.

In total, 11 additional impl-blocks (4 Error, 4 Display, 3 From) are required to model this (still quite simple) example. There are solutions to this problem, like the 'quick_error' crate, but I wanted a more elegant way to generate this boilerplate code. Being used to codegen-libraries like lombok for Java, I wanted to create something similar, resulting in the 'error' attribute.

## Usage
A more detailed description on how to use this attribute in general is documented on the attribute itself. For now, I just show how to use it on the former example.

To use our 'complex_method' like shown, add the 'error' attribute like so

``` rust
#[error(message = "The external server returned code {self.response_code} instead of 200.")]
struct NetworkError { pub response_code: usize }

#[error(message = "Syntax error in line {self.line}.")]
struct ParseError { pub line: usize }

#[error(message = "Invalid value, expected '42' but got {self.0}.")]
struct InvalidValueError(pub f32)

#[error(impl_from)]
enum ComplexError {
    #[error(message = "Error while receiving the external file: {_0}")]
    Network(NetworkError),
    #[error(message = "Error while parsing the file: {_0}")]
    Parsing(ParseError),
    #[error(message = "Error while retreiving the value: {_0}")]
    InvalidValue(InvalidValueError)
}

fn complex_function() -> Result<f32, ComplexError> {
    let file = receive_file()?;
    let the_object = parse_file(file)?;
    let value = check_and_get_value(the_object)?;
    value
}
```

This will create any required implementation (Error, Display, From) with much less code.

## Downsides
- Lack of IDE support for these kinds of macros. The IDE will warn you about upcoming compiler errors regarding not implemented traits.

## Future plans
- push 'error_gen' to crates.io 