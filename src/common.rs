use syn::Path;

/// Convert a syn::Path to a name (as String)
pub fn path_to_name(path: &Path) -> String {
    path.get_ident().map(|ident| ident.to_string()).expect("The given path was not an identifier.")
}

/// To assert an implementor creates the expected result, we transform it into a string and compare
/// it which an expectation (generally another string). This function checks if they are equal.
#[cfg(test)]
pub fn assert_tokens_are_equal<L, R>(left: L, right: R) where L: AsRef<str>, R: AsRef<str> {
    assert_eq!(remove_whitespace(left.as_ref()), remove_whitespace(right.as_ref()))
}

#[cfg(test)]
fn remove_whitespace(string: &str) -> String {
    string.chars().filter(|c| !c.is_whitespace()).collect()
}