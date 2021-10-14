use syn::Path;

/// Convert a syn::Path to a name (as String)
pub fn path_to_name(path: &Path) -> String {
    path.get_ident().map(|ident| ident.to_string()).expect("The given path was not an identifier.")
}