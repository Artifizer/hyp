//! E1402: Division by zero
//!
//! Detects division operations that don't check for zero divisors.
//! Division by zero causes a runtime panic.

use crate::{checker::Checker, define_checker, violation::Violation};

use syn::{spanned::Spanned, visit::Visit, BinOp};

define_checker! {
    /// Checker for E1402: Division by zero
    E1402DivisionByZero,
    code = "E1402",
    name = "Division by zero",
    suggestions = "Use checked_div() which returns None for division by zero, or validate the divisor before dividing.",
    target_items = [Function],
    config_entry_name = "e1402_division_by_zero",
    /// Configuration for E1402: Division by zero checker
    config = E1402Config {
        /// Whether this checker is enabled
        enabled: bool = true,
        /// Severity level, defaults to High
        severity: crate::config::SeverityLevel = crate::config::SeverityLevel::High,
        /// Categories this checker belongs to
        categories: Vec<crate::config::CheckerCategory> = vec![crate::config::CheckerCategory::Operations],
    },
    // AST node item checker
    check_item(self, item, file_path) {
        let mut visitor = DivisionVisitor {
            violations: Vec::new(),
            file_path,
            checker: self,
            protected_divisors: Vec::new(),
        };
        visitor.visit_item(item);
        Ok(visitor.violations)
    }
}

/// Visitor that looks for division operations
struct DivisionVisitor<'a> {
    violations: Vec<Violation>,
    file_path: &'a str,
    checker: &'a E1402DivisionByZero,
    /// Track if we're inside a protected context (e.g., inside else block after checking for zero)
    protected_divisors: Vec<String>,
}

impl<'a> DivisionVisitor<'a> {
    fn create_violation(&self, span: proc_macro2::Span) -> Violation {
        let start = span.start();
        Violation::new(
            self.checker.code(),
            self.checker.name(),
            self.checker.severity().into(),
            "Division operation without zero check. Will panic if divisor is zero.",
            self.file_path,
            start.line,
            start.column + 1,
        )
        .with_suggestion(self.checker.suggestions())
    }

    /// Check if a divisor is protected by context
    fn is_protected(&self, divisor_expr: &syn::Expr) -> bool {
        let divisor_str = expr_to_string(divisor_expr).replace(" ", "");
        self.protected_divisors.iter().any(|p| {
            let p_normalized = p.replace(" ", "");
            divisor_str.contains(&p_normalized)
        })
    }
}

impl<'a> Visit<'a> for DivisionVisitor<'a> {
    fn visit_expr_binary(&mut self, node: &'a syn::ExprBinary) {
        if matches!(node.op, BinOp::Div(_) | BinOp::Rem(_)) {
            // Check if the divisor is a non-zero literal (safe case)
            // or if it's protected by context
            if !is_non_zero_literal(&node.right) && !self.is_protected(&node.right) {
                self.violations.push(self.create_violation(node.span()));
            }
        }

        // Continue visiting nested expressions
        syn::visit::visit_expr_binary(self, node);
    }

    fn visit_expr_if(&mut self, node: &'a syn::ExprIf) {
        // Check if the condition protects against zero/empty
        let (protected_in_else, protected_in_then) = extract_protected_divisors(&node.cond);

        // Visit the condition
        self.visit_expr(&node.cond);

        // Visit the then branch with its protection context
        if !protected_in_then.is_empty() {
            let original_len = self.protected_divisors.len();
            self.protected_divisors.extend(protected_in_then);
            self.visit_block(&node.then_branch);
            self.protected_divisors.truncate(original_len);
        } else {
            self.visit_block(&node.then_branch);
        }

        // Visit the else branch with its protection context
        if let Some((_, else_branch)) = &node.else_branch {
            if !protected_in_else.is_empty() {
                let original_len = self.protected_divisors.len();
                self.protected_divisors.extend(protected_in_else);
                self.visit_expr(else_branch);
                self.protected_divisors.truncate(original_len);
            } else {
                self.visit_expr(else_branch);
            }
        }
    }
}

