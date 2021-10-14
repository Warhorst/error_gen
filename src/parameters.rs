use syn::{Lit, NestedMeta, Meta, Attribute, AttributeArgs};
use syn::Lit::*;
use std::collections::HashMap;
use crate::common::*;

/// Representation of attributes as key value pairs with names as key and primitives as values.
pub struct Parameters {
    values: HashMap<String, LitValue>,
}

impl Parameters {
    pub fn from_attribute(attribute: Attribute) -> Self {
        let meta = attribute.parse_meta().unwrap();

        match meta {
            syn::Meta::List(list) => Self::from_nested_metas(list.nested),
            _ => panic!("Expected meta list for attribute 'error'")
        }
    }

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
            .collect::<HashMap<_, _>>();

        Parameters { values }
    }

    /// Parse meta to name value tuples. The value of a NameValue is the
    /// value of the corresponding literal. The value of a path is always true.
    /// Meta lists are ignored.
    fn meta_to_name_value(meta: Meta) -> (String, LitValue) {
        match meta {
            syn::Meta::NameValue(name_value) => {
                let name = path_to_name(&name_value.path);
                let value = LitValue::from(&name_value.lit);
                (name, value)
            }
            syn::Meta::Path(path) => {
                let name = path_to_name(&path);
                let value = LitValue::Boolean(true);
                (name, value)
            }
            syn::Meta::List(_) => panic!("Unexpected meta list")
        }
    }

    pub fn bool_for_name(&self, name: &str) -> Option<bool> {
        self.values.get(name).map(LitValue::bool_value)
    }

    pub fn string_for_name(&self, name: &str) -> Option<String> {
        self.values.get(name).map(LitValue::string_value)
    }

    pub fn is_empty(&self) -> bool {
        self.values.is_empty()
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