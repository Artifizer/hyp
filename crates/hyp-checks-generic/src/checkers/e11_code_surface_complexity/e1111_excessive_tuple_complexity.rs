//! E1111: Excessive tuple complexity
//!
//! Detects tuples with too many elements. Large tuples are hard to read and maintain.
//! Consider using a struct with named fields instead.

use crate::{checker::Checker, define_checker, violation::Violation};
use syn::{visit::Visit};

define_checker! {
    /// Checker for E1111: Excessive tuple complexity
    E1111ExcessiveTupleComplexity,
    code = "E1111",
    name = "Excessive tuple complexity",
    suggestions = "Replace large tuples with structs that have named fields for better readability",
    target_items = [Function, Struct, Enum, Trait, Impl],
    config_entry_name = "e1111_excessive_tuple_complexity",
    config = E1111Config {
        enabled: bool = true,
        severity: crate::config::SeverityLevel = crate::config::SeverityLevel::Low,
        categories: Vec<crate::config::CheckerCategory> = vec![crate::config::CheckerCategory::Complexity],
        /// Maximum number of tuple elements before flagging (default: 5)
        max_tuple_elements: usize = 5,
    },
    check_item(self, item, file_path) {
        let mut visitor = TupleVisitor {
            violations: Vec::new(),
            file_path,
            checker: self,
        };
        visitor.visit_item(item);
        Ok(visitor.violations)
    }
}

struct TupleVisitor<'a> {
    violations: Vec<Violation>,
    file_path: &'a str,
    checker: &'a E1111ExcessiveTupleComplexity,
}

impl<'a> Visit<'a> for TupleVisitor<'a> {
    fn visit_type(&mut self, node: &'a syn::Type) {
        if let syn::Type::Tuple(tuple) = node {
            let element_count = tuple.elems.len();

            // Flag tuples with more than max_tuple_elements
            if element_count > self.checker.config.max_tuple_elements {
                let span = tuple.paren_token.span.join();
                self.violations.push(
                    Violation::new(
                        self.checker.code(),
                        self.checker.name(),
                        self.checker.severity().into(),
                        format!(
                            "Tuple has {} elements (max: {}). Consider using a struct with named fields",
                            element_count,
                            self.checker.config.max_tuple_elements
                        ),
                        self.file_path,
                        span.start().line,
                        span.start().column + 1,
                    )
                    .with_suggestion(self.checker.suggestions()),
                );
            }
        }

        syn::visit::visit_type(self, node);
    }

    fn visit_expr(&mut self, node: &'a syn::Expr) {
        // Check tuple expressions (tuple literals)
        if let syn::Expr::Tuple(tuple) = node {
            let element_count = tuple.elems.len();

            // Flag tuples with more than max_tuple_elements
            // Skip empty tuples (unit type)
            if element_count > self.checker.config.max_tuple_elements {
                let span = tuple.paren_token.span.join();
                self.violations.push(
                    Violation::new(
                        self.checker.code(),
                        self.checker.name(),
                        self.checker.severity().into(),
                        format!(
                            "Tuple literal has {} elements (max: {}). Consider using a struct with named fields",
                            element_count,
                            self.checker.config.max_tuple_elements
                        ),
                        self.file_path,
                        span.start().line,
                        span.start().column + 1,
                    )
                    .with_suggestion(self.checker.suggestions()),
                );
            }
        }

        syn::visit::visit_expr(self, node);
    }

    fn visit_pat(&mut self, node: &'a syn::Pat) {
        // Check tuple patterns in destructuring
        if let syn::Pat::Tuple(tuple) = node {
            let element_count = tuple.elems.len();

            if element_count > self.checker.config.max_tuple_elements {
                let span = tuple.paren_token.span.join();
                self.violations.push(
                    Violation::new(
                        self.checker.code(),
                        self.checker.name(),
                        self.checker.severity().into(),
                        format!(
                            "Tuple pattern has {} elements (max: {}). Consider using a struct with named fields",
                            element_count,
                            self.checker.config.max_tuple_elements
                        ),
                        self.file_path,
                        span.start().line,
                        span.start().column + 1,
                    )
                    .with_suggestion(self.checker.suggestions()),
                );
            }
        }

        syn::visit::visit_pat(self, node);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn check_code(code: &str) -> Vec<Violation> {
        let checker = E1111ExcessiveTupleComplexity::default();
        let file = syn::parse_file(code).expect("Failed to parse");
        let mut violations = Vec::new();
        for item in &file.items {
            violations.extend(checker.check_item(item, "test.rs").unwrap());
        }
        violations
    }

    #[test]
    fn test_detects_large_tuple_type() {
        let code = r#"
            fn example() -> (i32, i32, i32, i32, i32, i32) {
                (1, 2, 3, 4, 5, 6)
            }
        "#;
        let violations = check_code(code);
        // Should detect both the return type and the tuple literal
        assert!(!violations.is_empty());
        assert!(violations.iter().any(|v| v.message.contains("Tuple")));
    }

    #[test]
    fn test_detects_large_tuple_literal() {
        let code = r#"
            fn example() {
                let data = (1, 2, 3, 4, 5, 6);
            }
        "#;
        let violations = check_code(code);
        assert!(!violations.is_empty());
        assert!(violations.iter().any(|v| v.message.contains("6 elements")));
    }

    #[test]
    fn test_detects_large_tuple_pattern() {
        let code = r#"
            fn example(data: (i32, i32, i32, i32, i32, i32)) {
                let (a, b, c, d, e, f) = data;
            }
        "#;
        let violations = check_code(code);
        // Should detect the parameter type and the pattern
        assert!(!violations.is_empty());
    }

    #[test]
    fn test_allows_small_tuples() {
        let code = r#"
            fn example() -> (i32, i32) {
                (1, 2)
            }
        "#;
        let violations = check_code(code);
        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_allows_triple() {
        let code = r#"
            fn example() -> (i32, i32, i32) {
                (1, 2, 3)
            }
        "#;
        let violations = check_code(code);
        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_allows_unit_type() {
        let code = r#"
            fn example() -> () {
                ()
            }
        "#;
        let violations = check_code(code);
        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_detects_in_struct_field() {
        let code = r#"
            struct Data {
                coords: (f64, f64, f64, f64, f64, f64),
            }
        "#;
        let violations = check_code(code);
        assert!(!violations.is_empty());
    }
}
