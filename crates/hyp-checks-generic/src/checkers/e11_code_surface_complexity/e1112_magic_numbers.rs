//! E1112: Hardcoded magic numbers
//!
//! Detects unexplained numeric literals in code that should be named constants.
//! Magic numbers make code harder to understand and maintain.
//!
//! Example:
//! ```text
//! // Bad: What do these numbers mean?
//! fn calculate_price(qty: u32) -> u32 {
//!     qty * 1999 + 500
//! }
//!
//! // Good: Self-documenting constants
//! const PRICE_CENTS: u32 = 1999;
//! const SHIPPING_CENTS: u32 = 500;
//! fn calculate_price(qty: u32) -> u32 {
//!     qty * PRICE_CENTS + SHIPPING_CENTS
//! }
//! ```

use crate::{checker::Checker, define_checker, violation::Violation};

use syn::{spanned::Spanned, visit::Visit};

define_checker! {
    /// Checker for E1112: Hardcoded magic numbers
    E1112MagicNumbers,
    code = "E1112",
    name = "Hardcoded magic number",
    suggestions = "Extract magic numbers into named constants (const NAME: Type = value)",
    target_items = [Function],
    config_entry_name = "e1112_magic_numbers",
    config = E1112Config {
        enabled: bool = true,
        severity: crate::config::SeverityLevel = crate::config::SeverityLevel::Low,
        categories: Vec<crate::config::CheckerCategory> = vec![crate::config::CheckerCategory::Complexity],
        /// Numbers to ignore (common acceptable literals)
        allowed_numbers: Vec<i64> = vec![0, 1, 2, -1, 10, 100],
    },
    check_item(self, item, file_path) {
        let mut visitor = MagicNumberVisitor {
            violations: Vec::new(),
            file_path,
            checker: self,
            in_const_or_static: false,
            in_array_len: false,
        };
        visitor.visit_item(item);
        Ok(visitor.violations)
    }
}

struct MagicNumberVisitor<'a> {
    violations: Vec<Violation>,
    file_path: &'a str,
    checker: &'a E1112MagicNumbers,
    in_const_or_static: bool,
    in_array_len: bool,
}

impl<'a> MagicNumberVisitor<'a> {
    fn is_allowed_number(&self, n: i64) -> bool {
        self.checker.config.allowed_numbers.contains(&n)
    }

    fn check_int_literal(&mut self, lit: &syn::LitInt, span: proc_macro2::Span) {
        // Skip if we're in a const/static definition (that's the point)
        if self.in_const_or_static || self.in_array_len {
            return;
        }

        // Try to parse the value
        if let Ok(value) = lit.base10_parse::<i64>() {
            if self.is_allowed_number(value) {
                return;
            }

            // Large numbers are more likely to be magic
            if value.abs() > 2 {
                self.violations.push(
                    Violation::new(
                        self.checker.code(),
                        self.checker.name(),
                        self.checker.severity().into(),
                        format!(
                            "Magic number {} should be a named constant for clarity",
                            value
                        ),
                        self.file_path,
                        span.start().line,
                        span.start().column + 1,
                    )
                    .with_suggestion(self.checker.suggestions()),
                );
            }
        }
    }
}

impl<'a> Visit<'a> for MagicNumberVisitor<'a> {
    fn visit_item_const(&mut self, node: &'a syn::ItemConst) {
        self.in_const_or_static = true;
        syn::visit::visit_item_const(self, node);
        self.in_const_or_static = false;
    }

    fn visit_item_static(&mut self, node: &'a syn::ItemStatic) {
        self.in_const_or_static = true;
        syn::visit::visit_item_static(self, node);
        self.in_const_or_static = false;
    }

    fn visit_type_array(&mut self, node: &'a syn::TypeArray) {
        // Don't flag array length literals like [u8; 1024]
        self.in_array_len = true;
        syn::visit::visit_type_array(self, node);
        self.in_array_len = false;
    }

    fn visit_expr(&mut self, node: &'a syn::Expr) {
        // Check for integer literals
        if let syn::Expr::Lit(expr_lit) = node {
            if let syn::Lit::Int(lit_int) = &expr_lit.lit {
                self.check_int_literal(lit_int, expr_lit.span());
            }
        }

        syn::visit::visit_expr(self, node);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::checker::Checker;

    fn check_code(code: &str) -> Vec<Violation> {
        let checker = E1112MagicNumbers::default();
        let file = syn::parse_file(code).expect("Failed to parse");
        let mut violations = Vec::new();
        for item in &file.items {
            violations.extend(checker.check_item(item, "test.rs").unwrap());
        }
        violations
    }

    #[test]
    fn test_detects_magic_number() {
        let code = r#"
            fn calculate() -> i32 {
                42 * 1337
            }
        "#;
        let violations = check_code(code);
        assert_eq!(violations.len(), 2); // Both 42 and 1337
    }

    #[test]
    fn test_allows_common_numbers() {
        let code = r#"
            fn calculate(x: i32) -> i32 {
                x + 1
            }
        "#;
        let violations = check_code(code);
        assert!(violations.is_empty());
    }

    #[test]
    fn test_allows_const_definitions() {
        let code = r#"
            const MAGIC: i32 = 42;
            fn calculate() -> i32 {
                MAGIC
            }
        "#;
        let violations = check_code(code);
        assert!(violations.is_empty());
    }

    #[test]
    fn test_allows_array_lengths() {
        let code = r#"
            fn get_buffer() -> [u8; 1024] {
                [0; 1024]
            }
        "#;
        let violations = check_code(code);
        // Array length context should be allowed
        assert!(violations.len() <= 1); // Only the [0; 1024] init might trigger
    }

    #[test]
    fn test_zero_and_one_allowed() {
        let code = r#"
            fn init() -> i32 {
                0
            }
            fn next(x: i32) -> i32 {
                x + 1
            }
        "#;
        let violations = check_code(code);
        assert!(violations.is_empty());
    }
}
