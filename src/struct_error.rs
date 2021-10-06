use proc_macro::TokenStream;

use quote::{format_ident, quote};
use syn::{AttributeArgs, Fields, Fields::*, Generics, Ident, Index, ItemStruct, NestedMeta, parse_macro_input};
use syn::__private::TokenStream2;

pub fn implement(attr_args: AttributeArgs, item_struct: ItemStruct) -> TokenStream {
    let display_string_opt = get_display_string(attr_args);

    let ident = &item_struct.ident;
    let fields = &item_struct.fields;
    let generics = &item_struct.generics;
    let where_clause = &item_struct.generics.where_clause;

    let display_implementation = create_display_implementation(display_string_opt, ident, fields, generics);

    (quote! {
        #[derive(Debug)] #item_struct
        impl #generics std::error::Error for #ident #generics #where_clause {}
        #display_implementation
    }).into()
}

/// Return the display-string from the attribute parameters.
///
/// If no display string was provided, Display is not implemented for this Error. This is useful if you
/// need more complex logic to implement Display.
fn get_display_string(args: AttributeArgs) -> Option<String> {
    let arg_amount = args.len();
    if arg_amount > 1 { panic!("Too many arguments were provided to the 'error' attribute. Expected 0 or 1, got {}", arg_amount) }

    match arg_amount {
        0 => None,
        _ => Some(get_string_from_meta(args.first().unwrap()))
    }
}

fn get_string_from_meta(meta: &NestedMeta) -> String {
    match meta {
        syn::NestedMeta::Lit(literal) => match literal {
            syn::Lit::Str(literal_string) => literal_string.value(),
            _ => panic!("Argument of 'error' attribute must be a string literal!")
        },
        _ => panic!("Argument of 'error' attribute must be a string literal!")
    }
}

fn create_display_implementation(display_string_opt: Option<String>, ident: &Ident, fields: &Fields, generics: &Generics) -> TokenStream2 {
    let mut display_string = match display_string_opt {
        Some(string) => string,
        None => return quote! {}
    };

    let write_parameters = match fields {
        Named(_) => create_named_write_parameters(&display_string, fields),
        Unnamed(_) => create_positional_write_parameters(&mut display_string, fields),
        Unit => vec![]
    };

    create_implementation_with_write_parameters(display_string, ident, generics, write_parameters)
}

fn create_named_write_parameters(display_string: &String, fields: &Fields) -> Vec<TokenStream2> {
    let mut parameters = vec![];
    for field in fields {
        if let Some(ref i) = &field.ident {
            let field_name = i.to_string();
            if display_string.contains(&format!("{{{}}}", field_name)) {
                let id = format_ident!("{}", field_name);
                parameters.push(quote! {, #id = self.#id});
            }
        }
    }
    parameters
}

fn create_positional_write_parameters(display_string: &mut String, fields: &Fields) -> Vec<TokenStream2> {
    let mut parameters = vec![];
    let mut ignored_fields = 0;

    for i in 0..fields.len() {
        let string = format!("{{{}}}", i);

        if display_string.contains(&string) {
            *display_string = display_string.replace(&string, &format!("{{{}}}", i - ignored_fields));
            let index = Index::from(i);
            parameters.push(quote! {, self.#index});
        } else {
            ignored_fields += 1
        }
    }
    parameters
}

fn create_implementation_with_write_parameters(display_string: String, ident: &Ident, generics: &Generics, parameters: Vec<TokenStream2>) -> TokenStream2 {
    let where_clause = &generics.where_clause;
    quote! {
        impl #generics std::fmt::Display for #ident #generics #where_clause {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, #display_string #(#parameters)*)
            }
        }
    }
}