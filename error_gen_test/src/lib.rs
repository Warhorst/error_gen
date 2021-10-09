// Todo replace with real tests
#[cfg(test)]
mod struct_tests {
    use std::fmt::{Debug, Formatter};

    use error_gen::error;

    /// No message, implement Display manually
    #[error]
    struct E0;

    #[error("Unit struct")]
    struct E1;

    #[error("Tuple like struct with positional parameters {0}")]
    struct E2(usize);

    #[error("Tuple like struct where positions are ignored {0} {2}")]
    struct E3(usize, usize, usize);

    #[error("Struct with named fields. Reference them in the message by name {i}")]
    struct E4 {
        i: usize,
        j: usize,
    }

    #[error("Generics. Constraints like 'T: Debug' need te be in a where-clause")]
    struct E5<T>(T) where T: Debug;

    #[error("Lifetimes")]
    struct E6<'a>(&'a usize);

    #[error("Lifetimes and generics")]
    struct E7<'a, T>(&'a T) where T: Debug;

    // TODO: Rename "description" to "message" and "derive_from" to "impl_from"
    #[error(description = "some default")]
    enum E8<T> where T: Debug {
        #[error(description = "A foo occurred {1}, {3}, {1}, {2}, {0}. Her some random number: 0.")]
        Foo(usize, f32, usize, u8),
        #[error(description = "A wild bar appeared: {some_val}, {some_other_val}")]
        Bar { some_val: f32, some_other_val: usize },
        #[error(description = "Generic and dangerous")]
        Baz(T),
        #[error(description = "This is some error")]
        Oof,
        #[error(derive_from)]
        Rab(usize)
    }

    impl std::fmt::Display for E0 {
        fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
            write!(f, "Implement Display manually for more flexibility")
        }
    }

    #[test]
    fn test() {
        assert_eq!(Some('a'), char::from_u32(97))
    }
}
