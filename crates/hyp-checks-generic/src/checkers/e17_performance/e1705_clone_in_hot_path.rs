//! E1705: Clone in hot path
//!
//! Detects clone() calls inside loops which may indicate performance issues
//! in frequently executed code paths.

use crate::{checker::Checker, define_checker, violation::Violation};

use syn::visit::Visit;

define_checker! {
    /// Checker for E1705: Clone in hot path
    E1705CloneInHotPath,
    code = "E1705",
    name = "Clone in hot path",
    suggestions = "Consider using references or Cow<T> to avoid cloning in loops",
    target_items = [Function],
    config_entry_name = "e1705_clone_in_hot_path",
    config = E1705Config {
        enabled: bool = true,
        severity: crate::config::SeverityLevel = crate::config::SeverityLevel::Low,
        categories: Vec<crate::config::CheckerCategory> = vec![crate::config::CheckerCategory::Complexity],
    },
    check_item(self, item, file_path) {
        let mut visitor = CloneVisitor {
            violations: Vec::new(),
            file_path,
            checker: self,
            loop_depth: 0,
        };
        visitor.visit_item(item);
        Ok(visitor.violations)
    }
}

struct CloneVisitor<'a> {
    violations: Vec<Violation>,
    file_path: &'a str,
    checker: &'a E1705CloneInHotPath,
    loop_depth: usize,
}

impl<'a> Visit<'a> for CloneVisitor<'a> {
    fn visit_expr_for_loop(&mut self, node: &'a syn::ExprForLoop) {
        self.loop_depth += 1;
        syn::visit::visit_expr_for_loop(self, node);
        self.loop_depth -= 1;
    }

    fn visit_expr_while(&mut self, node: &'a syn::ExprWhile) {
        self.loop_depth += 1;
        syn::visit::visit_expr_while(self, node);
        self.loop_depth -= 1;
    }

    fn visit_expr_loop(&mut self, node: &'a syn::ExprLoop) {
        self.loop_depth += 1;
        syn::visit::visit_expr_loop(self, node);
        self.loop_depth -= 1;
    }

    fn visit_expr_method_call(&mut self, node: &'a syn::ExprMethodCall) {
        if self.loop_depth > 0 {
            let method_name = node.method.to_string();

            if method_name == "clone" && node.args.is_empty() {
                use syn::spanned::Spanned;
                let start = node.span().start();
                let depth_msg = if self.loop_depth > 1 {
                    format!(" (nested {} levels deep)", self.loop_depth)
                } else {
                    String::new()
                };
                self.violations.push(
                    Violation::new(
                        self.checker.code(),
                        self.checker.name(),
                        self.checker.severity().into(),
                        format!(
                            "clone() called inside a loop{}. Consider using references or Cow<T>.",
                            depth_msg
                        ),
                        self.file_path,
                        start.line,
                        start.column + 1,
                    )
                    .with_suggestion(self.checker.suggestions()),
                );
            }
        }

        syn::visit::visit_expr_method_call(self, node);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detects_clone_in_loop() {
        let code = r#"
            fn example(data: &[String]) {
                for s in data {
                    let x = s.clone();
                }
            }
        "#;

        let syntax = syn::parse_file(code).unwrap();
        let checker = E1705CloneInHotPath::default();

        let mut violations = Vec::new();
        for item in &syntax.items {
            violations.extend(checker.check_item(item, "test.rs").unwrap());
        }

        assert_eq!(violations.len(), 1);
    }

    #[test]
    fn test_clone_outside_loop_passes() {
        let code = r#"
            fn example(s: String) {
                let x = s.clone();
            }
        "#;

        let syntax = syn::parse_file(code).unwrap();
        let checker = E1705CloneInHotPath::default();

        let mut violations = Vec::new();
        for item in &syntax.items {
            violations.extend(checker.check_item(item, "test.rs").unwrap());
        }

        assert_eq!(violations.len(), 0);
    }
}
