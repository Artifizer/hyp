//! E1703: String concatenation in loop
//!
//! Detects string concatenation patterns inside loops that could be
//! optimized with String::with_capacity or push_str.

use crate::{checker::Checker, define_checker, violation::Violation};

use syn::visit::Visit;

define_checker! {
    /// Checker for E1703: String concatenation in loop
    E1703StringConcatInLoop,
    code = "E1703",
    name = "String concatenation in loop",
    suggestions = "Use String::with_capacity() or push_str() to avoid repeated allocations",
    target_items = [Function],
    config_entry_name = "e1703_string_concat_in_loop",
    config = E1703Config {
        enabled: bool = true,
        severity: crate::config::SeverityLevel = crate::config::SeverityLevel::Medium,
        categories: Vec<crate::config::CheckerCategory> = vec![crate::config::CheckerCategory::Complexity],
    },
    check_item(self, item, file_path) {
        let mut visitor = LoopVisitor {
            violations: Vec::new(),
            file_path,
            checker: self,
            in_loop: false,
        };
        visitor.visit_item(item);
        Ok(visitor.violations)
    }
}

struct LoopVisitor<'a> {
    violations: Vec<Violation>,
    file_path: &'a str,
    checker: &'a E1703StringConcatInLoop,
    in_loop: bool,
}

impl<'a> Visit<'a> for LoopVisitor<'a> {
    fn visit_expr_for_loop(&mut self, node: &'a syn::ExprForLoop) {
        let was_in_loop = self.in_loop;
        self.in_loop = true;
        syn::visit::visit_expr_for_loop(self, node);
        self.in_loop = was_in_loop;
    }

    fn visit_expr_while(&mut self, node: &'a syn::ExprWhile) {
        let was_in_loop = self.in_loop;
        self.in_loop = true;
        syn::visit::visit_expr_while(self, node);
        self.in_loop = was_in_loop;
    }

    fn visit_expr_loop(&mut self, node: &'a syn::ExprLoop) {
        let was_in_loop = self.in_loop;
        self.in_loop = true;
        syn::visit::visit_expr_loop(self, node);
        self.in_loop = was_in_loop;
    }

    fn visit_expr_binary(&mut self, node: &'a syn::ExprBinary) {
        if self.in_loop {
            // Check for += with string
            if let syn::BinOp::AddAssign(_) = node.op {
                use syn::spanned::Spanned;
                let start = node.span().start();
                self.violations.push(
                    Violation::new(
                        self.checker.code(),
                        self.checker.name(),
                        self.checker.severity().into(),
                        "String concatenation with += inside a loop causes repeated allocations.",
                        self.file_path,
                        start.line,
                        start.column + 1,
                    )
                    .with_suggestion(self.checker.suggestions()),
                );
            }
        }

        syn::visit::visit_expr_binary(self, node);
    }

    fn visit_expr_call(&mut self, node: &'a syn::ExprCall) {
        if self.in_loop {
            if let syn::Expr::Path(path) = &*node.func {
                let path_str = path
                    .path
                    .segments
                    .iter()
                    .map(|s| s.ident.to_string())
                    .collect::<Vec<_>>()
                    .join("::");

                // Check for format! macro usage in loop
                if path_str == "format" {
                    use syn::spanned::Spanned;
                    let start = node.span().start();
                    self.violations.push(
                        Violation::new(
                            self.checker.code(),
                            self.checker.name(),
                            self.checker.severity().into(),
                            "format!() in a loop creates new Strings. Consider using write! with a pre-allocated buffer.",
                            self.file_path,
                            start.line,
                            start.column + 1,
                        )
                        .with_suggestion(self.checker.suggestions()),
                    );
                }
            }
        }

        syn::visit::visit_expr_call(self, node);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detects_concat_in_loop() {
        let code = r#"
            fn example() {
                let mut s = String::new();
                for i in 0..10 {
                    s += "x";
                }
            }
        "#;

        let syntax = syn::parse_file(code).unwrap();
        let checker = E1703StringConcatInLoop::default();

        let mut violations = Vec::new();
        for item in &syntax.items {
            violations.extend(checker.check_item(item, "test.rs").unwrap());
        }

        assert_eq!(violations.len(), 1);
    }

    #[test]
    fn test_concat_outside_loop_passes() {
        let code = r#"
            fn example() {
                let mut s = String::new();
                s += "x";
            }
        "#;

        let syntax = syn::parse_file(code).unwrap();
        let checker = E1703StringConcatInLoop::default();

        let mut violations = Vec::new();
        for item in &syntax.items {
            violations.extend(checker.check_item(item, "test.rs").unwrap());
        }

        assert_eq!(violations.len(), 0);
    }
}
