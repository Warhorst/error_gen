use quote::quote;
use syn::__private::TokenStream2;

pub struct WriteImplementor {
    current_expression: Option<String>,
    expressions: Vec<String>,
    current_depth: usize,
}

impl WriteImplementor {
    pub fn new() -> Self {
        WriteImplementor {
            current_expression: None,
            expressions: vec![],
            current_depth: 0,
        }
    }

    /// Creates an implementation of a write! macro call for a given message.
    /// The message can contain expressions in braces, which will be used
    /// to fill these spaces. Example:
    ///
    /// "This is my value: {e.critical_string_value()}"
    /// will become
    /// write!{f, "This is my value {}", e.critical_string_value()}
    pub fn implement(mut self, message: String) -> TokenStream2 {
        let new_message: String = message
            .chars()
            .filter(|c| match c {
                '{' => self.handle_opening_parenthesis(*c),
                '}' => self.handle_closing_parenthesis(*c),
                c if self.currently_creates_expression() => {
                    self.append_to_expression(*c);
                    false
                },
                _ => true
            })
            .collect();


        self.create_write_implementation(new_message)
    }

    fn handle_opening_parenthesis(&mut self, parenthesis: char) -> bool {
        match self.currently_creates_expression() {
            true => {
                self.append_to_expression(parenthesis);
                self.increase_depth();
                false
            },
            false => {
                self.start_creation();
                true
            }
        }
    }

    fn handle_closing_parenthesis(&mut self, parenthesis: char) -> bool {
        match self.currently_creates_expression() {
            true => match self.current_depth == 1 {
                true => {
                    self.end_creation();
                    true
                },
                false => {
                    self.append_to_expression(parenthesis);
                    self.decrease_depth();
                    false
                }
            },
            false => true
        }
    }

    fn currently_creates_expression(&self) -> bool {
        self.current_expression.is_some()
    }

    fn append_to_expression(&mut self, c: char) {
        if let Some(e) = self.current_expression.as_mut() {
            e.push(c)
        }
    }

    fn increase_depth(&mut self) {
        self.current_depth += 1
    }

    fn decrease_depth(&mut self) {
        self.current_depth -= 1
    }

    fn start_creation(&mut self) {
        self.current_expression = Some(String::new());
        self.increase_depth()
    }

    fn end_creation(&mut self) {
        self.expressions.push(self.current_expression.as_mut().unwrap().clone());
        self.current_expression = None;
        self.decrease_depth()
    }

    fn create_write_implementation(self, message: String) -> TokenStream2 {
        let expressions: TokenStream2 = self.expressions
            .into_iter()
            .map(|e| e.parse::<TokenStream2>().unwrap())
            .map(|ts| quote! {,#ts})
            .collect();

        quote! {write!(f, #message #expressions)}
    }
}

#[cfg(test)]
mod tests {
    use crate::common::assert_tokens_are_equal;
    use crate::impl_display::write::WriteImplementor;

    #[test]
    fn implement_works() {
        let message = "some complex stuff: {e.foo()}, {if b {42} else {43}}".to_string();
        let ts = WriteImplementor::new().implement(message).to_string();
        let expected = r#"write!(f, "some complex stuff: {}, {}", e.foo(), if b {42} else {43})"#;
        assert_tokens_are_equal(ts, expected)
    }

    #[test]
    fn implement_multiple_expressions_works() {
        let message = "complex: {{let mut i = 0; i += 1; i}}".to_string();
        let ts = WriteImplementor::new().implement(message).to_string();
        let expected = r#"write!(f, "complex: {}", {let mut i = 0; i += 1; i})"#;
        assert_tokens_are_equal(ts, expected)
    }
}