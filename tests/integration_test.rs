use std::fmt::{Debug, Formatter, Display};

use error_gen::error;
use std::error::Error;

#[test]
fn unit_no_message_works() {
    #[error]
    struct S;

    impl std::fmt::Display for S {
        fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
            write!(f, "unit message manual")
        }
    }

    check_struct_implementation_works(S, "unit message manual")
}

#[test]
fn unit_works() {
    #[error(message = "unit struct")]
    struct S;

    check_struct_implementation_works(S, "unit struct")
}

#[test]
fn tuple_works() {
    #[error(message = "tuple like {0}")]
    struct S(usize);

    check_struct_implementation_works(S(42), "tuple like 42")
}

#[test]
fn tuple_multiple_works() {
    #[error(message = "tuple like multiple {0} {2}")]
    struct S(usize, usize, usize);

    check_struct_implementation_works(S(41, 42, 43), "tuple like multiple 41 43")
}

#[test]
fn named_fields_works() {
    #[error(message = "named fields {i} {j}")]
    struct S {
        i: usize,
        j: usize,
    }

    let s = S { i: 41, j: 42 };
    check_struct_implementation_works(s, "named fields 41 42")
}

#[test]
fn generics_works() {
    #[error(message = "generics {0}")]
    struct S<T>(T) where T: Debug + Display;

    check_struct_implementation_works(S(42), "generics 42")
}

#[test]
fn lifetimes_works() {
    #[error(message = "lifetimes {0}")]
    struct S<'a>(&'a usize);

    let i = 42;
    check_struct_implementation_works(S(&i), "lifetimes 42")
}

#[test]
fn lifetimes_and_generics_works() {
    #[error(message = "lifetimes and generics {0}")]
    struct S<'a, T>(&'a T) where T: Debug + Display;

    let i = 42;
    check_struct_implementation_works(S(&i), "lifetimes and generics 42")
}

/// Check if the struct is a fully qualified Error.
/// It implements all necessary traits if it is a valid parameter for this function.
/// Also its Display-implementation should create the expected message.
fn check_struct_implementation_works<S>(s: S, expected_message: &str)
    where S: Error + Debug + Display
{
    assert_eq!(s.to_string(), expected_message);
}

/// If this method call does not produce a compile error, From was correctly
/// implemented for the given type f. Also checks if the expected value
/// was produced.
fn check_from_implementation_works<E, F>(from_value: F, expected: E)
    where E: Error + Debug + Display + From<F> + Eq {

    let fun: fn(F) -> Result<(), E> = |f| {
        Err(f)?;
        Ok(())
    };
    assert_eq!(expected, fun(from_value).err().unwrap());
}

#[error(message = "some default")]
enum E8<T> where T: Debug {
    #[error(message = "A foo occurred {1}, {3}, {1}, {2}, {0}. Here some random number: 0.")]
    Foo(usize, f32, usize, u8),
    #[error(message = "A wild bar appeared: {some_val}, {some_other_val}")]
    Bar { some_val: f32, some_other_val: usize },
    #[error(message = "Generic and dangerous")]
    Baz(T),
    #[error(message = "This is some error")]
    Oof,
    #[error(impl_from)]
    Rab(usize),
}