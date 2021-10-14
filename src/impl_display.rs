use syn::{ItemEnum, Variant, FieldsNamed, FieldsUnnamed, Field, ItemStruct, Index};
use syn::__private::TokenStream2;
use quote::{quote, format_ident};
use syn::__private::quote::__private::Ident;
use syn::Fields::*;
use std::collections::HashMap;

/// Holds the necessary information to generate a std::fmt::Display implementation for an struct.
pub struct DisplayDataStruct<'a> {
    item_struct: &'a ItemStruct,
    message_opt: Option<String>
}

impl<'a> DisplayDataStruct<'a> {
    pub fn new(item_struct: &'a ItemStruct, message_opt: Option<String>) -> Self {
        DisplayDataStruct { item_struct, message_opt }
    }

    pub fn to_display_implementation(self) -> TokenStream2 {
        let mut message = match self.message_opt.clone() {
            Some(string) => string,
            None => return quote! {}
        };

        match &self.item_struct.fields {
            Named(fields) => {
                let params = self.create_named_write_parameters(&message, &fields);
                self.create_implementation_with_write_parameters(&message, params)
            },
            Unnamed(fields) => {
                let params = self.create_positional_write_parameters(&mut message, &fields);
                self.create_implementation_with_write_parameters(&message, params)
            },
            Unit => self.create_implementation_with_write_parameters(&message, vec![])
        }
    }

    pub fn create_named_write_parameters(&self, message: &String, fields: &FieldsNamed) -> Vec<TokenStream2> {
        get_used_identifiers_in_string(message, fields)
            .into_iter()
            .map(|ident| quote! {, #ident = self.#ident})
            .collect()
    }

    fn create_positional_write_parameters(&self, message: &mut String, fields: &FieldsUnnamed) -> Vec<TokenStream2> {
        let mut parameters = vec![];
        let mut ignored_fields = 0;

        for i in 0..fields.unnamed.len() {
            let string = format!("{{{}}}", i);

            if message.contains(&string) {
                *message = message.replace(&string, &format!("{{{}}}", i - ignored_fields));
                let index = Index::from(i);
                parameters.push(quote! {, self.#index});
            } else {
                ignored_fields += 1
            }
        }
        parameters
    }

    fn create_implementation_with_write_parameters(&self, message: &String, parameters: Vec<TokenStream2>) -> TokenStream2 {
        let ident = &self.item_struct.ident;
        let generics = &self.item_struct.generics;
        let where_clause = &generics.where_clause;
        quote! {
            impl #generics std::fmt::Display for #ident #generics #where_clause {
                fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                    write!(f, #message #(#parameters)*)
                }
            }
        }
    }
}

/// Holds the necessary information to generate a std::fmt::Display implementation for an enum.
pub struct DisplayDataEnum<'a> {
    item_enum: &'a ItemEnum,
    default_message: Option<String>,
    match_arm_data: Vec<MatchArmData<'a>>,
}

impl<'a> DisplayDataEnum<'a> {
    pub fn new_empty(item_enum: &'a ItemEnum, default_message: Option<String>) -> Self {
        DisplayDataEnum {
            item_enum,
            default_message,
            match_arm_data: vec![],
        }
    }

    pub fn add_match_arm_data(&mut self, message: String, variant: &'a Variant) {
        self.match_arm_data.push(MatchArmData::new(message, &self.item_enum.ident, variant))
    }

    pub fn to_display_implementation(self) -> TokenStream2 {
        if self.default_message_required() {
            panic!("Not all enum variants have a display message. Provide a default message at the enum definition.")
        }

        let match_arms: Vec<TokenStream2> = self.match_arm_data
            .into_iter()
            .map(MatchArmData::to_match_arm)
            .collect();

        let ident = &self.item_enum.ident;
        let generics = &self.item_enum.generics;
        let where_clause = &generics.where_clause;
        let default = match match_arms.len() == self.item_enum.variants.len() {
            true => quote! {},
            false => match self.default_message {
                Some(m) => quote! {_ => write!(f, #m)},
                None => panic!("Not all enum variants have a display message. Provide a default message at the enum definition.")
            }
        };

        quote! {
            impl #generics std::fmt::Display for #ident #generics #where_clause {
                fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                    match self {
                        #(#match_arms)*
                        #default
                    }
                }
            }
        }
    }

    fn default_message_required(&self) -> bool {
        self.match_arm_data.len() != self.item_enum.variants.len() && self.default_message.is_none()
    }
}

struct MatchArmData<'a> {
    message: String,
    enum_ident: &'a Ident,
    variant: &'a Variant,
}

impl<'a> MatchArmData<'a> {
    pub fn new(message: String, enum_ident: &'a Ident, variant: &'a Variant) -> Self {
        MatchArmData { message, enum_ident, variant }
    }

    fn to_match_arm(self) -> TokenStream2 {
        match &self.variant.fields {
            Named(fields) => MatchArmDataNamed::new(self.message, self.enum_ident, &self.variant.ident, fields).to_match_arm(),
            Unnamed(fields) => MatchArmDataUnnamed::new(self.message, self.enum_ident, &self.variant.ident, fields).to_match_arm(),
            Unit => MatchArmDataUnit::new(self.message, self.enum_ident, &self.variant.ident).to_match_arm()
        }
    }
}

struct MatchArmDataNamed<'a> {
    message: String,
    enum_ident: &'a Ident,
    variant_ident: &'a Ident,
    fields: &'a FieldsNamed,
}

