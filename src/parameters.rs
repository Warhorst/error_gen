use std::collections::HashMap;
use std::ops::Deref;

use syn::{Attribute, AttributeArgs, Lit, Meta, NestedMeta};
use syn::Lit::*;
use syn::Meta::*;

use crate::common::*;

pub const ERROR_ATTRIBUTE: &'static str = "error";
pub const MESSAGE: &'static str = "message";
pub const IMPL_FROM: &'static str = "impl_from";

/// Representation of attributes as key value pairs with names as key and primitives as values.
pub struct Parameters {
    values: HashMap<String, LitValue>,
}

impl Parameters {
    /// Create Parameters from an Attribute, like
    ///
    /// #[error(message = "AHHHH", impl_from)] <--
    /// struct AnError;
    ///
    /// (only list-like attributes like above are valid)
    pub fn from_attribute(attribute: &Attribute) -> Self {
        let meta = attribute.parse_meta().unwrap();

        match meta {
            syn::Meta::List(list) => Self::from_nested_metas(list.nested),
            _ => panic!(r#"Expected list-like attribute, like [error(param0 = "foo", param1 = false)]"#)
        }
    }

    /// Create Parameters from AttributeArgs, which are just the values of a list-like Attribute.
    /// Example
    ///
    /// #[error( <-- list-like Attribute
    ///     message = "AHHH", impl_from <-- AttributeArgs
    /// )]
    pub fn from_attribute_args(args: AttributeArgs) -> Self {
        Self::from_nested_metas(args)
    }

    fn from_nested_metas<I>(nested_metas: I) -> Self
        where I: IntoIterator<Item=NestedMeta> {
        let values = nested_metas
            .into_iter()
            .filter_map(|nested| match nested {
                syn::NestedMeta::Meta(meta) => Some(meta),
                syn::NestedMeta::Lit(_) => panic!("Unexpected literal in meta list")
            })
            .map(Self::meta_to_name_value)
            .collect();

        Parameters { values }
    }

    /// Parse meta to name value tuples. The value of a NameValue is the
    /// value of the corresponding literal. The value of a path is always true.
    /// Meta lists are ignored.
    fn meta_to_name_value(meta: Meta) -> (String, LitValue) {
        match meta {
            NameValue(name_value) => {
                let name = path_to_name(&name_value.path);
                let value = LitValue::from(&name_value.lit);
                (name, value)
            }
            Path(path) => {
                let name = path_to_name(&path);
                let value = LitValue::Boolean(true);
                (name, value)
            }
            List(_) => panic!("Unexpected meta list")
        }
    }

    /// Not setting this value and setting it to 'false' means the same, so
    /// returning an Option<bool> is pointless here
    pub fn bool_for_name(&self, name: &str) -> bool {
        self.values.get(name).map(LitValue::bool_value).unwrap_or(false)
    }

    pub fn string_for_name(&self, name: &str) -> Option<String> {
        self.values.get(name).map(LitValue::string_value)
    }

    /// Return how many parameters are set.
    pub fn size(&self) -> usize {
        self.values.len()
    }

    /// Check if these parameters have a parameter with the given name.
    pub fn has_parameter(&self, param: &str) -> bool {
        self.values.contains_key(param)
    }

    /// Return an iterator over the names of this Parameters.
    pub fn name_iter(&self) -> ParameterIter {
        self.into_iter()
    }
}

pub struct ParameterIter<'a> {
    index: usize,
    keys: Vec<&'a str>,
}

impl<'a> ParameterIter<'a> {
    fn new(parameters: &'a Parameters) -> Self {
        ParameterIter {
            index: 0,
            keys: parameters.values.keys().map(|key| key.deref()).collect(),
        }
    }
}

impl<'a> Iterator for ParameterIter<'a> {
    type Item = &'a str;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index == self.keys.len() { return None; }

        let result = self.keys[self.index];
        self.index += 1;
        Some(result)
    }
}

impl<'a> IntoIterator for &'a Parameters {
    type Item = &'a str;
    type IntoIter = ParameterIter<'a>;

    fn into_iter(self) -> Self::IntoIter {
        ParameterIter::new(self)
    }
}

/// syn::Lit describes a literal from a token stream.
/// This is not very handy to use, for example when creating a literal value like 'true'.
/// This LitValue enum fixes this issue by ignoring the token stream part and only wrapping the literal value.
enum LitValue {
    String(String),
    Boolean(bool),
}

impl LitValue {
    pub fn bool_value(&self) -> bool {
        if let LitValue::Boolean(b) = self {
            return *b;
        }
        panic!("Expected boolean")
    }

    pub fn string_value(&self) -> String {
        if let LitValue::String(s) = self {
            return s.clone();
        }
        panic!("Expected string")
    }
}

impl From<&Lit> for LitValue {
    fn from(lit: &syn::Lit) -> Self {
        match lit {
            Str(lit_str) => LitValue::String(lit_str.value()),
            Bool(lit_bool) => LitValue::Boolean(lit_bool.value),
            _ => panic!("Unexpected literal value")
        }
    }
}

#[cfg(test)]
mod parameters_tests {
    use std::collections::HashMap;

    use syn::Attribute;

    use crate::parameters::{LitValue, Parameters};

    #[test]
    fn from_attribute_works() {
        let attribute: Attribute = syn::parse_quote!(#[foo(bar = true)]);
        let parameters = Parameters::from_attribute(&attribute);
        assert_eq!(parameters.size(), 1);
        assert_eq!(parameters.has_parameter("bar"), true);
        assert_eq!(parameters.bool_for_name("bar"), true);
    }

    #[test]
    #[should_panic]
    fn from_attribute_path_like_fails() {
        let attribute: Attribute = syn::parse_quote!(#[foo]);
        Parameters::from_attribute(&attribute);
    }

    #[test]
    #[should_panic]
    fn from_attribute_key_value_like_fails() {
        let attribute: Attribute = syn::parse_quote!(#[foo = true]);
        Parameters::from_attribute(&attribute);
    }

    #[test]
    fn size_works() {
        let parameters = create_example_parameters();
        assert_eq!(parameters.size(), 2)
    }

    #[test]
    fn has_parameter_works() {
        let parameters = create_example_parameters();
        assert_eq!(parameters.has_parameter("foo"), true);
        assert_eq!(parameters.has_parameter("baz"), true);
        assert_eq!(parameters.has_parameter("oof"), false);
    }

    #[test]
    fn name_iter_works() {
        let parameters = create_example_parameters();
        let names = parameters.name_iter().collect::<Vec<_>>();
        assert_eq!(names.len(), parameters.size());
        assert!(names.contains(&"foo"));
        assert!(names.contains(&"baz"));
    }

    fn create_example_parameters() -> Parameters {
        let mut values = HashMap::new();
        values.insert("foo".to_string(), LitValue::String("bar".to_string()));
        values.insert("baz".to_string(), LitValue::Boolean(true));
        Parameters { values }
    }
}