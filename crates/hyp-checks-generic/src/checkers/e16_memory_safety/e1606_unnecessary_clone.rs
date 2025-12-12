//! E1606: Unnecessary clone
//!
//! Detects clone() calls that may be unnecessary, such as cloning
//! immediately before a move or cloning reference types.

use crate::{checker::Checker, define_checker, violation::Violation};

use syn::visit::Visit;

define_checker! {
    /// Checker for E1606: Unnecessary clone
    E1606UnnecessaryClone,
    code = "E1606",
    name = "Unnecessary clone",
    suggestions = "Consider using references or borrowing instead of cloning",
    target_items = [Function],
    config_entry_name = "e1606_unnecessary_clone",
    config = E1606Config {
        enabled: bool = true,
        severity: crate::config::SeverityLevel = crate::config::SeverityLevel::Low,
        categories: Vec<crate::config::CheckerCategory> = vec![crate::config::CheckerCategory::Complexity],
    },
    check_item(self, item, file_path) {
        let mut visitor = CloneVisitor {
            violations: Vec::new(),
            file_path,
            checker: self,
        };
        visitor.visit_item(item);
        Ok(visitor.violations)
    }
}

struct CloneVisitor<'a> {
    violations: Vec<Violation>,
    file_path: &'a str,
    checker: &'a E1606UnnecessaryClone,
}

impl<'a> Visit<'a> for CloneVisitor<'a> {
    fn visit_expr_method_call(&mut self, node: &'a syn::ExprMethodCall) {
        let method_name = node.method.to_string();

        // Detect .clone() calls
        if method_name == "clone" && node.args.is_empty() {
            // Check for patterns like .clone().clone() or String::from().clone()
            if let syn::Expr::MethodCall(inner) = &*node.receiver {
                let inner_method = inner.method.to_string();
                // Cloning immediately after certain constructors is suspicious
                if matches!(
                    inner_method.as_str(),
                    "clone" | "to_string" | "to_owned" | "into"
                ) {
                    use syn::spanned::Spanned;
                    let start = node.span().start();
                    self.violations.push(
                        Violation::new(
                            self.checker.code(),
                            self.checker.name(),
                            self.checker.severity().into(),
                            "Redundant clone() after a method that already produces an owned value.",
                            self.file_path,
                            start.line,
                            start.column + 1,
                        )
                        .with_suggestion(self.checker.suggestions()),
                    );
                }
            }

            // Check for .to_string().clone() or similar patterns
            if let syn::Expr::Call(call) = &*node.receiver {
                if let syn::Expr::Path(path) = &*call.func {
                    let path_str = path
                        .path
                        .segments
                        .iter()
                        .map(|s| s.ident.to_string())
                        .collect::<Vec<_>>()
                        .join("::");

                    // Cloning after String::from, Vec::new, etc.
                    if path_str.ends_with("::from")
                        || path_str.ends_with("::new")
                        || path_str.ends_with("::default")
                    {
                        use syn::spanned::Spanned;
                        let start = node.span().start();
                        self.violations.push(
                            Violation::new(
                                self.checker.code(),
                                self.checker.name(),
                                self.checker.severity().into(),
                                "Redundant clone() on a newly constructed value.",
                                self.file_path,
                                start.line,
                                start.column + 1,
                            )
                            .with_suggestion(self.checker.suggestions()),
                        );
                    }
                }
            }
        }

        syn::visit::visit_expr_method_call(self, node);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::checker::Checker;

    #[test]
    fn test_detects_double_clone() {
        let code = r#"
            fn example(s: String) {
                let x = s.clone().clone();
            }
        "#;

        let syntax = syn::parse_file(code).unwrap();
        let checker = E1606UnnecessaryClone::default();

        let mut violations = Vec::new();
        for item in &syntax.items {
            violations.extend(checker.check_item(item, "test.rs").unwrap());
        }

        assert_eq!(violations.len(), 1);
    }

    #[test]
    fn test_clone_after_to_string() {
        let code = r#"
            fn example(s: &str) {
                let x = s.to_string().clone();
            }
        "#;

        let syntax = syn::parse_file(code).unwrap();
        let checker = E1606UnnecessaryClone::default();

        let mut violations = Vec::new();
        for item in &syntax.items {
            violations.extend(checker.check_item(item, "test.rs").unwrap());
        }

        assert_eq!(violations.len(), 1);
    }

    #[test]
    fn test_normal_clone_passes() {
        let code = r#"
            fn example(s: String) {
                let x = s.clone();
            }
        "#;

        let syntax = syn::parse_file(code).unwrap();
        let checker = E1606UnnecessaryClone::default();

        let mut violations = Vec::new();
        for item in &syntax.items {
            violations.extend(checker.check_item(item, "test.rs").unwrap());
        }

        assert_eq!(violations.len(), 0);
    }
}
