//! E1413: 64-bit integer overflow/underflow (i64/u64 only)
//!
//! Detects unchecked arithmetic operations on explicit 64-bit integers (i64, u64) that can overflow
//! in release builds. While less common than smaller types, 64-bit overflows still occur in scenarios
//! involving large data sets, timestamps, file sizes, or cryptographic operations.
//!
//! Note: usize/isize are NOT covered by this checker - they're handled by E1401 with HIGH severity
//! because they're typically used for array indices, lengths, and memory sizes where overflow is a
//! critical security issue regardless of platform.

use crate::{checker::Checker, define_checker, violation::Violation};

use syn::{spanned::Spanned, visit::Visit, BinOp};

define_checker! {
    /// Checker for E1413: 64-bit integer overflow/underflow
    E1413SixtyFourIntegerOverflow,
    code = "E1413",
    name = "64-bit integer overflow/underflow",
    suggestions = "Use checked_add(), saturating_add(), or wrapping_add() for explicit overflow handling, or use wider types like u128.",
    target_items = [Function],
    config_entry_name = "e1413_64bit_integer_overflow",
    /// Configuration for E1413: 64-bit integer overflow checker
    config = E1413Config {
        /// Whether this checker is enabled
        enabled: bool = true,
        /// Severity level, defaults to Low (less critical than smaller types)
        severity: crate::config::SeverityLevel = crate::config::SeverityLevel::Low,
        /// Categories this checker belongs to
        categories: Vec<crate::config::CheckerCategory> = vec![crate::config::CheckerCategory::Operations],
    },
    // AST node item checker
    check_item(self, item, file_path) {
        let mut visitor = SixtyFourOverflowVisitor {
            violations: Vec::new(),
            file_path,
            checker: self,
        };
        visitor.visit_item(item);
        Ok(visitor.violations)
    }
}

/// Visitor that looks for unchecked arithmetic operations on 64-bit integers
struct SixtyFourOverflowVisitor<'a> {
    violations: Vec<Violation>,
    file_path: &'a str,
    checker: &'a E1413SixtyFourIntegerOverflow,
}

impl<'a> SixtyFourOverflowVisitor<'a> {
    fn create_violation(&self, span: proc_macro2::Span, op: &str) -> Violation {
        let start = span.start();
        Violation::new(
            self.checker.code(),
            self.checker.name(),
            self.checker.severity().into(),
            format!(
                "Unchecked 64-bit {} operation can overflow in release builds with large values.",
                op
            ),
            self.file_path,
            start.line,
            start.column + 1,
        )
        .with_suggestion(self.checker.suggestions())
    }
}

impl<'a> Visit<'a> for SixtyFourOverflowVisitor<'a> {
    fn visit_expr_binary(&mut self, node: &'a syn::ExprBinary) {
        let op_name = match &node.op {
            BinOp::Add(_) => Some("addition"),
            BinOp::Sub(_) => Some("subtraction"),
            BinOp::Mul(_) => Some("multiplication"),
            _ => None,
        };

        if let Some(op) = op_name {
            // Check if operation involves explicit 64-bit literals (i64, u64 only)
            // Note: usize/isize are handled by E1401 due to their security-critical nature
            let has_64bit_literal = contains_64bit_integer_literal(&node.left)
                || contains_64bit_integer_literal(&node.right);
            let both_small_literals = is_small_literal(&node.left) && is_small_literal(&node.right);

            if has_64bit_literal && !both_small_literals {
                self.violations.push(self.create_violation(node.span(), op));
            }
        }

        // Continue visiting nested expressions
        syn::visit::visit_expr_binary(self, node);
    }
}

