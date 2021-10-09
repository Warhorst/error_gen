use syn::{Path, Attribute, Field, FieldsNamed, FieldsUnnamed, Variant};
use syn::__private::TokenStream2;
use quote::{format_ident, quote};
use syn::__private::quote::__private::Ident;
use std::collections::HashMap;

const ERROR_ATTRIBUTE: &'static str = "error";

/// Convert a syn::Path to a name (as String)
pub fn path_to_name(path: &Path) -> String {
    path.get_ident().map(|ident| ident.to_string()).expect("The given path was not an identifier.")
}

pub fn attribute_is_error(attribute: &Attribute) -> bool {
    path_to_name(&attribute.path) == ERROR_ATTRIBUTE
}

// TODO: Maybe move the whole display related stuff in separate module

pub fn create_unnamed_variant_match_arm(message: String, fields: &FieldsUnnamed, ident: &Ident, variant: &Variant, ) -> TokenStream2 {
    let parameter_map = get_unnamed_parameters_in_message(&message, fields);
    create_unnamed_match_arm(parameter_map, ident, variant, fields, message)
}

fn get_unnamed_parameters_in_message(message: &String, fields: &FieldsUnnamed) -> HashMap<usize, UnnamedEnumParamter> {
    let field_amount = fields.unnamed.len();
    (0..field_amount).into_iter()
        .map(|i| match message.contains(&format!("{{{}}}", i)) {
            true => {
                let letter = char::from_u32((i as u32) + 97).unwrap();
                let ident = format_ident!("{}", letter);
                let match_param = if i < field_amount - 1 { quote! {#ident,} } else { quote! {#ident} };
                let write_param = quote! {,#ident = #ident};
                (i, UnnamedEnumParamter::new(Some(letter), match_param, write_param))
            }
            false => {
                let match_param = if i < field_amount - 1 { quote! {_,} } else { quote! {_} };
                let write_param = quote! {};
                (i, UnnamedEnumParamter::new(None, match_param, write_param))
            }
        })
        .collect()
}

fn create_unnamed_match_arm(parameter_map: HashMap<usize, UnnamedEnumParamter>, ident: &Ident, variant: &Variant, fields: &FieldsUnnamed, mut message: String) -> TokenStream2 {
    let mut match_params = vec![];
    let mut write_params = vec![];

    for i in 0..fields.unnamed.len() {
        let parameter = parameter_map.get(&i).unwrap();
        if let Some(l) = parameter.letter {
            message = message.replace(&format!("{{{}}}", i), &format!("{{{}}}", l))
        }
        match_params.push(parameter.match_param.clone());
        write_params.push(parameter.write_param.clone());
    }

    let variant_ident = &variant.ident;
    quote! {
        #ident::#variant_ident(#(#match_params)*) => write!(f, #message #(#write_params)*),
    }
}

struct UnnamedEnumParamter {
    letter: Option<char>,
    match_param: TokenStream2,
    write_param: TokenStream2,
}

impl UnnamedEnumParamter {
    pub fn new(letter: Option<char>, match_param: TokenStream2, write_param: TokenStream2) -> Self {
        UnnamedEnumParamter { letter, match_param, write_param }
    }
}

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
        0 => tokens.push(quote! {..}),
        len if len < fields.named.len() => tokens.push(quote! {,..}),
        _ => ()
    };

    tokens
}

fn get_field_name(field: &Field) -> String {
    field.ident.as_ref().unwrap().to_string()
}