use syn::Variant;

use crate::parameters::{IMPL_FROM, MESSAGE, Parameters};

/// Central service to check if the attributes parameters are correctly used.
pub struct Validator {
    errors: Vec<ValidationError>,
}

impl Validator {
    const POSSIBLE_STRUCT_PARAMETERS: &'static [&'static str] = &[MESSAGE];
    const POSSIBLE_ENUM_PARAMETERS: &'static [&'static str] = &[IMPL_FROM, MESSAGE];

    pub fn new() -> Self {
        Validator {
            errors: vec![],
        }
    }

    /// Check if the given parameters from some struct are valid to use.
    /// Only checks the name, not the type or the value.
    fn check_struct_parameters(&mut self, parameters: &Parameters) {
        self.check_parameters(parameters, Self::POSSIBLE_STRUCT_PARAMETERS)
    }

    /// Check if the given parameters from some enum or enum variants are valid to use.
    /// Only checks the name, not the type or the value.
    fn check_enum_parameters(&mut self, parameters: &Parameters) {
        self.check_parameters(parameters, Self::POSSIBLE_ENUM_PARAMETERS)
    }

    /// Check if some given Parameters only contain keys which are possible to use.
    fn check_parameters(&mut self, parameters: &Parameters, possible_parameters: &'static [&'static str]) {
        self.errors.extend(parameters.name_iter()
            .flat_map(|name| match possible_parameters.contains(&name) {
                true => None,
                false => Some(ValidationError::UnknownParameter { param: name.to_string()})
            })
        )
    }

    pub fn check_impl_from(&mut self, global_impl_from: bool, variant_params: &Parameters, variant: &Variant) {
        if global_impl_from && variant_params.bool_for_name(IMPL_FROM) {
            self.errors.push(ValidationError::UnnecessaryImplFrom { variant_name: variant.ident.to_string() })
        }
    }
}

#[derive(Debug, Eq, PartialEq)]
pub enum ValidationError {
    UnknownParameter { param: String },
    UnnecessaryImplFrom { variant_name: String },
}