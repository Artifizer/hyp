//! E1015: Unwrap/expect without context
//!
//! Detects usage of `.unwrap()` and `.expect()` with poor or no error messages.
//! These methods crash the program without providing useful debugging context.

use crate::{checker::Checker, define_checker, violation::Violation};

use syn::visit::Visit;

define_checker! {
    /// Checker for E1002: Unwrap/expect without context
    E1015UnwrapExpect,
    code = "E1015",
    name = "Unwrap/expect without context",
    suggestions = "Use pattern matching, if let, or the ? operator instead. If unwrap is necessary, use expect() with a descriptive message.",
    target_items = [Function],
    config_entry_name = "e1015_unwrap_expect",
    /// Configuration for E1015: Unwrap/expect checker
    config = E1015Config {
        /// Whether this checker is enabled
        enabled: bool = true,
        /// Severity level, defaults to High
        severity: crate::config::SeverityLevel = crate::config::SeverityLevel::High,
        /// Categories this checker belongs to
        categories: Vec<crate::config::CheckerCategory> = vec![crate::config::CheckerCategory::Operations],
        /// Minimum length for expect messages to be considered descriptive
        min_expect_message_length: usize = 10,
    },
    // AST node item checker
    check_item(self, item, file_path) {
        let mut visitor = UnwrapExpectVisitor {
            violations: Vec::new(),
            file_path,
            checker: self,
        };
        visitor.visit_item(item);
        Ok(visitor.violations)
    }
}

/// Visitor that traverses the AST looking for unwrap/expect calls
struct UnwrapExpectVisitor<'a> {
    violations: Vec<Violation>,
    file_path: &'a str,
    checker: &'a E1015UnwrapExpect,
}

impl<'a> UnwrapExpectVisitor<'a> {
    fn create_violation(&self, span: proc_macro2::Span, message: &str) -> Violation {
        let start = span.start();
        Violation::new(
            self.checker.code(),
            self.checker.name(),
            self.checker.severity().into(),
            message,
            self.file_path,
            start.line,
            start.column + 1,
        )
        .with_suggestion(self.checker.suggestions())
    }
}

impl<'a> Visit<'a> for UnwrapExpectVisitor<'a> {
    fn visit_expr_method_call(&mut self, node: &'a syn::ExprMethodCall) {
        let method_name = node.method.to_string();

        match method_name.as_str() {
            "unwrap" => {
                self.violations.push(self.create_violation(
                    node.method.span(),
                    "Using unwrap() will panic on None/Err without any context. Use expect() with a descriptive message or proper error handling.",
                ));
            }
            "expect" => {
                // Check if the expect message is descriptive enough
                if let Some(syn::Expr::Lit(syn::ExprLit {
                    lit: syn::Lit::Str(s),
                    ..
                })) = node.args.first()
                {
                    let msg = s.value();
                    if msg.len() < self.checker.config.min_expect_message_length {
                        self.violations.push(self.create_violation(
                            node.method.span(),
                            &format!(
                                "expect() message '{}' is too short. Provide a descriptive message explaining what was expected.",
                                msg
                            ),
                        ));
                    }
                }
            }
            _ => {}
        }

        // Continue visiting nested expressions
        syn::visit::visit_expr_method_call(self, node);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detects_unwrap() {
        let code = r#"
            fn example() {
                let data = Some(42);
                let _value = data.unwrap();
            }
        "#;

        let syntax = syn::parse_file(code).unwrap();
        let checker = E1015UnwrapExpect::default();

        let mut violations = Vec::new();
        for item in &syntax.items {
            violations.extend(checker.check_item(item, "test.rs").unwrap());
        }

        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].code, "E1015");
        assert!(violations[0].message.contains("unwrap()"));
    }

    #[test]
    fn test_detects_short_expect_message() {
        let code = r#"
            fn example() {
                let data = Some(42);
                let _value = data.expect("failed");
            }
        "#;

        let syntax = syn::parse_file(code).unwrap();
        let checker = E1015UnwrapExpect::default();

        let mut violations = Vec::new();
        for item in &syntax.items {
            violations.extend(checker.check_item(item, "test.rs").unwrap());
        }

        assert_eq!(violations.len(), 1);
        assert!(violations[0].message.contains("too short"));
    }

    #[test]
    fn test_allows_descriptive_expect() {
        let code = r#"
            fn example() {
                let data = Some(42);
                let _value = data.expect("Configuration file must exist and be readable");
            }
        "#;

        let syntax = syn::parse_file(code).unwrap();
        let checker = E1015UnwrapExpect::default();

        let mut violations = Vec::new();
        for item in &syntax.items {
            violations.extend(checker.check_item(item, "test.rs").unwrap());
        }

        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_detects_chained_unwrap() {
        let code = r#"
            fn example() {
                let data = Some(Some(42));
                let _value = data.unwrap().unwrap();
            }
        "#;

        let syntax = syn::parse_file(code).unwrap();
        let checker = E1015UnwrapExpect::default();

        let mut violations = Vec::new();
        for item in &syntax.items {
            violations.extend(checker.check_item(item, "test.rs").unwrap());
        }

        assert_eq!(violations.len(), 2);
    }
}
