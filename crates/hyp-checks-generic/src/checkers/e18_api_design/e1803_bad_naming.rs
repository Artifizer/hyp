//! E1803: Bad naming
//!
//! Detects naming convention violations such as non-snake_case functions,
//! non-CamelCase types, or overly short/cryptic names.

use crate::{define_checker, violation::Violation};

define_checker! {
    /// Checker for E1803: Bad naming
    E1803BadNaming,
    code = "E1803",
    name = "Bad naming",
    suggestions = "Use snake_case for functions/variables, CamelCase for types, SCREAMING_SNAKE_CASE for constants",
    target_items = [Function, Struct, Enum, Const],
    config_entry_name = "e1803_bad_naming",
    config = E1803Config {
        enabled: bool = true,
        severity: crate::config::SeverityLevel = crate::config::SeverityLevel::Low,
        categories: Vec<crate::config::CheckerCategory> = vec![crate::config::CheckerCategory::Complexity],
        /// Minimum length for function names
        min_function_name_length: usize = 2,
    },
    check_item(self, item, file_path) {
        let mut violations = Vec::new();

        match item {
            syn::Item::Fn(func) => {
                let name = func.sig.ident.to_string();

                // Check for too short names (except common ones like 'new')
                if name.len() < self.config.min_function_name_length
                    && !matches!(name.as_str(), "new" | "as" | "to" | "is" | "id")
                {
                    let start = func.sig.ident.span().start();
                    violations.push(
                        Violation::new(
                            self.code(),
                            self.name(),
                            self.severity().into(),
                            format!("Function name '{}' is too short. Use descriptive names.", name),
                            file_path,
                            start.line,
                            start.column + 1,
                        )
                        .with_suggestion(self.suggestions()),
                    );
                }

                // Check for non-snake_case
                if !is_snake_case(&name) && !name.starts_with('_') {
                    let start = func.sig.ident.span().start();
                    violations.push(
                        Violation::new(
                            self.code(),
                            self.name(),
                            self.severity().into(),
                            format!("Function '{}' should use snake_case naming.", name),
                            file_path,
                            start.line,
                            start.column + 1,
                        )
                        .with_suggestion(self.suggestions()),
                    );
                }
            }
            syn::Item::Struct(s) => {
                let name = s.ident.to_string();
                if !is_camel_case(&name) {
                    let start = s.ident.span().start();
                    violations.push(
                        Violation::new(
                            self.code(),
                            self.name(),
                            self.severity().into(),
                            format!("Struct '{}' should use CamelCase naming.", name),
                            file_path,
                            start.line,
                            start.column + 1,
                        )
                        .with_suggestion(self.suggestions()),
                    );
                }
            }
            syn::Item::Enum(e) => {
                let name = e.ident.to_string();
                if !is_camel_case(&name) {
                    let start = e.ident.span().start();
                    violations.push(
                        Violation::new(
                            self.code(),
                            self.name(),
                            self.severity().into(),
                            format!("Enum '{}' should use CamelCase naming.", name),
                            file_path,
                            start.line,
                            start.column + 1,
                        )
                        .with_suggestion(self.suggestions()),
                    );
                }
            }
            syn::Item::Const(c) => {
                let name = c.ident.to_string();
                if !is_screaming_snake_case(&name) && !name.starts_with('_') {
                    let start = c.ident.span().start();
                    violations.push(
                        Violation::new(
                            self.code(),
                            self.name(),
                            self.severity().into(),
                            format!("Constant '{}' should use SCREAMING_SNAKE_CASE naming.", name),
                            file_path,
                            start.line,
                            start.column + 1,
                        )
                        .with_suggestion(self.suggestions()),
                    );
                }
            }
            _ => {}
        }

        Ok(violations)
    }
}

fn is_snake_case(s: &str) -> bool {
    if s.is_empty() {
        return false;
    }
    s.chars().all(|c| c.is_lowercase() || c.is_ascii_digit() || c == '_')
}

fn is_camel_case(s: &str) -> bool {
    let mut chars = s.chars();
    match chars.next() {
        Some(first) => first.is_uppercase() && !s.contains('_'),
        None => false,
    }
}

fn is_screaming_snake_case(s: &str) -> bool {
    if s.is_empty() {
        return false;
    }
    s.chars().all(|c| c.is_uppercase() || c.is_ascii_digit() || c == '_')
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::checker::Checker;

    #[test]
    fn test_detects_non_snake_case_function() {
        let code = r#"
            fn calculateTotal() {}
        "#;

        let syntax = syn::parse_file(code).unwrap();
        let checker = E1803BadNaming::default();

        let mut violations = Vec::new();
        for item in &syntax.items {
            violations.extend(checker.check_item(item, "test.rs").unwrap());
        }

        assert_eq!(violations.len(), 1);
    }

    #[test]
    fn test_snake_case_passes() {
        let code = r#"
            fn calculate_total() {}
        "#;

        let syntax = syn::parse_file(code).unwrap();
        let checker = E1803BadNaming::default();

        let mut violations = Vec::new();
        for item in &syntax.items {
            violations.extend(checker.check_item(item, "test.rs").unwrap());
        }

        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_detects_non_screaming_const() {
        let code = r#"
            const maxValue: i32 = 100;
        "#;

        let syntax = syn::parse_file(code).unwrap();
        let checker = E1803BadNaming::default();

        let mut violations = Vec::new();
        for item in &syntax.items {
            violations.extend(checker.check_item(item, "test.rs").unwrap());
        }

        assert_eq!(violations.len(), 1);
    }
}
