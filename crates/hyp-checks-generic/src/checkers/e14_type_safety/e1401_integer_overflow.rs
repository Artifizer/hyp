//! E1401: Integer overflow/underflow (8/16/32-bit types + usize/isize)
//!
//! Detects unchecked arithmetic operations on 8/16/32-bit integers and usize/isize that can overflow
//! in release builds. In release mode, Rust wraps on overflow which causes subtle bugs.
//!
//! Note: usize/isize are included here (HIGH severity) because they're typically used for array
//! indices, lengths, and memory sizes where overflow is a critical security issue regardless of
//! platform. Explicit 64-bit types (i64/u64) used for large arbitrary numbers are covered by E1413
//! with lower severity.

use crate::{checker::Checker, define_checker, violation::Violation};

use syn::{spanned::Spanned, visit::Visit, BinOp};

define_checker! {
    /// Checker for E1401: Integer overflow/underflow (8/16/32-bit + usize/isize)
    E1401IntegerOverflow,
    code = "E1401",
    name = "Integer overflow/underflow (8/16/32-bit + usize/isize)",
    suggestions = "Use checked_add(), saturating_add(), or wrapping_add() for explicit overflow handling.",
    target_items = [Function],
    config_entry_name = "e1401_integer_overflow",
    /// Configuration for E1401: Integer overflow checker (includes usize/isize for security)
    config = E1401Config {
        /// Whether this checker is enabled
        enabled: bool = true,
        /// Severity level, defaults to High (explicit i64/u64 are E1413 with Low severity)
        severity: crate::config::SeverityLevel = crate::config::SeverityLevel::High,
        /// Categories this checker belongs to
        categories: Vec<crate::config::CheckerCategory> = vec![crate::config::CheckerCategory::Operations],
    },
    // AST node item checker
    check_item(self, item, file_path) {
        let mut visitor = OverflowVisitor {
            violations: Vec::new(),
            file_path,
            checker: self,
        };
        visitor.visit_item(item);
        Ok(visitor.violations)
    }
}

/// Visitor that looks for unchecked arithmetic operations
struct OverflowVisitor<'a> {
    violations: Vec<Violation>,
    file_path: &'a str,
    checker: &'a E1401IntegerOverflow,
}

impl<'a> OverflowVisitor<'a> {
    fn create_violation(&self, span: proc_macro2::Span, op: &str) -> Violation {
        let start = span.start();
        Violation::new(
            self.checker.code(),
            self.checker.name(),
            self.checker.severity().into(),
            format!(
                "Unchecked {} operation can overflow in release builds, causing wraparound behavior.",
                op
            ),
            self.file_path,
            start.line,
            start.column + 1,
        )
        .with_suggestion(self.checker.suggestions())
    }
}

impl<'a> Visit<'a> for OverflowVisitor<'a> {
    fn visit_expr_binary(&mut self, node: &'a syn::ExprBinary) {
        let op_name = match &node.op {
            BinOp::Add(_) => Some("addition"),
            BinOp::Sub(_) => Some("subtraction"),
            BinOp::Mul(_) => Some("multiplication"),
            _ => None,
        };

        if let Some(op) = op_name {
            // Flag if operation involves at least one literal AND it's a reasonable concern
            // Skip if BOTH operands are small literals (compiler will check those)
            // Skip only explicit 64-bit literals (i64/u64) - those are covered by E1413
            // Include usize/isize because they're critical for size/index calculations
            let has_non_64bit_literal = contains_non_64bit_integer_literal(&node.left)
                || contains_non_64bit_integer_literal(&node.right);
            let both_small_literals = is_small_literal(&node.left) && is_small_literal(&node.right);

            if has_non_64bit_literal && !both_small_literals {
                self.violations.push(self.create_violation(node.span(), op));
            }
        }

        // Continue visiting nested expressions
        syn::visit::visit_expr_binary(self, node);
    }
}

/// Check if an expression contains a non-64-bit integer literal
/// This excludes only i64, u64 suffixes (those are handled by E1413)
/// Note: usize/isize are included here because they're used for sizes/indices where
/// overflow is critical regardless of platform (32-bit or 64-bit)
fn contains_non_64bit_integer_literal(expr: &syn::Expr) -> bool {
    match expr {
        syn::Expr::Lit(lit) => {
            if let syn::Lit::Int(int_lit) = &lit.lit {
                let suffix = int_lit.suffix();
                // Exclude only explicit 64-bit types (i64, u64)
                // Include usize/isize because they're used for critical size/index calculations
                return suffix != "i64" && suffix != "u64";
            }
            false
        }
        syn::Expr::Binary(bin) => {
            contains_non_64bit_integer_literal(&bin.left)
                || contains_non_64bit_integer_literal(&bin.right)
        }
        syn::Expr::Paren(paren) => contains_non_64bit_integer_literal(&paren.expr),
        syn::Expr::Group(group) => contains_non_64bit_integer_literal(&group.expr),
        _ => false,
    }
}

