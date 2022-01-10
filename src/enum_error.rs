use quote::quote;
use syn::{Attribute, AttributeArgs, ItemEnum, Variant};
use syn::__private::TokenStream2;

use crate::common::*;
use crate::impl_display::enums::EnumDisplayImplementor;
use crate::impl_from::enums::EnumFromImplementer;
use crate::parameters::Parameters;

pub type VariantWithParams<'a> = (&'a Variant, Option<Parameters>);

/// Generate the implementations for a given enum to be a fully qualified and
/// usable error. This means
///
/// - std::error::Error is implemented
/// - std::fmt::Debug and Display are implemented
/// - std::convert::From is implemented (if possible) to allow the usage of the ?-operator
pub fn implement(attr_args: AttributeArgs, mut item_enum: ItemEnum) -> TokenStream2 {
    let enum_parameters = Parameters::from_attribute_args(attr_args);

    let variants_with_parameters = item_enum.variants
        .iter()
        .map(to_variant_with_parameters)
        .collect::<Vec<_>>();

    let display_implementation = match EnumDisplayImplementor::new(&item_enum, &enum_parameters, &variants_with_parameters).implement() {
        Ok(implementation) => implementation,
        Err(e) => panic!("{}", e)
    };
    let from_implementations = match EnumFromImplementer::new(&item_enum, &enum_parameters, &variants_with_parameters).implement() {
        Ok(implementation) => implementation,
        Err(e) => panic!("{}", e)
    };

    remove_variant_attributes(&mut item_enum);

    let ident = &item_enum.ident;
    let generics = &item_enum.generics;
    let (impl_generics, type_generics, where_clause) = generics.split_for_impl();

    quote! {
        #[derive(Debug)] #item_enum
        impl #impl_generics std::error::Error for #ident #type_generics #where_clause {}

        #from_implementations

        #display_implementation
    }
}

fn to_variant_with_parameters(variant: &Variant) -> VariantWithParams {
    match get_error_attribute(&variant.attrs) {
        Some(attr) => (variant, Some(Parameters::from_attribute(attr))),
        None => (variant, None)
    }
}

fn get_error_attribute(attributes: &Vec<Attribute>) -> Option<&Attribute> {
    let index = attributes
        .iter()
        .enumerate()
        .find_map(|(i, att)| match attribute_is_error(att) {
            true => Some(i),
            false => None
        })?;
    attributes.get(index)
}

//  Attributes on non items seem to be only allowed as helper attributes in custom derives
//  (https://doc.rust-lang.org/reference/procedural-macros.html#derive-macro-helper-attributes). proc_macro_aatributes on the other hand are only allowed
//  on items (https://doc.rust-lang.org/reference/items.html) and need to be removed manually.
fn remove_variant_attributes(item_enum: &mut ItemEnum) {
    item_enum.variants
        .iter_mut()
        .for_each(remove_error_attribute_from_variant)
}

/// Search the index of the error attribute in the given variants attributes.
/// If the index could be found, remove the entry from the variants attributes.
fn remove_error_attribute_from_variant(variant: &mut Variant) {
    let index_opt = variant.attrs
        .iter()
        .enumerate()
        .find_map(|(i, att)| match attribute_is_error(att) {
            true => Some(i),
            false => None
        });

    if let Some(i) = index_opt {
        variant.attrs.remove(i);
    }
}

#[cfg(test)]
mod tests {
    use crate::assert_enum_implementation_as_expected;

    #[test]
    fn no_parameters() {
        assert_enum_implementation_as_expected!(
            item: {
                #[error]
                enum E {
                    Named {foo: usize},
                    Unnamed(usize),
                    Unit
                }
            }

            expected: {
                #[derive(Debug)]
                enum E {
                    Named {foo: usize},
                    Unnamed(usize),
                    Unit
                }

                impl std::error::Error for E {}
            }
        )
    }

    #[test]
    fn no_parameters_no_fields() {
        assert_enum_implementation_as_expected!(
            item: {
                #[error]
                enum E {
                    Named {},
                    Unnamed(),
                    Unit
                }
            }

            expected: {
                #[derive(Debug)]
                enum E {
                    Named {},
                    Unnamed(),
                    Unit
                }

                impl std::error::Error for E {}
            }
        )
    }

    // TODO: Check if Variant attributes remain too
    #[test]
    fn other_attributes_remain() {
        assert_enum_implementation_as_expected!(
            item: {
                #[error]
                #[derive(Clone)]
                enum E {
                    Unit
                }
            }

            expected: {
                #[derive(Debug)]
                #[derive(Clone)]
                enum E {
                    Unit
                }

                impl std::error::Error for E {}
            }
        )
    }