impl<'a> MatchArmDataNamed<'a> {
    pub fn new(message: String, enum_ident: &'a Ident, variant_ident: &'a Ident, fields: &'a FieldsNamed) -> Self {
        MatchArmDataNamed { message, enum_ident, variant_ident, fields }
    }

    pub fn to_match_arm(self) -> TokenStream2 {
        let enum_ident = &self.enum_ident;
        let variant_ident = &self.variant_ident;
        let message = &self.message;
        let match_patterns = self.create_match_patterns();
        let write_parameters = self.create_write_parameters();

        quote! {
            #enum_ident::#variant_ident{#(#match_patterns)*} => write!(f, #message #(#write_parameters)*),
        }
    }

    fn create_match_patterns(&self) -> Vec<TokenStream2> {
        let identifiers = get_used_identifiers_in_string(&self.message, self.fields);
        let field_amount = self.fields.named.len();

        let mut tokens = identifiers.into_iter()
            .enumerate()
            .map(|(i, ident)| match i {
                0 => quote! {#ident},
                _ => quote! {, #ident}
            })
            .collect::<Vec<_>>();

        match tokens.len() {
            0 => vec![quote! {..}],
            len if len == field_amount => tokens,
            _ => {
                tokens.extend(vec![quote! {, ..}]);
                tokens
            }
        }
    }

    fn create_write_parameters(&self) -> Vec<TokenStream2> {
        get_used_identifiers_in_string(&self.message, self.fields)
            .into_iter()
            .map(|ident| quote! {, #ident = #ident})
            .collect()
    }
}

struct MatchArmDataUnnamed<'a> {
    message: String,
    enum_ident: &'a Ident,
    variant_ident: &'a Ident,
    fields: &'a FieldsUnnamed,
}

impl<'a> MatchArmDataUnnamed<'a> {
    pub fn new(message: String, enum_ident: &'a Ident, variant_ident: &'a Ident, fields: &'a FieldsUnnamed) -> Self {
        MatchArmDataUnnamed { message, enum_ident, variant_ident, fields }
    }

    pub fn to_match_arm(mut self) -> TokenStream2 {
        let parameter_map = self.get_unnamed_parameters_in_message();
        let mut match_params = vec![];
        let mut write_params = vec![];

        for i in 0..self.fields.unnamed.len() {
            let parameter = parameter_map.get(&i).unwrap();
            if let Some(l) = parameter.letter {
                self.message = self.message.replace(&format!("{{{}}}", i), &format!("{{{}}}", l))
            }
            match_params.push(parameter.pattern.clone());
            write_params.push(parameter.write_param.clone());
        }

        let enum_ident = &self.enum_ident;
        let variant_ident = &self.variant_ident;
        let message = self.message;
        quote! {
            #enum_ident::#variant_ident(#(#match_params)*) => write!(f, #message #(#write_params)*),
        }
    }

    fn get_unnamed_parameters_in_message(&self) -> HashMap<usize, UnnamedEnumParamter> {
        let field_amount = self.fields.unnamed.len();
        (0..field_amount).into_iter()
            .map(|i| match self.message.contains(&format!("{{{}}}", i)) {
                true => {
                    let letter = char::from_u32((i as u32) + 97).unwrap();
                    let ident = format_ident!("{}", letter);
                    let pattern = if i < field_amount - 1 { quote! {#ident,} } else { quote! {#ident} };
                    let write_param = quote! {,#ident = #ident};
                    (i, UnnamedEnumParamter::new(Some(letter), pattern, write_param))
                }
                false => {
                    let pattern = if i < field_amount - 1 { quote! {_,} } else { quote! {_} };
                    let write_param = quote! {};
                    (i, UnnamedEnumParamter::new(None, pattern, write_param))
                }
            })
            .collect()
    }
}

struct UnnamedEnumParamter {
    letter: Option<char>,
    pattern: TokenStream2,
    write_param: TokenStream2,
}

impl UnnamedEnumParamter {
    pub fn new(letter: Option<char>, pattern: TokenStream2, write_param: TokenStream2) -> Self {
        UnnamedEnumParamter { letter, pattern, write_param }
    }
}

struct MatchArmDataUnit<'a> {
    message: String,
    enum_ident: &'a Ident,
    variant_ident: &'a Ident,
}

impl<'a> MatchArmDataUnit<'a> {
    pub fn new(message: String, enum_ident: &'a Ident, variant_ident: &'a Ident) -> Self {
        MatchArmDataUnit { message, enum_ident, variant_ident }
    }

    pub fn to_match_arm(self) -> TokenStream2 {
        let enum_ident = &self.enum_ident;
        let variant_ident = &self.variant_ident;
        let message = &self.message;
        quote! {
            #enum_ident::#variant_ident => write!(f, #message),
        }
    }
}

pub fn get_used_identifiers_in_string<'a>(display_string: &String, fields: &'a FieldsNamed) -> Vec<&'a Ident> {
    fields.named.iter()
        .map(|field| {
            let ident = get_field_ident(field);
            if display_string.contains(&format!("{{{}}}", ident.to_string())) {
                return Some(ident);
            }
            None
        })
        .flat_map(Option::into_iter)
        .collect()
}

fn get_field_ident(field: &Field) -> &Ident {
    field.ident.as_ref().unwrap()
}