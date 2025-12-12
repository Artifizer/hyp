//! E1405: Integer division rounding errors
//!
//! Detects integer division that might lose precision when the result
//! should be a floating-point value.

use crate::{checker::Checker, define_checker, violation::Violation};

use syn::{spanned::Spanned, visit::Visit, BinOp};

define_checker! {
    /// Checker for E1405: Integer division rounding errors
    E1405IntegerDivisionRounding,
    code = "E1405",
    name = "Integer division rounding",
    suggestions = "Cast to floating-point before division if precision is needed, or use checked_div()",
    target_items = [Function],
    config_entry_name = "e1405_integer_division_rounding",
    /// Configuration for E1405: Integer division rounding checker
    config = E1405Config {
        /// Whether this checker is enabled
        enabled: bool = true,
        /// Severity level, defaults to Low
        severity: crate::config::SeverityLevel = crate::config::SeverityLevel::Low,
        /// Categories this checker belongs to
        categories: Vec<crate::config::CheckerCategory> = vec![crate::config::CheckerCategory::Operations],
    },
    check_item(self, item, file_path) {
        let mut visitor = DivisionVisitor {
            violations: Vec::new(),
            file_path,
            checker: self,
        };
        visitor.visit_item(item);
        Ok(visitor.violations)
    }
}

struct DivisionVisitor<'a> {
    violations: Vec<Violation>,
    file_path: &'a str,
    checker: &'a E1405IntegerDivisionRounding,
}

impl<'a> DivisionVisitor<'a> {
    fn create_violation(&self, span: proc_macro2::Span) -> Violation {
        let start = span.start();
        Violation::new(
            self.checker.code(),
            self.checker.name(),
            self.checker.severity().into(),
            "Integer division truncates toward zero. If precision is needed, convert to floating-point first.",
            self.file_path,
            start.line,
            start.column + 1,
        )
        .with_suggestion(self.checker.suggestions())
    }
}

impl<'a> Visit<'a> for DivisionVisitor<'a> {
    fn visit_expr_binary(&mut self, node: &'a syn::ExprBinary) {
        if let BinOp::Div(_) = node.op {
            // Check if both operands look like integers (literals)
            if is_likely_integer_expr(&node.left) && is_likely_integer_expr(&node.right) {
                self.violations.push(self.create_violation(node.span()));
            }
        }

        syn::visit::visit_expr_binary(self, node);
    }
}

/// Heuristic to detect if an expression is likely an integer
fn is_likely_integer_expr(expr: &syn::Expr) -> bool {
    match expr {
        syn::Expr::Lit(lit) => matches!(lit.lit, syn::Lit::Int(_)),
        syn::Expr::Path(_) => true, // Variables could be integers
        syn::Expr::Paren(paren) => is_likely_integer_expr(&paren.expr),
        syn::Expr::Binary(bin) => {
            // Arithmetic on integers produces integers
            matches!(bin.op, BinOp::Add(_) | BinOp::Sub(_) | BinOp::Mul(_) | BinOp::Div(_) | BinOp::Rem(_))
        }
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detects_integer_division() {
        let code = r#"
            fn example() {
                let result = 5 / 2;  // Returns 2, not 2.5
            }
        "#;

        let syntax = syn::parse_file(code).unwrap();
        let checker = E1405IntegerDivisionRounding::default();

        let mut violations = Vec::new();
        for item in &syntax.items {
            violations.extend(checker.check_item(item, "test.rs").unwrap());
        }

        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].code, "E1405");
    }

    #[test]
    fn test_float_division_passes() {
        let code = r#"
            fn example() {
                let result = 5.0 / 2.0;
            }
        "#;

        let syntax = syn::parse_file(code).unwrap();
        let checker = E1405IntegerDivisionRounding::default();

        let mut violations = Vec::new();
        for item in &syntax.items {
            violations.extend(checker.check_item(item, "test.rs").unwrap());
        }

        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_variable_division() {
        let code = r#"
            fn example(a: i32, b: i32) -> i32 {
                a / b
            }
        "#;

        let syntax = syn::parse_file(code).unwrap();
        let checker = E1405IntegerDivisionRounding::default();

        let mut violations = Vec::new();
        for item in &syntax.items {
            violations.extend(checker.check_item(item, "test.rs").unwrap());
        }

        // Variables are flagged since they might be integers
        assert_eq!(violations.len(), 1);
    }
}
