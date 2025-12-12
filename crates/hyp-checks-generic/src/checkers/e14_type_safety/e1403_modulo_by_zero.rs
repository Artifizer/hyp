//! E1403: Modulo by zero
//!
//! Detects modulo operations that don't check for zero divisors.
//! Modulo by zero causes a runtime panic.

use crate::{checker::Checker, define_checker, violation::Violation};

use syn::{spanned::Spanned, visit::Visit, BinOp};

define_checker! {
    /// Checker for E1403: Modulo by zero
    E1403ModuloByZero,
    code = "E1403",
    name = "Modulo by zero",
    suggestions = "Use checked_rem() which returns None for modulo by zero, or validate the divisor before the operation.",
    target_items = [Function],
    config_entry_name = "e1403_modulo_by_zero",
    /// Configuration for E1403: Modulo by zero checker
    config = E1403Config {
        /// Whether this checker is enabled
        enabled: bool = true,
        /// Severity level, defaults to High
        severity: crate::config::SeverityLevel = crate::config::SeverityLevel::High,
        /// Categories this checker belongs to
        categories: Vec<crate::config::CheckerCategory> = vec![crate::config::CheckerCategory::Operations],
    },
    // AST node item checker
    check_item(self, item, file_path) {
        let mut visitor = ModuloVisitor {
            violations: Vec::new(),
            file_path,
            checker: self,
        };
        visitor.visit_item(item);
        Ok(visitor.violations)
    }
}

/// Visitor that looks for modulo operations
struct ModuloVisitor<'a> {
    violations: Vec<Violation>,
    file_path: &'a str,
    checker: &'a E1403ModuloByZero,
}

impl<'a> ModuloVisitor<'a> {
    fn create_violation(&self, span: proc_macro2::Span) -> Violation {
        let start = span.start();
        Violation::new(
            self.checker.code(),
            self.checker.name(),
            self.checker.severity().into(),
            "Modulo operation without zero check. Will panic if divisor is zero.",
            self.file_path,
            start.line,
            start.column + 1,
        )
        .with_suggestion(self.checker.suggestions())
    }
}

impl<'a> Visit<'a> for ModuloVisitor<'a> {
    fn visit_expr_binary(&mut self, node: &'a syn::ExprBinary) {
        if matches!(node.op, BinOp::Rem(_)) {
            // Check if the divisor is a non-zero literal (safe case)
            if !is_non_zero_literal(&node.right) {
                self.violations.push(self.create_violation(node.span()));
            }
        }

        // Continue visiting nested expressions
        syn::visit::visit_expr_binary(self, node);
    }
}

/// Check if an expression is a non-zero integer literal
fn is_non_zero_literal(expr: &syn::Expr) -> bool {
    match expr {
        syn::Expr::Lit(lit) => {
            if let syn::Lit::Int(int_lit) = &lit.lit {
                int_lit
                    .base10_parse::<i128>()
                    .map(|v| v != 0)
                    .unwrap_or(false)
            } else {
                false
            }
        }
        syn::Expr::Paren(paren) => is_non_zero_literal(&paren.expr),
        syn::Expr::Group(group) => is_non_zero_literal(&group.expr),
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detects_modulo_by_variable() {
        let code = r#"
            fn example(x: i32, y: i32) -> i32 {
                x % y
            }
        "#;

        let syntax = syn::parse_file(code).unwrap();
        let checker = E1403ModuloByZero::default();

        let mut violations = Vec::new();
        for item in &syntax.items {
            violations.extend(checker.check_item(item, "test.rs").unwrap());
        }

        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].code, "E1403");
    }

    #[test]
    fn test_allows_modulo_by_non_zero_literal() {
        let code = r#"
            fn example(x: i32) -> i32 {
                x % 7
            }
        "#;

        let syntax = syn::parse_file(code).unwrap();
        let checker = E1403ModuloByZero::default();

        let mut violations = Vec::new();
        for item in &syntax.items {
            violations.extend(checker.check_item(item, "test.rs").unwrap());
        }

        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_detects_modulo_by_zero_literal() {
        let code = r#"
            fn example(x: i32) -> i32 {
                x % 0
            }
        "#;

        let syntax = syn::parse_file(code).unwrap();
        let checker = E1403ModuloByZero::default();

        let mut violations = Vec::new();
        for item in &syntax.items {
            violations.extend(checker.check_item(item, "test.rs").unwrap());
        }

        assert_eq!(violations.len(), 1);
    }

    #[test]
    fn test_no_modulo_passes() {
        let code = r#"
            fn example(x: i32, y: i32) -> i32 {
                x * y
            }
        "#;

        let syntax = syn::parse_file(code).unwrap();
        let checker = E1403ModuloByZero::default();

        let mut violations = Vec::new();
        for item in &syntax.items {
            violations.extend(checker.check_item(item, "test.rs").unwrap());
        }

        assert_eq!(violations.len(), 0);
    }
}
