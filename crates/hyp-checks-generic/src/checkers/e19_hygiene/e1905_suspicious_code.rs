//! E1905: Suspicious Code Detection
//!
//! Detects code patterns that are almost certainly bugs:
//! - eq_op: Comparing identical expressions (e.g., `x == x`)
//! - ifs_same_cond: Same condition in if-else chain
//! - self_assignment: Assigning variable to itself
//! - never_loop: Loop that never loops (unconditional break at start)
//! - while_immutable_condition: Infinite/never loop due to literal condition
//! - impossible_comparisons: Comparisons that are always true/false

use crate::{checker::Checker, define_checker, violation::Violation};
use syn::{spanned::Spanned, visit::Visit, BinOp, Expr, ExprBinary, Stmt};

define_checker! {
    /// Checker for E1905: Suspicious code detection
    E1905SuspiciousCode,
    code = "E1905",
    name = "Suspicious code pattern detected",
    suggestions = "Review and fix the suspicious pattern - these are almost always bugs",
    target_items = [Function, Impl],
    config_entry_name = "e1905_suspicious_code",
    config = E1905Config {
        enabled: bool = true,
        severity: crate::config::SeverityLevel = crate::config::SeverityLevel::High,
        categories: Vec<crate::config::CheckerCategory> = vec![crate::config::CheckerCategory::Operations],
        detect_eq_op: bool = true,
        detect_ifs_same_cond: bool = true,
        detect_self_assignment: bool = true,
        detect_never_loop: bool = true,
        detect_while_immutable_condition: bool = true,
        detect_impossible_comparisons: bool = true,
    },
    check_item(self, item, file_path) {
        let mut visitor = SuspiciousCodeVisitor {
            violations: Vec::new(),
            file_path,
            checker: self,
        };
        visitor.visit_item(item);
        Ok(visitor.violations)
    }
}

struct SuspiciousCodeVisitor<'a> {
    violations: Vec<Violation>,
    file_path: &'a str,
    checker: &'a E1905SuspiciousCode,
}