    #[test]
    fn lifetimes_remain() {
        assert_enum_implementation_as_expected!(
            item: {
                #[error]
                enum E<'a> {
                    Unnamed(&'a usize)
                }
            }

            expected: {
                #[derive(Debug)]
                enum E<'a> {
                    Unnamed(&'a usize)
                }

                impl<'a> std::error::Error for E<'a> {}
            }
        )
    }

    #[test]
    fn generics_remain() {
        assert_enum_implementation_as_expected!(
            item: {
                #[error]
                enum E<T> {
                    Unnamed(T)
                }
            }

            expected: {
                #[derive(Debug)]
                enum E<T> {
                    Unnamed(T)
                }

                impl<T> std::error::Error for E<T> {}
            }
        )
    }

    #[test]
    fn const_generics_remain() {
        assert_enum_implementation_as_expected!(
            item: {
                #[error]
                enum E<const C: usize> {
                    Unnamed(C)
                }
            }

            expected: {
                #[derive(Debug)]
                enum E<const C: usize> {
                    Unnamed(C)
                }

                impl<const C: usize> std::error::Error for E<C> {}
            }
        )
    }

    #[test]
    fn named_unnamed_impl_from() {
        assert_enum_implementation_as_expected!(
            item: {
                #[error]
                enum E {
                    #[error(impl_from)]
                    Named {foo: usize},
                    #[error(impl_from)]
                    Unnamed(f32),
                    Unit
                }
            }

            expected: {
                #[derive(Debug)]
                enum E {
                    Named {foo: usize},
                    Unnamed(f32),
                    Unit
                }

                impl std::error::Error for E {}

                impl std::convert::From<usize> for E {
                    fn from(val: usize) -> Self {
                        E::Named {foo: val}
                    }
                }

                impl std::convert::From<f32> for E {
                    fn from(val: f32) -> Self {
                        E::Unnamed(val)
                    }
                }
            }
        )
    }

    #[test]
    fn named_unnamed_global_impl_from() {
        assert_enum_implementation_as_expected!(
            item: {
                #[error(impl_from)]
                enum E {
                    Named {foo: usize},
                    Unnamed(f32),
                }
            }

            expected: {
                #[derive(Debug)]
                enum E {
                    Named {foo: usize},
                    Unnamed(f32),
                }

                impl std::error::Error for E {}

                impl std::convert::From<usize> for E {
                    fn from(val: usize) -> Self {
                        E::Named {foo: val}
                    }
                }

                impl std::convert::From<f32> for E {
                    fn from(val: f32) -> Self {
                        E::Unnamed(val)
                    }
                }
            }
        )
    }

    #[test]
    #[should_panic(expected = "'std::convert::From' cannot be implemented for enum 'E'. The following variants don't have exactly one field: Unit")]
    fn unit_impl_from_should_panic() {
        assert_enum_implementation_as_expected!(
            item: {
                #[error]
                enum E {
                    #[error(impl_from)]
                    Unit
                }
            }

            expected: {
                should panic
            }
        )
    }

    #[test]
    #[should_panic(expected = "'std::convert::From' cannot be implemented for enum 'E'. The following variants don't have exactly one field: Named,Unnamed,Unit")]
    fn impl_from_multiple_errors_should_panic() {
        assert_enum_implementation_as_expected!(
            item: {
                #[error]
                enum E {
                    #[error(impl_from)]
                    Named {one: usize, two: usize},
                    #[error(impl_from)]
                    Unnamed(f32, f32),
                    #[error(impl_from)]
                    Unit
                }
            }

            expected: {
                should panic
            }
        )
    }

    #[test]
    #[should_panic(expected = "'std::convert::From' cannot be implemented for enum 'E'. The following variants don't have exactly one field: Named,Unnamed,Unit")]
    fn impl_from_global_multiple_errors_should_panic() {
        assert_enum_implementation_as_expected!(
            item: {
                #[error(impl_from)]
                enum E {
                    Named {one: usize, two: usize},
                    Unnamed(f32, f32),
                    Unit
                }
            }

            expected: {
                should panic
            }
        )
    }

    #[test]
    #[should_panic(expected = "The 'impl_from' parameter was set on enum 'E' and at least one of its variants. Choose only one (enum or variants).")]
    fn impl_from_on_variant_and_global_should_panic() {
        assert_enum_implementation_as_expected!(
            item: {
                #[error(impl_from)]
                enum E {
                    #[error(impl_from)]
                    Named {foo: usize},
                    Unnamed(f32),
                }
            }

            expected: {
                should panic
            }
        )
    }

