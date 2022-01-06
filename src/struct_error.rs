use quote::quote;
use syn::{AttributeArgs, ItemStruct};
use syn::__private::TokenStream2;

use crate::impl_display::structs::StructDisplayImplementor;
use crate::impl_from::structs::StructFromImplementer;
use crate::parameters::Parameters;

/// Generate the implementations for a given struct to be a fully qualified and
/// usable error. This means
///
/// - std::error::Error is implemented
/// - std::fmt::Debug and Display are implemented
/// - std::convert::From is implemented (if possible) to allow the usage of the ?-operator
pub fn implement(attr_args: AttributeArgs, item_struct: ItemStruct) -> TokenStream2 {
    let parameters = Parameters::from_attribute_args(attr_args);

    let ident = &item_struct.ident;
    let generics = &item_struct.generics;
    let (impl_generics, type_generics, where_clause) = generics.split_for_impl();

    let display_implementation = StructDisplayImplementor::new(&item_struct, &parameters).implement();
    let from_implementation = match StructFromImplementer::new(&item_struct, &parameters).implement() {
        Ok(implementation) => implementation,
        Err(e) => panic!("{}", e)
    };

    quote! {
        #[derive(Debug)] #item_struct
        impl #impl_generics std::error::Error for #ident #type_generics #where_clause {}
        #display_implementation
        #from_implementation
    }
}

#[cfg(test)]
mod tests {
    use crate::assert_implementation_as_expected;

    #[test]
    fn named_no_parameters() {
        assert_implementation_as_expected!(
            item: {
                #[error]
                struct S {
                    foo: usize
                }
            }

            expected: {
                #[derive(Debug)]
                struct S {
                    foo: usize
                }

                impl std::error::Error for S {}
            }
        )
    }

    #[test]
    fn named_no_fields_no_parameters() {
        assert_implementation_as_expected!(
            item: {
                #[error]
                struct S {
                }
            }

            expected: {
                #[derive(Debug)]
                struct S {
                }

                impl std::error::Error for S {}
            }
        )
    }

    #[test]
    fn unnamed_no_parameters() {
        assert_implementation_as_expected!(
            item: {
                #[error]
                struct S (usize);
            }

            expected: {
                #[derive(Debug)]
                struct S (usize);

                impl std::error::Error for S {}
            }
        )
    }

    #[test]
    fn unnamed_no_fields_no_parameters() {
        assert_implementation_as_expected!(
            item: {
                #[error]
                struct S ();
            }

            expected: {
                #[derive(Debug)]
                struct S ();

                impl std::error::Error for S {}
            }
        )
    }

    #[test]
    fn unit_no_parameters() {
        assert_implementation_as_expected!(
            item: {
                #[error]
                struct S;
            }

            expected: {
                #[derive(Debug)]
                struct S;

                impl std::error::Error for S {}
            }
        )
    }

    #[test]
    fn other_attributes_remain() {
        assert_implementation_as_expected!(
            item: {
                #[error]
                #[derive(Clone)]
                struct S;
            }

            expected: {
                #[derive(Debug)]
                #[derive(Clone)]
                struct S;

                impl std::error::Error for S {}
            }
        )
    }

