//! E1706: Non-tail recursion
//!
//! Detects recursive functions where the recursive call is not in tail position,
//! meaning the function does work after the recursive call returns. This can
//! cause stack overflow for deep recursion since Rust doesn't guarantee TCO.

use crate::{define_checker, violation::Violation};

use syn::visit::Visit;

define_checker! {
    /// Checker for E1706: Non-tail recursion
    E1706NonTailRecursion,
    code = "E1706",
    name = "Non-tail recursive function",
    suggestions = "Convert to iteration, use an accumulator for tail recursion, or use trampolining",
    target_items = [Function],
    config_entry_name = "e1706_non_tail_recursion",
    config = E1706Config {
        enabled: bool = true,
        severity: crate::config::SeverityLevel = crate::config::SeverityLevel::Medium,
        categories: Vec<crate::config::CheckerCategory> = vec![crate::config::CheckerCategory::Complexity],
    },
    check_item(self, item, file_path) {
        let mut violations = Vec::new();

        if let syn::Item::Fn(func) = item {
            let fn_name = func.sig.ident.to_string();
            let mut visitor = RecursionVisitor {
                fn_name: &fn_name,
                has_non_tail_recursion: false,
                in_tail_position: true,
            };

            // Visit the function body
            for stmt in &func.block.stmts {
                // Last statement might be in tail position
                let is_last = std::ptr::eq(stmt, func.block.stmts.last().unwrap());
                visitor.in_tail_position = is_last;
                visitor.visit_stmt(stmt);
            }

            if visitor.has_non_tail_recursion {
                let start = func.sig.ident.span().start();
                violations.push(
                    Violation::new(
                        self.code(),
                        self.name(),
                        self.severity().into(),
                        format!(
                            "Function '{}' has non-tail recursive call. This can cause stack overflow for deep recursion.",
                            fn_name
                        ),
                        file_path,
                        start.line,
                        start.column + 1,
                    )
                    .with_suggestion(self.suggestions()),
                );
            }
        }

        Ok(violations)
    }
}

struct RecursionVisitor<'a> {
    fn_name: &'a str,
    has_non_tail_recursion: bool,
    in_tail_position: bool,
}

impl<'ast> Visit<'ast> for RecursionVisitor<'_> {
    fn visit_expr_call(&mut self, node: &'ast syn::ExprCall) {
        if let syn::Expr::Path(path) = &*node.func {
            let path_str = path
                .path
                .segments
                .iter()
                .map(|s| s.ident.to_string())
                .collect::<Vec<_>>()
                .join("::");

            if path_str == self.fn_name {
                // This is a recursive call - check if it's in tail position
                if !self.in_tail_position {
                    self.has_non_tail_recursion = true;
                }
            }
        }

        // Recursive calls in arguments are definitely not in tail position
        let was_tail = self.in_tail_position;
        self.in_tail_position = false;
        syn::visit::visit_expr_call(self, node);
        self.in_tail_position = was_tail;
    }

    fn visit_expr_binary(&mut self, node: &'ast syn::ExprBinary) {
        // Binary operations containing recursive calls are not tail recursive
        let was_tail = self.in_tail_position;
        self.in_tail_position = false;
        syn::visit::visit_expr_binary(self, node);
        self.in_tail_position = was_tail;
    }

    fn visit_expr_if(&mut self, node: &'ast syn::ExprIf) {
        // Condition is not in tail position
        let was_tail = self.in_tail_position;
        self.in_tail_position = false;
        self.visit_expr(&node.cond);

        // Then and else branches maintain tail position
        self.in_tail_position = was_tail;
        self.visit_block(&node.then_branch);

        if let Some((_, else_branch)) = &node.else_branch {
            self.in_tail_position = was_tail;
            self.visit_expr(else_branch);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::checker::Checker;

    #[test]
    fn test_detects_non_tail_recursion() {
        let code = r#"
            fn factorial(n: u64) -> u64 {
                if n == 0 {
                    1
                } else {
                    n * factorial(n - 1)
                }
            }
        "#;

        let syntax = syn::parse_file(code).unwrap();
        let checker = E1706NonTailRecursion::default();

        let mut violations = Vec::new();
        for item in &syntax.items {
            violations.extend(checker.check_item(item, "test.rs").unwrap());
        }

        assert_eq!(violations.len(), 1);
    }

    #[test]
    fn test_tail_recursion_passes() {
        let code = r#"
            fn factorial_tail(n: u64, acc: u64) -> u64 {
                if n == 0 {
                    acc
                } else {
                    factorial_tail(n - 1, n * acc)
                }
            }
        "#;

        let syntax = syn::parse_file(code).unwrap();
        let checker = E1706NonTailRecursion::default();

        let mut violations = Vec::new();
        for item in &syntax.items {
            violations.extend(checker.check_item(item, "test.rs").unwrap());
        }

        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_no_recursion_passes() {
        let code = r#"
            fn add(a: i32, b: i32) -> i32 {
                a + b
            }
        "#;

        let syntax = syn::parse_file(code).unwrap();
        let checker = E1706NonTailRecursion::default();

        let mut violations = Vec::new();
        for item in &syntax.items {
            violations.extend(checker.check_item(item, "test.rs").unwrap());
        }

        assert_eq!(violations.len(), 0);
    }
}
