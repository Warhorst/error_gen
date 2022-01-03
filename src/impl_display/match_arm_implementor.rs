use quote::{format_ident, quote};
use syn::{FieldsNamed, FieldsUnnamed, Ident, Variant};
use syn::__private::TokenStream2;
use syn::Fields::*;

use crate::impl_display::write_implementor::WriteImplementor;

/// Creates match arms for match expressions in an enums std::fmt::Display implementation.
pub struct MatchArmImplementor<'a> {
    enum_ident: &'a Ident,
    message: &'a str
}

impl<'a> MatchArmImplementor<'a> {
    pub fn new(enum_ident: &'a Ident, message: &'a str) -> Self {
        MatchArmImplementor { enum_ident, message }
    }

    pub fn implement_default(self) -> TokenStream2 {
        let write_implementation = WriteImplementor::new().implement(self.message.to_string());

        quote! {
            _ => {
                let e = self; #write_implementation
            }
        }
    }

    pub fn implement_for(self, variant: &Variant) -> TokenStream2 {
        let ident = &variant.ident;
        match &variant.fields {
            Named(f) => self.implement_named(ident, f),
            Unnamed(f) => self.implement_unnamed(ident, f),
            Unit => self.implement_unit(ident)
        }
    }

    fn implement_named(self, variant_ident: &Ident, fields: &FieldsNamed) -> TokenStream2 {
        let field_names = fields.named
            .iter()
            .map(|f| f.ident.as_ref().unwrap());

        let enum_ident = self.enum_ident;
        let write_implementation = WriteImplementor::new().implement(self.message.to_string());

        quote! {
            e @ #enum_ident :: #variant_ident { #(#field_names,)* } => #write_implementation
        }
    }

    fn implement_unnamed(self, variant_ident: &Ident, fields: &FieldsUnnamed) -> TokenStream2 {
        let field_names = (0..fields.unnamed.len())
            .into_iter()
            .map(|i| format!("_{}", i))
            .map(|ident_str| {
                let ident = format_ident!("{}", ident_str);
                quote!(#ident)
            });

        let enum_ident = self.enum_ident;
        let write_implementation = WriteImplementor::new().implement(self.message.to_string());

        quote! {
            e @ #enum_ident :: #variant_ident ( #(#field_names,)* ) => #write_implementation
        }
    }

    fn implement_unit(self, variant_ident: &Ident) -> TokenStream2 {
        let enum_ident = self.enum_ident;
        let write_implementation = WriteImplementor::new().implement(self.message.to_string());

        quote! {
            e @ #enum_ident :: #variant_ident => #write_implementation
        }
    }
}

#[cfg(test)]
mod tests {
    use syn::{parse_quote, Variant};

    use crate::common::assert_tokens_are_equal;
    use crate::impl_display::match_arm_implementor::MatchArmImplementor;

    #[test]
    fn implement_default_works() {
        let message = "something default: {print_cool_message()}";
        let ts = implement_default(message);
        let expected = r#"_ => { let e = self; write!(f, "something default: {}", print_cool_message()) }"#;
        assert_tokens_are_equal(ts, expected)
    }

    #[test]
    fn implement_named_works() {
        let var = parse_quote!(Foo {val: usize});
        let message = "Print the val: {val}";

        let ts = implement_for(var, message);
        let expected = r#"e @ Enum::Foo { val, } => write!(f, "Print the val: {}", val)"#;
        assert_tokens_are_equal(ts, expected)
    }

    #[test]
    fn implement_unnamed_works() {
        let var = parse_quote!(Foo (usize));
        let message = "Print the val: {_0}";

        let ts = implement_for(var, message);
        let expected = r#"e @ Enum::Foo( _0, ) => write!(f, "Print the val: {}", _0)"#;
        assert_tokens_are_equal(ts, expected)
    }

    #[test]
    fn implement_unit_works() {
        let var = parse_quote!(Foo);
        let message = "Print the enum: {e}";

        let ts = implement_for(var, message);
        let expected = r#"e @ Enum::Foo => write!(f, "Print the enum: {}", e)"#;
        assert_tokens_are_equal(ts, expected)
    }

    fn implement_default(message: &str) -> String {
        MatchArmImplementor::new(&parse_quote!(Enum), message).implement_default().to_string()
    }

    /// Implement the match arm for the given variant and message. The enum ident is always "Enum".
    fn implement_for(var: Variant, message: &str) -> String {
        MatchArmImplementor::new(&parse_quote!(Enum), message).implement_for(&var).to_string()
    }
}