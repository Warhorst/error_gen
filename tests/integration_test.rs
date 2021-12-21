use std::error::Error;
use std::fmt::{Debug, Display, Formatter};

use error_gen::error;

#[test]
fn unit_no_message_works() {
    #[error]
    struct S;

    impl std::fmt::Display for S {
        fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
            write!(f, "unit message manual")
        }
    }

    check_error_implementation_works(S, "unit message manual")
}

#[test]
fn unit_works() {
    #[error(message = "unit struct")]
    struct S;

    check_error_implementation_works(S, "unit struct")
}

#[test]
fn tuple_works() {
    #[error(message = "tuple like {0}")]
    struct S(usize);

    check_error_implementation_works(S(42), "tuple like 42")
}

#[test]
fn tuple_multiple_works() {
    #[error(message = "tuple like multiple {0} {2}")]
    struct S(usize, usize, usize);

    check_error_implementation_works(S(41, 42, 43), "tuple like multiple 41 43")
}

#[test]
fn named_fields_works() {
    #[error(message = "named fields {i} {j}")]
    struct S {
        i: usize,
        j: usize,
    }

    let s = S { i: 41, j: 42 };
    check_error_implementation_works(s, "named fields 41 42")
}

#[test]
fn generics_works() {
    #[error(message = "generics {0}")]
    struct S<T: Debug + Display>(T);

    check_error_implementation_works(S(42), "generics 42")
}

#[test]
fn generics_where_clause_works() {
    #[error(message = "generics {0}")]
    struct S<T>(T) where T: Debug + Display;

    check_error_implementation_works(S(42), "generics 42")
}

#[test]
fn lifetimes_works() {
    #[error(message = "lifetimes {0}")]
    struct S<'a>(&'a usize);

    let i = 42;
    check_error_implementation_works(S(&i), "lifetimes 42")
}

#[test]
fn lifetimes_and_generics_works() {
    #[error(message = "lifetimes and generics {0}")]
    struct S<'a, T>(&'a T) where T: Debug + Display;

    let i = 42;
    check_error_implementation_works(S(&i), "lifetimes and generics 42")
}

#[test]
fn implement_from_for_named_works() {
    #[error(message = "impl_from named: {val}", impl_from)]
    #[derive(Eq, PartialEq)]
    struct S {
        val: usize,
    }
    check_error_implementation_works(S { val: 42 }, "impl_from named: 42");
    check_from_implementation_works(42, S { val: 42 })
}

#[test]
fn implement_from_for_unnamed_works() {
    #[error(message = "impl_from unnamed: {0}", impl_from)]
    #[derive(Eq, PartialEq)]
    struct S(usize);
    check_error_implementation_works(S(42), "impl_from unnamed: 42");
    check_from_implementation_works(42, S(42))
}

#[test]
fn enum_no_display_message_works() {
    #[error]
    enum E {
        Foo
    }

    impl std::fmt::Display for E {
        fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
            match self {
                E::Foo => write!(f, "Foo")
            }
        }
    }

    check_error_implementation_works(E::Foo, "Foo");
}

#[test]
fn enum_default_message_works() {
    #[error(message = "some default")]
    enum E {
        Foo,
        Bar,
    }

    check_error_implementation_works(E::Foo, "some default");
    check_error_implementation_works(E::Bar, "some default")
}

#[test]
fn enum_custom_message_works() {
    #[error(message = "some default")]
    enum E {
        Foo,
        #[error(message = "some custom for Bar")]
        Bar,
    }

    check_error_implementation_works(E::Foo, "some default");
    check_error_implementation_works(E::Bar, "some custom for Bar")
}

#[test]
fn enum_check_from_works() {
    #[error(message = "some default")]
    #[derive(Eq, PartialEq)]
    enum E {
        #[error(impl_from)]
        Foo(usize)
    }

    check_error_implementation_works(E::Foo(42), "some default");
    check_from_implementation_works(42, E::Foo(42))
}

#[test]
fn enum_check_named_message_works() {
    #[error]
    enum E {
        #[error(message = "Value: {val}")]
        Foo { val: usize }
    }

    check_error_implementation_works(E::Foo { val: 42 }, "Value: 42");
}

#[test]
fn enum_check_positional_message_works() {
    #[error]
    enum E {
        #[error(message = "Float: {1}, Int: {0}")]
        Foo(usize, f32)
    }

    check_error_implementation_works(E::Foo(42, 420.5), "Float: 420.5, Int: 42");
}

#[test]
fn enum_check_generics_and_lifetimes_works() {
    #[error]
    #[derive(Eq, PartialEq)]
    enum E<'a, A, B: Debug + Display> where A: Debug + Display {
        #[error(message = "{0}", impl_from)]
        Where(A),
        #[error(message = "{0}")]
        Generic(B),
        #[error(message = "{0}")]
        Lifetime(&'a usize),
    }

    check_error_implementation_works(E::<'_, usize, usize>::Where(42), "42");
    check_error_implementation_works(E::<'_, usize, usize>::Generic(42), "42");
    check_error_implementation_works(E::<'_, usize, usize>::Lifetime(&42), "42");
    check_from_implementation_works(42, E::<'_, usize, usize>::Where(42))
}

#[test]
fn check_global_impl_from_works() {
    #[error(message = "Error", impl_from)]
    #[derive(Clone, Eq, PartialEq)]
    enum Z {
        A(usize),
        B(&'static str),
    }

    let a = Z::A(42);
    let b = Z::B("42");

    check_error_implementation_works(a.clone(), "Error");
    check_error_implementation_works(b.clone(), "Error");
    check_from_implementation_works(42, a);
    check_from_implementation_works("42", b);
}

/// Check if the given value is a fully qualified Error.
/// It implements all necessary traits if it is a valid parameter for this function.
/// Also its Display-implementation should create the expected message.
fn check_error_implementation_works<E>(e: E, expected_message: &str)
    where E: Error + Debug + Display
{
    assert_eq!(e.to_string(), expected_message);
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