/// Check if an expression is a non-zero integer literal
fn is_non_zero_literal(expr: &syn::Expr) -> bool {
    match expr {
        syn::Expr::Lit(lit) => {
            if let syn::Lit::Int(int_lit) = &lit.lit {
                // Parse the literal value and check if it's non-zero
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

/// Convert expression to a string for pattern matching
fn expr_to_string(expr: &syn::Expr) -> String {
    quote::quote!(#expr).to_string()
}

/// Extract protected divisors from a condition
/// Returns (protected_in_else, protected_in_then)
/// For example:
/// - if x.is_empty() -> else branch is safe for x.len()
/// - if !x.is_empty() -> then branch is safe for x.len()
/// - if y == 0 -> else branch is safe for y
/// - if y != 0 -> then branch is safe for y
fn extract_protected_divisors(cond: &syn::Expr) -> (Vec<String>, Vec<String>) {
    let mut protected_in_else = Vec::new();
    let mut protected_in_then = Vec::new();

    match cond {
        // x.is_empty() - else branch is safe for x.len()
        syn::Expr::MethodCall(method) if method.method == "is_empty" => {
            let receiver_str = expr_to_string(&method.receiver);
            let receiver_str = receiver_str.replace(" ", "");
            protected_in_else.push(format!("{}.len()", receiver_str));
        }
        // Binary operations
        syn::Expr::Binary(bin) => {
            match bin.op {
                // x != 0, x > 0, x >= 1 - then branch is safe
                BinOp::Ne(_) | BinOp::Gt(_) | BinOp::Ge(_) => {
                    if is_zero_literal(&bin.right) {
                        protected_in_then.push(expr_to_string(&bin.left).replace(" ", ""));
                    }
                    if is_zero_literal(&bin.left) {
                        protected_in_then.push(expr_to_string(&bin.right).replace(" ", ""));
                    }
                }
                // x == 0, x <= 0 - else branch is safe
                BinOp::Eq(_) | BinOp::Le(_) => {
                    if is_zero_literal(&bin.right) {
                        protected_in_else.push(expr_to_string(&bin.left).replace(" ", ""));
                    }
                    if is_zero_literal(&bin.left) {
                        protected_in_else.push(expr_to_string(&bin.right).replace(" ", ""));
                    }
                }
                _ => {}
            }
        }
        // !x.is_empty() - then branch is safe for x.len()
        syn::Expr::Unary(unary) if matches!(unary.op, syn::UnOp::Not(_)) => {
            if let syn::Expr::MethodCall(method) = &*unary.expr {
                if method.method == "is_empty" {
                    let receiver_str = expr_to_string(&method.receiver);
                    let receiver_str = receiver_str.replace(" ", "");
                    protected_in_then.push(format!("{}.len()", receiver_str));
                }
            }
        }
        _ => {}
    }

    (protected_in_else, protected_in_then)
}

/// Check if an expression is a zero literal
fn is_zero_literal(expr: &syn::Expr) -> bool {
    match expr {
        syn::Expr::Lit(lit) => {
            if let syn::Lit::Int(int_lit) = &lit.lit {
                int_lit.base10_parse::<i128>().map(|v| v == 0).unwrap_or(false)
            } else if let syn::Lit::Float(float_lit) = &lit.lit {
                float_lit.base10_parse::<f64>().map(|v| v.abs() < f64::EPSILON).unwrap_or(false)
            } else {
                false
            }
        }
        syn::Expr::Paren(paren) => is_zero_literal(&paren.expr),
        syn::Expr::Group(group) => is_zero_literal(&group.expr),
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detects_division_by_variable() {
        let code = r#"
            fn example(x: i32, y: i32) -> i32 {
                x / y
            }
        "#;

        let syntax = syn::parse_file(code).unwrap();
        let checker = E1402DivisionByZero::default();

        let mut violations = Vec::new();
        for item in &syntax.items {
            violations.extend(checker.check_item(item, "test.rs").unwrap());
        }

        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].code, "E1402");
    }

    #[test]
    fn test_allows_division_by_non_zero_literal() {
        let code = r#"
            fn example(x: i32) -> i32 {
                x / 2
            }
        "#;

        let syntax = syn::parse_file(code).unwrap();
        let checker = E1402DivisionByZero::default();

        let mut violations = Vec::new();
        for item in &syntax.items {
            violations.extend(checker.check_item(item, "test.rs").unwrap());
        }

        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_detects_division_by_zero_literal() {
        let code = r#"
            fn example(x: i32) -> i32 {
                x / 0
            }
        "#;

        let syntax = syn::parse_file(code).unwrap();
        let checker = E1402DivisionByZero::default();

        let mut violations = Vec::new();
        for item in &syntax.items {
            violations.extend(checker.check_item(item, "test.rs").unwrap());
        }

        // Division by 0 literal is still flagged (not a non-zero literal)
        assert_eq!(violations.len(), 1);
    }

    #[test]
    fn test_no_division_passes() {
        let code = r#"
            fn example(x: i32, y: i32) -> i32 {
                x + y
            }
        "#;

        let syntax = syn::parse_file(code).unwrap();
        let checker = E1402DivisionByZero::default();

        let mut violations = Vec::new();
        for item in &syntax.items {
            violations.extend(checker.check_item(item, "test.rs").unwrap());
        }

        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_protected_by_is_empty_check() {
        let code = r#"
            fn example(violations: Vec<i32>, total_loc: usize) -> f64 {
                let score = if violations.is_empty() {
                    0.0
                } else {
                    total_loc as f64 / violations.len() as f64
                };
                score
            }
        "#;

        let syntax = syn::parse_file(code).unwrap();
        let checker = E1402DivisionByZero::default();

        let mut violations = Vec::new();
        for item in &syntax.items {
            violations.extend(checker.check_item(item, "test.rs").unwrap());
        }

        // Should not flag - division is protected by is_empty() check
        assert_eq!(violations.len(), 0, "Division protected by is_empty() should not be flagged");
    }

    #[test]
    fn test_protected_by_not_equal_zero() {
        let code = r#"
            fn example(x: i32, y: i32) -> i32 {
                if y == 0 {
                    0
                } else {
                    x / y
                }
            }
        "#;

        let syntax = syn::parse_file(code).unwrap();
        let checker = E1402DivisionByZero::default();

        let mut violations = Vec::new();
        for item in &syntax.items {
            violations.extend(checker.check_item(item, "test.rs").unwrap());
        }

        // Should not flag - division is protected by y == 0 check
        assert_eq!(violations.len(), 0, "Division protected by zero check should not be flagged");
    }

    #[test]
    fn test_unprotected_division_in_then_branch() {
        let code = r#"
            fn example(violations: Vec<i32>, total_loc: usize) -> f64 {
                if violations.is_empty() {
                    total_loc as f64 / violations.len() as f64
                } else {
                    0.0
                }
            }
        "#;

        let syntax = syn::parse_file(code).unwrap();
        let checker = E1402DivisionByZero::default();

        let mut violations = Vec::new();
        for item in &syntax.items {
            violations.extend(checker.check_item(item, "test.rs").unwrap());
        }

        // Should flag - division in then branch is NOT protected
        assert_eq!(violations.len(), 1, "Division in then branch after is_empty() should be flagged");
    }
}