/// Check if an expression contains a 64-bit integer literal (suffixed with i64, u64 only)
/// Note: usize/isize are NOT included here - they're handled by E1401 (HIGH severity)
/// because they're typically used for critical size/index calculations
fn contains_64bit_integer_literal(expr: &syn::Expr) -> bool {
    match expr {
        syn::Expr::Lit(lit) => {
            if let syn::Lit::Int(int_lit) = &lit.lit {
                let suffix = int_lit.suffix();
                // Only explicit 64-bit types (i64, u64)
                // usize/isize are handled by E1401 due to their security-critical nature
                return suffix == "i64" || suffix == "u64";
            }
            false
        }
        syn::Expr::Binary(bin) => {
            contains_64bit_integer_literal(&bin.left)
                || contains_64bit_integer_literal(&bin.right)
        }
        syn::Expr::Paren(paren) => contains_64bit_integer_literal(&paren.expr),
        syn::Expr::Group(group) => contains_64bit_integer_literal(&group.expr),
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
    fn test_detects_u64_addition_overflow() {
        let code = r#"
            fn example(x: u64) -> u64 {
                x + 1_000_000_000_000u64
            }
        "#;

        let syntax = syn::parse_file(code).unwrap();
        let checker = E1413SixtyFourIntegerOverflow::default();

        let mut violations = Vec::new();
        for item in &syntax.items {
            violations.extend(checker.check_item(item, "test.rs").unwrap());
        }

        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].code, "E1413");
        assert!(violations[0].message.contains("addition"));
    }

    #[test]
    fn test_detects_i64_subtraction_overflow() {
        let code = r#"
            fn example(x: i64) -> i64 {
                x - 1_000_000_000i64
            }
        "#;

        let syntax = syn::parse_file(code).unwrap();
        let checker = E1413SixtyFourIntegerOverflow::default();

        let mut violations = Vec::new();
        for item in &syntax.items {
            violations.extend(checker.check_item(item, "test.rs").unwrap());
        }

        assert_eq!(violations.len(), 1);
        assert!(violations[0].message.contains("subtraction"));
    }

    #[test]
    fn test_detects_u64_multiplication_overflow() {
        let code = r#"
            fn example(x: u64, y: u64) -> u64 {
                x * 1_000_000_000u64
            }
        "#;

        let syntax = syn::parse_file(code).unwrap();
        let checker = E1413SixtyFourIntegerOverflow::default();

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
            fn example(x: u64) -> Option<u64> {
                x.checked_add(1_000_000_000u64)
            }
        "#;

        let syntax = syn::parse_file(code).unwrap();
        let checker = E1413SixtyFourIntegerOverflow::default();

        let mut violations = Vec::new();
        for item in &syntax.items {
            violations.extend(checker.check_item(item, "test.rs").unwrap());
        }

        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_ignores_small_literals() {
        let code = r#"
            fn example() -> u64 {
                5u64 + 10u64
            }
        "#;

        let syntax = syn::parse_file(code).unwrap();
        let checker = E1413SixtyFourIntegerOverflow::default();

        let mut violations = Vec::new();
        for item in &syntax.items {
            violations.extend(checker.check_item(item, "test.rs").unwrap());
        }

        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_ignores_u32_operations() {
        let code = r#"
            fn example(x: u32) -> u32 {
                x + 100u32
            }
        "#;

        let syntax = syn::parse_file(code).unwrap();
        let checker = E1413SixtyFourIntegerOverflow::default();

        let mut violations = Vec::new();
        for item in &syntax.items {
            violations.extend(checker.check_item(item, "test.rs").unwrap());
        }

        // Should not detect u32, that's for E1401
        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_ignores_usize_operations() {
        let code = r#"
            fn example(x: usize) -> usize {
                x + 1000usize
            }
        "#;

        let syntax = syn::parse_file(code).unwrap();
        let checker = E1413SixtyFourIntegerOverflow::default();

        let mut violations = Vec::new();
        for item in &syntax.items {
            violations.extend(checker.check_item(item, "test.rs").unwrap());
        }

        // Should not detect usize - that's for E1401 (HIGH severity)
        // usize is used for sizes/indices which are security-critical
        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_ignores_isize_operations() {
        let code = r#"
            fn example(x: isize) -> isize {
                x * 100isize
            }
        "#;

        let syntax = syn::parse_file(code).unwrap();
        let checker = E1413SixtyFourIntegerOverflow::default();

        let mut violations = Vec::new();
        for item in &syntax.items {
            violations.extend(checker.check_item(item, "test.rs").unwrap());
        }

        // Should not detect isize - that's for E1401 (HIGH severity)
        assert_eq!(violations.len(), 0);
    }
}