    #[test]
    fn lifetimes_remain() {
        assert_implementation_as_expected!(
            item: {
                #[error]
                struct S<'a> (&'a usize);
            }

            expected: {
                #[derive(Debug)]
                struct S<'a> (&'a usize);

                impl<'a> std::error::Error for S<'a> {}
            }
        )
    }

    #[test]
    fn generics_remain() {
        assert_implementation_as_expected!(
            item: {
                #[error]
                struct S<A: Clone, B> (A, B) where B: Clone;
            }

            expected: {
                #[derive(Debug)]
                struct S<A: Clone, B> (A, B) where B: Clone;

                impl<A: Clone, B> std::error::Error for S<A, B> where B: Clone {}
            }
        )
    }

    #[test]
    fn const_generics_remain() {
        assert_implementation_as_expected!(
            item: {
                #[error]
                struct S<const C: usize> (C);
            }

            expected: {
                #[derive(Debug)]
                struct S<const C: usize> (C);

                impl<const C: usize> std::error::Error for S<C> {}
            }
        )
    }

    #[test]
    fn named_impl_from() {
        assert_implementation_as_expected!(
            item: {
                #[error(impl_from)]
                struct S {
                    foo: usize
                }
            }

            expected: {
                #[derive(Debug)]
                struct S {
                    foo: usize
                }

                impl std::error::Error for S {}

                impl std::convert::From<usize> for S {
                    fn from(val: usize) -> Self {
                        S{ foo : val }
                    }
                }
            }
        )
    }

    #[test]
    #[should_panic(expected = "'std::convert::From' cannot be implemented for struct 'S', as it has not exactly one field.")]
    fn named_impl_from_no_fields_should_panic() {
        assert_implementation_as_expected!(
            item: {
                #[error(impl_from)]
                struct S {}
            }

            expected: {
                should not work
            }
        )
    }

    #[test]
    #[should_panic(expected = "'std::convert::From' cannot be implemented for struct 'S', as it has not exactly one field.")]
    fn named_impl_from_two_fields_should_panic() {
        assert_implementation_as_expected!(
            item: {
                #[error(impl_from)]
                struct S {one: usize, two: usize}
            }

            expected: {
                should not work
            }
        )
    }

    #[test]
    fn unnamed_impl_from() {
        assert_implementation_as_expected!(
            item: {
                #[error(impl_from)]
                struct S (usize);
            }

            expected: {
                #[derive(Debug)]
                struct S (usize);

                impl std::error::Error for S {}

                impl std::convert::From<usize> for S {
                    fn from(val: usize) -> Self {
                        S(val)
                    }
                }
            }
        )
    }

    #[test]
    #[should_panic(expected = "'std::convert::From' cannot be implemented for struct 'S', as it has not exactly one field.")]
    fn unnamed_impl_from_no_fields_should_panic() {
        assert_implementation_as_expected!(
            item: {
                #[error(impl_from)]
                struct S();
            }

            expected: {
                should not work
            }
        )
    }

    #[test]
    #[should_panic(expected = "'std::convert::From' cannot be implemented for struct 'S', as it has not exactly one field.")]
    fn unnamed_impl_from_two_fields_should_panic() {
        assert_implementation_as_expected!(
            item: {
                #[error(impl_from)]
                struct S(usize, usize);
            }

            expected: {
                should not work
            }
        )
    }

    #[test]
    #[should_panic(expected = "'std::convert::From' cannot be implemented for struct 'S', as it has not exactly one field.")]
    fn unit_impl_from_should_panic() {
        assert_implementation_as_expected!(
            item: {
                #[error(impl_from)]
                struct S;
            }

            expected: {
                should not work
            }
        )
    }

    #[test]
    fn named_impl_display() {
        assert_implementation_as_expected!(
            item: {
                #[error(message = "My foo value: {e.foo}")]
                struct S {
                    foo: usize
                }
            }

            expected: {
                #[derive(Debug)]
                struct S {
                    foo: usize
                }

                impl std::error::Error for S {}

                impl std::fmt::Display for S {
                    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        let e = self;
                        write!(f, "My foo value: {}", e.foo)
                    }
                }
            }
        )
    }

    #[test]
    fn unnamed_impl_display() {
        assert_implementation_as_expected!(
            item: {
                #[error(message = "My single value: {e.0}")]
                struct S (usize);
            }

            expected: {
                #[derive(Debug)]
                struct S (usize);

                impl std::error::Error for S {}

                impl std::fmt::Display for S {
                    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        let e = self;
                        write!(f, "My single value: {}", e.0)
                    }
                }
            }
        )
    }

    #[test]
    fn unit_impl_display() {
        assert_implementation_as_expected!(
            item: {
                #[error(message = "Something went wrong")]
                struct S;
            }

            expected: {
                #[derive(Debug)]
                struct S;

                impl std::error::Error for S {}

                impl std::fmt::Display for S {
                    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        let e = self;
                        write!(f, "Something went wrong")
                    }
                }
            }
        )
    }

    #[test]
    fn named_impl_from_and_display() {
        assert_implementation_as_expected!(
            item: {
                #[error(message = "My foo value: {e.foo}", impl_from)]
                struct S {
                    foo: usize
                }
            }

            expected: {
                #[derive(Debug)]
                struct S {
                    foo: usize
                }

                impl std::error::Error for S {}

                impl std::fmt::Display for S {
                    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        let e = self;
                        write!(f, "My foo value: {}", e.foo)
                    }
                }

                impl std::convert::From<usize> for S {
                    fn from(val: usize) -> Self {
                        S{ foo : val }
                    }
                }
            }
        )
    }

    #[test]
    fn unnamed_impl_from_and_display() {
        assert_implementation_as_expected!(
            item: {
                #[error(message = "My single value: {e.0}", impl_from)]
                struct S (usize);
            }

            expected: {
                #[derive(Debug)]
                struct S (usize);

                impl std::error::Error for S {}

                impl std::fmt::Display for S {
                    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        let e = self;
                        write!(f, "My single value: {}", e.0)
                    }
                }

                impl std::convert::From<usize> for S {
                    fn from(val: usize) -> Self {
                        S(val)
                    }
                }
            }
        )
    }

    /// Assert that the generated code for a given struct is as expected.
    ///
    /// Generates the code and compares the token streams (as strings) with
    /// each other.
    #[macro_export]
    macro_rules! assert_implementation_as_expected {
        (item: {$($item_struct:tt)*}  expected: {$($expected:tt)*}) => {
            {
                let mut item_struct: syn::ItemStruct = syn::parse_quote!($($item_struct)*);
                let error_attribute_index = item_struct.attrs
                    .iter()
                    .enumerate()
                    .find(|(_, a)|crate::common::attribute_is_error(a))
                    .expect("One attribute should be 'error'").0;

                let attribute_args = crate::test_helper::extract_attribute_args(item_struct.attrs.remove(error_attribute_index));
                let implementation_ts = crate::struct_error::implement(attribute_args, item_struct).to_string();
                let expected = quote::quote!($($expected)*).to_string();
                crate::test_helper::assert_tokens_are_equal(implementation_ts, expected)
            }
        };
    }
}