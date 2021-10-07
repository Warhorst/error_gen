use syn::{Path, Attribute, Field, FieldsNamed};
use syn::__private::TokenStream2;
use quote::{format_ident, quote};
use syn::__private::quote::__private::Ident;

const ERROR_ATTRIBUTE: &'static str = "error";

/// Convert a syn::Path to a name (as String)
pub fn path_to_name(path: &Path) -> String {
    path.get_ident().map(|ident| ident.to_string()).expect("The given path was not an identifier.")
}

pub fn attribute_is_error(attribute: &Attribute) -> bool {
    path_to_name(&attribute.path) == ERROR_ATTRIBUTE
}

// TODO: Maybe move the whole display related stuff in separate module

// TODO: Ident to String to Ident = bad
pub fn get_used_identifiers_in_string<'a, I>(display_string: &String, fields: I) -> Vec<Ident>
    where I: IntoIterator<Item=&'a Field> {
    fields.into_iter()
        .map(|field| {
            let field_name = get_field_name(field);
            if display_string.contains(&format!("{{{}}}", field_name)) {
                return Some(format_ident!("{}", field_name));
            }
            None
        })
        .flat_map(Option::into_iter)
        .collect()
}

pub fn create_named_write_parameters<'a, I>(display_string: &String, fields: I) -> Vec<TokenStream2>
    where I: IntoIterator<Item=&'a Field> {
    get_used_identifiers_in_string(display_string, fields)
        .into_iter()
        .map(|ident| quote! {, #ident = self.#ident})
        .collect()
}

pub fn create_named_write_parameters_enum<'a, I>(display_string: &String, fields: I) -> Vec<TokenStream2>
    where I: IntoIterator<Item=&'a Field> {
    get_used_identifiers_in_string(display_string, fields)
        .into_iter()
        .map(|ident| quote! {, #ident = #ident})
        .collect()
}

pub fn create_named_enum_variant_match_arm_parameters(display_string: &String, fields: &FieldsNamed) -> Vec<TokenStream2> {
    let mut tokens = vec![];
    let identifiers = get_used_identifiers_in_string(display_string, &fields.named);

    for (i, ident) in identifiers.iter().enumerate() {
        tokens.push(match i {
            0 => quote! {#ident},
            _ => quote! {,#ident}
        });
    }

    match tokens.len() {
        0 => tokens.push(quote!{..}),
        len if len < fields.named.len() =>  tokens.push(quote! {,..}),
        _ => ()
    };

    tokens
}

fn get_field_name(field: &Field) -> String {
    field.ident.as_ref().unwrap().to_string()
}