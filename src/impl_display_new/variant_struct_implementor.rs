use quote::quote;
use syn::{FieldsNamed, FieldsUnnamed, GenericParam, Ident, ItemEnum, Type, Variant};
use syn::Type::*;
use syn::GenericParam::*;
use syn::__private::TokenStream2;
use syn::Fields::*;

/// Creates structs from enum variants, which will be used
/// to access fields like they were structs in the first place.
pub struct VariantStructImplementor<'a> {
    item_enum: &'a ItemEnum,
    variant: &'a Variant,
}

impl<'a> VariantStructImplementor<'a> {
    pub fn new(item_enum: &'a ItemEnum, variant: &'a Variant) -> Self {
        VariantStructImplementor { item_enum, variant }
    }

    pub fn implement(self) -> TokenStream2 {
        match self.variant.fields {
            Named(ref f) => self.implement_named(f),
            Unnamed(ref f) => self.implement_unnamed(f),
            Unit => quote! {}
        }
    }

    fn implement_named(&self, variant_fields: &FieldsNamed) -> TokenStream2 {
        let enum_generics = self.get_enum_generics();

        let mut struct_generics = vec![];
        let fields = variant_fields.named
            .iter()
            .map(|f| {
                let ident = &f.ident;
                let ty = match &f.ty {
                    Type::Reference(r) => {
                        let inner = &r.elem;
                        quote! {&'a #inner}
                    },
                    ty => quote! {#ty}
                };

                let type_ident = get_type_ident(&f.ty);
                if enum_generics.contains(&type_ident) {
                    struct_generics.push(type_ident)
                }
                quote! { #ident: &'a #ty}
            })
            .collect::<Vec<_>>();

        let ident = &self.variant.ident;
        let generics = match (variant_fields.named.len(), struct_generics.len()) {
            (0, 0) => quote! {},
            (_, 0) => quote! {< 'a >},
            _ => quote! {< 'a, #(#struct_generics,)* >}
        };

        quote! {
            struct #ident #generics {
                #(#fields,)*
            }
        }
    }

    fn implement_unnamed(&self, variant_fields: &FieldsUnnamed) -> TokenStream2 {
        let enum_generics = self.get_enum_generics();
        let lifetime = match variant_fields.unnamed.len() {
            0 => quote! {},
            _ => quote! {<'a>}
        };

        let fields = variant_fields.unnamed
            .iter()
            .map(|f| {
                let ty = &f.ty;
                quote! {&'a #ty}
            });

        let ident = &self.variant.ident;
        quote! {
            struct #ident #lifetime(#(#fields,)*);
        }
    }

    fn get_enum_generics(&self) -> Vec<&'a Ident> {
        self.item_enum.generics.params
            .iter()
            .flat_map(|p| match p {
                Type(t) => Some(&t.ident),
                _ => None
            })
            .collect()
    }
}

/// Return the ident of a variants field type. As these can only be
/// paths or references to path, all other possibilities are ignored
/// and considered a programming error.
fn get_type_ident(ty: &Type) -> &Ident {
    match ty {
        Reference(r) => get_type_ident(r.elem.as_ref()),
        Path(p) => &p.path.segments.last().expect("path should have one element").ident,
        _ => panic!("unexpected type, expected was type of variant field")
    }
}

// TODO: lifetimes and generics
#[cfg(test)]
mod tests {
    use syn::{ItemEnum, parse_quote};
    use syn::Variant;

    use crate::common::assert_tokens_are_equal;
    use crate::impl_display_new::variant_struct_implementor::VariantStructImplementor;

    #[test]
    fn from_named_works() {
        let var = parse_quote!(SomeVar {u: usize});
        let ts = implement_for(var);
        let expected = r#"struct SomeVar<'a> {u: &'a usize,}"#;
        assert_tokens_are_equal(ts, expected)
    }

    #[test]
    fn from_named_no_fields_works() {
        let var = parse_quote!(SomeVar {});
        let ts = implement_for(var);
        let expected = r#"struct SomeVar {}"#;
        assert_tokens_are_equal(ts, expected)
    }

    #[test]
    fn from_named_reference_works() {
        let var = parse_quote!(SomeVar {u: &'l usize});
        let ts = implement_for(var);
        let expected = r#"struct SomeVar<'a> {u: &'a &'a usize,}"#;
        assert_tokens_are_equal(ts, expected)
    }

    /// Also check if additional generics from other variants are ignored.
    #[test]
    fn from_named_generics_works() {
        let item_enum = parse_quote!(enum E<T, U> { Foo{ val: T }, Bar {val: U}});
        let var = parse_quote!(Foo {val: T });
        let ts = implement_for_enum(item_enum, var);
        let expected = r#"struct Foo<'a, T,> {val: &'a T,}"#;
        assert_tokens_are_equal(ts, expected)
    }

    #[test]
    fn from_unnamed_works() {
        let var = parse_quote!(SomeVar(usize));
        let ts = implement_for(var);
        let expected = r#"struct SomeVar<'a>(&'a usize,);"#;
        assert_tokens_are_equal(ts, expected)
    }

    #[test]
    fn from_unnamed_no_fields_works() {
        let var = parse_quote!(SomeVar());
        let ts = implement_for(var);
        let expected = r#"struct SomeVar();"#;
        assert_tokens_are_equal(ts, expected)
    }

    #[test]
    fn from_unit_works() {
        let var = parse_quote!(SomeVar);
        let ts = implement_for(var);
        let expected = "";
        assert_tokens_are_equal(ts, expected)
    }

    fn implement_for(var: Variant) -> String {
        VariantStructImplementor::new(&parse_quote!(enum E {}), &var).implement().to_string()
    }

    fn implement_for_enum(item_enum: ItemEnum, var: Variant) -> String {
        VariantStructImplementor::new(&item_enum, &var).implement().to_string()
    }
}