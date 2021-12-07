use std::collections::HashMap;

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
    pub fn from_attribute(attribute: &Attribute) -> Self {
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