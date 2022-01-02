use quote::quote;
use syn::__private::TokenStream2;

pub struct WriteImplementor {
    message: String,
    current_expression: Option<String>,
    expressions: Vec<String>,
    current_depth: usize,
}

impl WriteImplementor {
    pub fn new(message: String) -> Self {
        WriteImplementor {
            message,
            current_expression: None,
            expressions: vec![],
            current_depth: 0,
        }
    }

    pub fn implement(mut self) -> TokenStream2 {
        let message_clone = self.message.clone();

        let new_message: String = message_clone
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
    use crate::impl_display_new::write_implementor::WriteImplementor;

    #[test]
    fn implement_works() {
        let message = "some complex stuff: {e.foo()}, {if b {42} else {43}}".to_string();
        let tokens = WriteImplementor::new(message).implement();

        let expected = r#"write!(f, "some complex stuff: {}, {}", e.foo(), if b {42} else {43})"#.to_string();
        assert_eq!(remove_whitespace(expected), remove_whitespace(tokens.to_string()))
    }

    fn remove_whitespace(s: String) -> String {
        s.chars().filter(|c| !c.is_whitespace()).collect()
    }
}