/// Compare two expressions for structural equality via their token stream representation
fn exprs_equal(left: &Expr, right: &Expr) -> bool {
    let left_tokens = quote::quote!(#left).to_string();
    let right_tokens = quote::quote!(#right).to_string();
    left_tokens == right_tokens
}

/// Check if an expression is a literal boolean
fn is_literal_bool(expr: &Expr) -> Option<bool> {
    if let Expr::Lit(lit) = expr {
        if let syn::Lit::Bool(b) = &lit.lit {
            return Some(b.value);
        }
    }
    None
}

/// Check if a statement is an unconditional break
fn is_unconditional_break(stmt: &Stmt) -> bool {
    matches!(stmt, Stmt::Expr(Expr::Break(_), _))
}

impl<'a> SuspiciousCodeVisitor<'a> {
    fn add_violation(&mut self, message: &str, span: proc_macro2::Span) {
        self.violations.push(
            Violation::new(
                self.checker.code(),
                self.checker.name(),
                self.checker.severity().into(),
                message,
                self.file_path,
                span.start().line,
                span.start().column + 1,
            )
            .with_suggestion(self.checker.suggestions()),
        );
    }

    /// Check for eq_op and impossible_comparisons patterns
    fn check_binary_expr(&mut self, expr: &ExprBinary) {
        let left = &*expr.left;
        let right = &*expr.right;

        if !exprs_equal(left, right) {
            return;
        }

        match expr.op {
            // eq_op: redundant comparisons
            BinOp::Eq(_) => {
                if self.checker.config.detect_eq_op {
                    self.add_violation(
                        "Comparing identical expressions with '==' - always true (suspicious)",
                        expr.span(),
                    );
                }
            }
            BinOp::Ne(_) => {
                if self.checker.config.detect_eq_op {
                    self.add_violation(
                        "Comparing identical expressions with '!=' - always false (suspicious)",
                        expr.span(),
                    );
                }
            }
            // impossible_comparisons: strict inequalities
            BinOp::Lt(_) => {
                if self.checker.config.detect_impossible_comparisons {
                    self.add_violation(
                        "Comparing identical expressions with '<' - always false (impossible comparison)",
                        expr.span(),
                    );
                }
            }
            BinOp::Gt(_) => {
                if self.checker.config.detect_impossible_comparisons {
                    self.add_violation(
                        "Comparing identical expressions with '>' - always false (impossible comparison)",
                        expr.span(),
                    );
                }
            }
            BinOp::Le(_) => {
                if self.checker.config.detect_impossible_comparisons {
                    self.add_violation(
                        "Comparing identical expressions with '<=' - always true (trivial comparison)",
                        expr.span(),
                    );
                }
            }
            BinOp::Ge(_) => {
                if self.checker.config.detect_impossible_comparisons {
                    self.add_violation(
                        "Comparing identical expressions with '>=' - always true (trivial comparison)",
                        expr.span(),
                    );
                }
            }
            // eq_op: redundant boolean operations
            BinOp::And(_) => {
                if self.checker.config.detect_eq_op {
                    self.add_violation(
                        "Identical expressions in '&&' - redundant (suspicious)",
                        expr.span(),
                    );
                }
            }
            BinOp::Or(_) => {
                if self.checker.config.detect_eq_op {
                    self.add_violation(
                        "Identical expressions in '||' - redundant (suspicious)",
                        expr.span(),
                    );
                }
            }
            // eq_op: redundant bitwise operations
            BinOp::BitXor(_) => {
                if self.checker.config.detect_eq_op {
                    self.add_violation(
                        "XOR of identical expressions - always zero (suspicious)",
                        expr.span(),
                    );
                }
            }
            BinOp::Sub(_) => {
                if self.checker.config.detect_eq_op {
                    self.add_violation(
                        "Subtracting identical expressions - always zero (suspicious)",
                        expr.span(),
                    );
                }
            }
            BinOp::Div(_) => {
                if self.checker.config.detect_eq_op {
                    self.add_violation(
                        "Dividing identical expressions - always one (suspicious)",
                        expr.span(),
                    );
                }
            }
            BinOp::Rem(_) => {
                if self.checker.config.detect_eq_op {
                    self.add_violation(
                        "Modulo of identical expressions - always zero (suspicious)",
                        expr.span(),
                    );
                }
            }
            _ => {}
        }
    }

    /// Check for self-assignment pattern
    fn check_assignment(&mut self, left: &Expr, right: &Expr, span: proc_macro2::Span) {
        if !self.checker.config.detect_self_assignment {
            return;
        }

        if exprs_equal(left, right) {
            self.add_violation(
                "Self-assignment detected - assigning variable to itself has no effect",
                span,
            );
        }
    }

    /// Check for ifs_same_cond pattern
    fn check_if_chain(&mut self, expr: &syn::ExprIf) {
        if !self.checker.config.detect_ifs_same_cond {
            return;
        }

        let mut conditions: Vec<(String, proc_macro2::Span)> = Vec::new();
        let first_cond = &*expr.cond;
        let cond_str = quote::quote!(#first_cond).to_string();
        conditions.push((cond_str, expr.cond.span()));

        // Walk the else-if chain
        let mut current_else = &expr.else_branch;
        while let Some((_, else_expr)) = current_else {
            if let Expr::If(else_if) = &**else_expr {
                let cond = &*else_if.cond;
                let cond_str = quote::quote!(#cond).to_string();
                let span = else_if.cond.span();

                // Check for duplicate condition
                for (prev_cond, prev_span) in &conditions {
                    if *prev_cond == cond_str {
                        self.add_violation(
                            &format!(
                                "Same condition used in if-else chain (first at line {}) - likely a typo",
                                prev_span.start().line
                            ),
                            span,
                        );
                    }
                }

                conditions.push((cond_str, span));
                current_else = &else_if.else_branch;
            } else {
                break;
            }
        }
    }

    /// Check for never_loop pattern
    fn check_loop_body(&mut self, stmts: &[Stmt], loop_span: proc_macro2::Span) {
        if !self.checker.config.detect_never_loop {
            return;
        }

        // Check if first statement is unconditional break
        if let Some(first) = stmts.first() {
            if is_unconditional_break(first) {
                self.add_violation(
                    "Loop never iterates - unconditional break at start",
                    loop_span,
                );
            }
        }
    }

    /// Check for while_immutable_condition pattern
    fn check_while_condition(&mut self, cond: &Expr, loop_span: proc_macro2::Span) {
        if !self.checker.config.detect_while_immutable_condition {
            return;
        }

        match is_literal_bool(cond) {
            Some(true) => {
                self.add_violation(
                    "While loop with 'true' condition - infinite loop unless broken",
                    loop_span,
                );
            }
            Some(false) => {
                self.add_violation(
                    "While loop with 'false' condition - body never executes",
                    loop_span,
                );
            }
            None => {}
        }
    }
}

impl<'a> Visit<'a> for SuspiciousCodeVisitor<'a> {
    fn visit_expr_binary(&mut self, node: &'a ExprBinary) {
        self.check_binary_expr(node);
        syn::visit::visit_expr_binary(self, node);
    }

    fn visit_expr_assign(&mut self, node: &'a syn::ExprAssign) {
        self.check_assignment(&node.left, &node.right, node.span());
        syn::visit::visit_expr_assign(self, node);
    }

    fn visit_expr_if(&mut self, node: &'a syn::ExprIf) {
        self.check_if_chain(node);
        syn::visit::visit_expr_if(self, node);
    }

    fn visit_expr_loop(&mut self, node: &'a syn::ExprLoop) {
        self.check_loop_body(&node.body.stmts, node.span());
        syn::visit::visit_expr_loop(self, node);
    }

    fn visit_expr_while(&mut self, node: &'a syn::ExprWhile) {
        self.check_while_condition(&node.cond, node.span());
        self.check_loop_body(&node.body.stmts, node.span());
        syn::visit::visit_expr_while(self, node);
    }

    fn visit_expr_for_loop(&mut self, node: &'a syn::ExprForLoop) {
        self.check_loop_body(&node.body.stmts, node.span());
        syn::visit::visit_expr_for_loop(self, node);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn check_code(code: &str) -> Vec<Violation> {
        let checker = E1905SuspiciousCode::default();
        let file = syn::parse_file(code).expect("Failed to parse");
        let mut violations = Vec::new();
        for item in &file.items {
            violations.extend(checker.check_item(item, "test.rs").unwrap());
        }
        violations
    }

    #[test]
    fn test_eq_op_equal() {
        let code = r#"
            fn test() {
                let x = 5;
                if x == x { }
            }
        "#;
        let violations = check_code(code);
        assert_eq!(violations.len(), 1);
        assert!(violations[0].message.contains("always true"));
    }

    #[test]
    fn test_eq_op_not_equal() {
        let code = r#"
            fn test() {
                let x = 5;
                if x != x { }
            }
        "#;
        let violations = check_code(code);
        assert_eq!(violations.len(), 1);
        assert!(violations[0].message.contains("always false"));
    }

    #[test]
    fn test_eq_op_and() {
        let code = r#"
            fn test() {
                let x = true;
                if x && x { }
            }
        "#;
        let violations = check_code(code);
        assert_eq!(violations.len(), 1);
        assert!(violations[0].message.contains("redundant"));
    }

    #[test]
    fn test_impossible_comparison_lt() {
        let code = r#"
            fn test() {
                let x = 5;
                if x < x { }
            }
        "#;
        let violations = check_code(code);
        assert_eq!(violations.len(), 1);
        assert!(violations[0].message.contains("always false"));
    }

    #[test]
    fn test_impossible_comparison_gt() {
        let code = r#"
            fn test() {
                let x = 5;
                if x > x { }
            }
        "#;
        let violations = check_code(code);
        assert_eq!(violations.len(), 1);
        assert!(violations[0].message.contains("always false"));
    }

    #[test]
    fn test_self_assignment() {
        let code = r#"
            fn test() {
                let mut x = 5;
                x = x;
            }
        "#;
        let violations = check_code(code);
        assert_eq!(violations.len(), 1);
        assert!(violations[0].message.contains("Self-assignment"));
    }

    #[test]
    fn test_ifs_same_cond() {
        let code = r#"
            fn test() {
                let x = 5;
                if x > 0 {
                } else if x > 0 {
                }
            }
        "#;
        let violations = check_code(code);
        assert_eq!(violations.len(), 1);
        assert!(violations[0].message.contains("Same condition"));
    }

    #[test]
    fn test_never_loop() {
        let code = r#"
            fn test() {
                loop {
                    break;
                }
            }
        "#;
        let violations = check_code(code);
        assert_eq!(violations.len(), 1);
        assert!(violations[0].message.contains("never iterates"));
    }

    #[test]
    fn test_while_true() {
        let code = r#"
            fn test() {
                while true {
                    break;
                }
            }
        "#;
        let violations = check_code(code);
        // Should detect both while_immutable_condition and never_loop
        assert!(violations.len() >= 1);
        assert!(violations.iter().any(|v| v.message.contains("infinite")));
    }

    #[test]
    fn test_while_false() {
        let code = r#"
            fn test() {
                while false {
                    println!("never");
                }
            }
        "#;
        let violations = check_code(code);
        assert_eq!(violations.len(), 1);
        assert!(violations[0].message.contains("never executes"));
    }

    #[test]
    fn test_no_false_positive_different_exprs() {
        let code = r#"
            fn test() {
                let x = 5;
                let y = 10;
                if x == y { }
                if x > y { }
            }
        "#;
        let violations = check_code(code);
        assert!(violations.is_empty());
    }

    #[test]
    fn test_xor_same() {
        let code = r#"
            fn test() {
                let x = 5;
                let _y = x ^ x;
            }
        "#;
        let violations = check_code(code);
        assert_eq!(violations.len(), 1);
        assert!(violations[0].message.contains("always zero"));
    }

    #[test]
    fn test_sub_same() {
        let code = r#"
            fn test() {
                let x = 5;
                let _y = x - x;
            }
        "#;
        let violations = check_code(code);
        assert_eq!(violations.len(), 1);
        assert!(violations[0].message.contains("always zero"));
    }
}