    #[test]
    fn impl_display() {
        assert_enum_implementation_as_expected!(
            item: {
                #[error]
                enum E {
                    #[error(message = "The foo value: {foo}")]
                    Named {foo: usize},
                    #[error(message = "The first value: {_0}")]
                    Unnamed(f32),
                    #[error(message = "Something went wrong")]
                    Unit
                }
            }

            expected: {
                #[derive(Debug)]
                enum E {
                    Named {foo: usize},
                    Unnamed(f32),
                    Unit
                }

                impl std::error::Error for E {}

                impl std::fmt::Display for E {
                    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        match self {
                            E::Named {foo,} => write!(f, "The foo value: {}", foo),
                            E::Unnamed (_0,) => write!(f, "The first value: {}", _0),
                            E::Unit => write!(f, "Something went wrong"),
                        }
                    }
                }
            }
        )
    }

    #[test]
    fn impl_display_default() {
        assert_enum_implementation_as_expected!(
            item: {
                #[error(message = "Something went wrong")]
                enum E {
                    #[error(message = "The foo value: {foo}")]
                    Named {foo: usize},
                    #[error(message = "The first value: {_0}")]
                    Unnamed(f32),
                    Unit
                }
            }

            expected: {
                #[derive(Debug)]
                enum E {
                    Named {foo: usize},
                    Unnamed(f32),
                    Unit
                }

                impl std::error::Error for E {}

                impl std::fmt::Display for E {
                    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        match self {
                            E::Named {foo,} => write!(f, "The foo value: {}", foo),
                            E::Unnamed (_0,) => write!(f, "The first value: {}", _0),
                            _ => write!(f, "Something went wrong")
                        }
                    }
                }
            }
        )
    }

    #[test]
    #[should_panic(expected = "All variants for enum 'E' have a Display message, but a default was provided anyways. Please remove the default.")]
    fn impl_display_unnecessary_default_should_panic() {
        assert_enum_implementation_as_expected!(
            item: {
                #[error(message = "Some default")]
                enum E {
                    #[error(message = "The foo value: {foo}")]
                    Named {foo: usize},
                    #[error(message = "The first value: {_0}")]
                    Unnamed(f32),
                    #[error(message = "Something went wrong")]
                    Unit
                }
            }

            expected: {
                should panic
            }
        )
    }

    #[test]
    fn impl_from_and_display() {
        assert_enum_implementation_as_expected!(
            item: {
                #[error]
                enum E {
                    #[error(message = "The foo value: {foo}", impl_from)]
                    Named {foo: usize},
                    #[error(message = "The first value: {_0}", impl_from)]
                    Unnamed(f32),
                    #[error(message = "Something went wrong")]
                    Unit
                }
            }

            expected: {
                #[derive(Debug)]
                enum E {
                    Named {foo: usize},
                    Unnamed(f32),
                    Unit
                }

                impl std::error::Error for E {}

                impl std::convert::From<usize> for E {
                    fn from(val: usize) -> Self {
                        E::Named {foo: val}
                    }
                }

                impl std::convert::From<f32> for E {
                    fn from(val: f32) -> Self {
                        E::Unnamed(val)
                    }
                }

                impl std::fmt::Display for E {
                    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        match self {
                            E::Named {foo,} => write!(f, "The foo value: {}", foo),
                            E::Unnamed (_0,) => write!(f, "The first value: {}", _0),
                            E::Unit => write!(f, "Something went wrong"),
                        }
                    }
                }
            }
        )
    }

    /// Assert that the generated code for a given enum is as expected.
    ///
    /// Generates the code and compares the token streams (as strings) with
    /// each other.
    #[macro_export]
    macro_rules! assert_enum_implementation_as_expected {
        (item: {$($item_enum:tt)*}  expected: {$($expected:tt)*}) => {
            {
                let mut item_enum: syn::ItemEnum = syn::parse_quote!($($item_enum)*);
                let error_attribute_index = item_enum.attrs
                    .iter()
                    .enumerate()
                    .find(|(_, a)|crate::common::attribute_is_error(a))
                    .expect("One attribute should be 'error'").0;

                let attribute_args = crate::test_helper::extract_attribute_args(item_enum.attrs.remove(error_attribute_index));
                let implementation_ts = crate::enum_error::implement(attribute_args, item_enum).to_string();
                let expected = quote::quote!($($expected)*).to_string();
                crate::test_helper::assert_tokens_are_equal(implementation_ts, expected)
            }
        };
    }
}