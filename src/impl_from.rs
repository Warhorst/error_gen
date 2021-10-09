use syn::{ItemEnum, Variant, FieldsNamed, FieldsUnnamed};
use syn::__private::TokenStream2;
use quote::quote;
use syn::Fields::*;

/// Create an implementation of the std::convert::From trait, based on the type of the enum variant.
///
/// From can only be implemented if the variant is not Unit or the variant only holds one field.
pub fn create_from_implementation(item_enum: &ItemEnum, variant: &Variant) -> TokenStream2 {
    match &variant.fields {
        Named(fields) => create_for_fields_named(item_enum, variant, fields),
        Unnamed(fields) => create_for_fields_unnamed(item_enum, variant, fields),
        Unit => panic!("Cannot implement From trait for Unit variants.")
    }
}

fn create_for_fields_named(item_enum: &ItemEnum, variant: &Variant, fields: &FieldsNamed) -> TokenStream2 {
    if fields.named.len() != 1 {
        panic!("From trait can only be implemented for variants with one field.")
    }

    let enum_ident = &item_enum.ident;
    let generics = &item_enum.generics;
    let where_clause = &generics.where_clause;
    let variant_ident = &variant.ident;

    let field = fields.named.first().unwrap();
    let ty = &field.ty;
    let field_ident = field.ident.as_ref().unwrap();

    quote! {
        impl #generics std::convert::From<#ty> for #enum_ident #generics #where_clause {
            fn from(val: #ty) -> Self {
                #enum_ident::#variant_ident{ #field_ident : val }
            }
        }
    }
}

fn create_for_fields_unnamed(item_enum: &ItemEnum, variant: &Variant, fields: &FieldsUnnamed) -> TokenStream2 {
    if fields.unnamed.len() != 1 {
        panic!("From trait can only be implemented for variants with one field.")
    }

    let enum_ident = &item_enum.ident;
    let generics = &item_enum.generics;
    let where_clause = &generics.where_clause;
    let variant_ident = &variant.ident;

    let field = fields.unnamed.first().unwrap();
    let ty = &field.ty;

    quote! {
        impl #generics std::convert::From<#ty> for #enum_ident #generics #where_clause {
            fn from(val: #ty) -> Self {
                #enum_ident::#variant_ident(val)
            }
        }
    }
}