/// Check if an expression is a small literal (< 20) that's safe in demo/test code
fn is_small_literal(expr: &syn::Expr) -> bool {
    match expr {
        syn::Expr::Lit(lit) => {
            if let syn::Lit::Int(int_lit) = &lit.lit {
                if let Ok(value) = int_lit.base10_parse::<u64>() {
                    return value < 20;
                }
            }
            false
        }
        syn::Expr::Paren(paren) => is_small_literal(&paren.expr),
        syn::Expr::Group(group) => is_small_literal(&group.expr),
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detects_addition_overflow() {
        let code = r#"
            fn example(x: u8) -> u8 {
                x + 100
            }
        "#;

        let syntax = syn::parse_file(code).unwrap();
        let checker = E1401IntegerOverflow::default();

        let mut violations = Vec::new();
        for item in &syntax.items {
            violations.extend(checker.check_item(item, "test.rs").unwrap());
        }

        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].code, "E1401");
        assert!(violations[0].message.contains("addition"));
    }

    #[test]
    fn test_detects_subtraction_overflow() {
        let code = r#"
            fn example(x: u8) -> u8 {
                x - 50
            }
        "#;

        let syntax = syn::parse_file(code).unwrap();
        let checker = E1401IntegerOverflow::default();

        let mut violations = Vec::new();
        for item in &syntax.items {
            violations.extend(checker.check_item(item, "test.rs").unwrap());
        }

        assert_eq!(violations.len(), 1);
        assert!(violations[0].message.contains("subtraction"));
    }

    #[test]
    fn test_detects_multiplication_overflow() {
        let code = r#"
            fn example(x: u8) -> u8 {
                x * 10
            }
        "#;

        let syntax = syn::parse_file(code).unwrap();
        let checker = E1401IntegerOverflow::default();

        let mut violations = Vec::new();
        for item in &syntax.items {
            violations.extend(checker.check_item(item, "test.rs").unwrap());
        }

        assert_eq!(violations.len(), 1);
        assert!(violations[0].message.contains("multiplication"));
    }

    #[test]
    fn test_checked_add_passes() {
        let code = r#"
            fn example(x: u8) -> Option<u8> {
                x.checked_add(100)
            }
        "#;

        let syntax = syn::parse_file(code).unwrap();
        let checker = E1401IntegerOverflow::default();

        let mut violations = Vec::new();
        for item in &syntax.items {
            violations.extend(checker.check_item(item, "test.rs").unwrap());
        }

        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_excludes_64bit_types() {
        let code = r#"
            fn example(x: u64) -> u64 {
                x + 1000u64
            }
        "#;

        let syntax = syn::parse_file(code).unwrap();
        let checker = E1401IntegerOverflow::default();

        let mut violations = Vec::new();
        for item in &syntax.items {
            violations.extend(checker.check_item(item, "test.rs").unwrap());
        }

        // Should not detect u64, that's for E1413
        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_excludes_i64_types() {
        let code = r#"
            fn example(x: i64) -> i64 {
                x - 1000i64
            }
        "#;

        let syntax = syn::parse_file(code).unwrap();
        let checker = E1401IntegerOverflow::default();

        let mut violations = Vec::new();
        for item in &syntax.items {
            violations.extend(checker.check_item(item, "test.rs").unwrap());
        }

        // Should not detect i64, that's for E1413
        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_detects_usize_operations() {
        let code = r#"
            fn example(x: usize) -> usize {
                x + 1000usize
            }
        "#;

        let syntax = syn::parse_file(code).unwrap();
        let checker = E1401IntegerOverflow::default();

        let mut violations = Vec::new();
        for item in &syntax.items {
            violations.extend(checker.check_item(item, "test.rs").unwrap());
        }

        // usize should be detected by E1401 (HIGH severity) because it's used for
        // critical size/index calculations regardless of platform
        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].code, "E1401");
    }

    #[test]
    fn test_detects_isize_operations() {
        let code = r#"
            fn example(x: isize) -> isize {
                x * 100isize
            }
        "#;

        let syntax = syn::parse_file(code).unwrap();
        let checker = E1401IntegerOverflow::default();

        let mut violations = Vec::new();
        for item in &syntax.items {
            violations.extend(checker.check_item(item, "test.rs").unwrap());
        }

        // isize should be detected by E1401 (HIGH severity)
        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].code, "E1401");
    }
}
