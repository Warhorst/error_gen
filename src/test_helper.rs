use syn::{Attribute, AttributeArgs};

/// To assert an implementor creates the expected result, we transform it into a string and compare
/// it which an expectation (generally another string). This function checks if they are equal.
pub fn assert_tokens_are_equal<L, R>(left: L, right: R) where L: AsRef<str>, R: AsRef<str> {
    assert_eq!(remove_whitespace(left.as_ref()), remove_whitespace(right.as_ref()))
}

fn remove_whitespace(string: &str) -> String {
    string.chars().filter(|c| !c.is_whitespace()).collect()
}

/// Because AttributeArgs are just a Vec of nested Meta,
/// we cannot parse it with parse_quote!. Instead,
/// we use this to extract the args from a whole attribute.
pub fn extract_attribute_args(attr: Attribute) -> AttributeArgs {
    match attr.parse_meta().unwrap() {
        syn::Meta::List(list) => list.nested.into_iter().collect(),
        _ => vec![]
    }
}