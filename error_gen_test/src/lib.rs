#[cfg(test)]
mod struct_tests {
    use std::fmt::{Debug, Formatter};

    use error_gen::{error, e_error};

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
        j: usize
    }

    #[error("Generics. Constraints like 'T: Debug' need te be in a where-clause")]
    struct E5<T>(T) where T: Debug;

    #[error("Lifetimes")]
    struct E6<'a>(&'a usize);

    #[error("Lifetimes and generics")]
    struct E7<'a, T>(&'a T) where T: Debug;

    #[e_error]
    enum E8 {
        #[e_error("Wololo", 5)]
        Foo,
        #[e_error("Wazup")]
        Bar
    }


    impl std::fmt::Display for E0 {
        fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
            write!(f, "Implement Display manually for more flexibility")
        }
    }
}