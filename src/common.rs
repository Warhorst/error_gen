use syn::{Path, Attribute};

const ERROR_ATTRIBUTE: &'static str = "error";

/// Convert a syn::Path to a name (as String)
pub fn path_to_name(path: &Path) -> String {
    path.get_ident().map(|ident| ident.to_string()).expect("The given path was not an identifier.")
}

pub fn attribute_is_error(attribute: &Attribute) -> bool {
    path_to_name(&attribute.path) == ERROR_ATTRIBUTE
}