//! E1303: Ignoring errors with let _ =
//!
//! Detects when `let _ =` is used to explicitly discard a Result, which silently
//! ignores errors that may be important to handle.

use crate::{checker::Checker, define_checker, violation::Violation};

use syn::visit::Visit;

define_checker! {
    /// Checker for E1303: Ignoring errors with let _ =
    E1303IgnoredErrors,
    code = "E1303",
    name = "Ignoring errors with let _ =",
    suggestions = "Handle errors properly with if/match, or use .ok() with a comment if intentionally ignoring",
    target_items = [Function],
    config_entry_name = "e1303_ignored_errors",
    config = E1303Config {
        enabled: bool = true,
        severity: crate::config::SeverityLevel = crate::config::SeverityLevel::High,
        categories: Vec<crate::config::CheckerCategory> = vec![crate::config::CheckerCategory::Operations],
    },
    check_item(self, item, file_path) {
        let mut visitor = IgnoredErrorVisitor {
            violations: Vec::new(),
            file_path,
            checker: self,
        };
        visitor.visit_item(item);
        Ok(visitor.violations)
    }
}

struct IgnoredErrorVisitor<'a> {
    violations: Vec<Violation>,
    file_path: &'a str,
    checker: &'a E1303IgnoredErrors,
}

impl<'a> Visit<'a> for IgnoredErrorVisitor<'a> {
    fn visit_local(&mut self, node: &'a syn::Local) {
        // Check for `let _ = ...` pattern
        if let syn::Pat::Wild(_) = &node.pat {
            if let Some(init) = &node.init {
                // Check if the expression is a potential Result-returning call
                if is_likely_result_expr(&init.expr) {
                    use syn::spanned::Spanned;
                    let start = node.span().start();
                    self.violations.push(
                        Violation::new(
                            self.checker.code(),
                            self.checker.name(),
                            self.checker.severity().into(),
                            "Using let _ = to ignore a Result. Errors should be handled or at least logged.",
                            self.file_path,
                            start.line,
                            start.column + 1,
                        )
                        .with_suggestion(self.checker.suggestions()),
                    );
                }
            }
        }

        syn::visit::visit_local(self, node);
    }
}

fn is_likely_result_expr(expr: &syn::Expr) -> bool {
    match expr {
        syn::Expr::Call(call) => {
            if let syn::Expr::Path(path) = &*call.func {
                let path_str = path
                    .path
                    .segments
                    .iter()
                    .map(|s| s.ident.to_string())
                    .collect::<Vec<_>>()
                    .join("::");

                // Common Result-returning std functions
                return path_str.contains("read")
                    || path_str.contains("write")
                    || path_str.contains("remove")
                    || path_str.contains("create")
                    || path_str.contains("open")
                    || path_str.contains("send")
                    || path_str.contains("fs::");
            }
            false
        }
        syn::Expr::MethodCall(method) => {
            let method_name = method.method.to_string();
            matches!(
                method_name.as_str(),
                "read" | "write" | "send" | "recv" | "connect" | "lock" | "try_lock"
            )
        }
        syn::Expr::Try(_) => true, // x? returns Result
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::checker::Checker;

    #[test]
    fn test_detects_ignored_result() {
        let code = r#"
            fn example() {
                let _ = std::fs::remove_file("temp.txt");
            }
        "#;

        let syntax = syn::parse_file(code).unwrap();
        let checker = E1303IgnoredErrors::default();

        let mut violations = Vec::new();
        for item in &syntax.items {
            violations.extend(checker.check_item(item, "test.rs").unwrap());
        }

        assert_eq!(violations.len(), 1);
    }

    #[test]
    fn test_handled_result_passes() {
        let code = r#"
            fn example() -> std::io::Result<()> {
                std::fs::remove_file("temp.txt")?;
                Ok(())
            }
        "#;

        let syntax = syn::parse_file(code).unwrap();
        let checker = E1303IgnoredErrors::default();

        let mut violations = Vec::new();
        for item in &syntax.items {
            violations.extend(checker.check_item(item, "test.rs").unwrap());
        }

        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_named_binding_passes() {
        let code = r#"
            fn example() {
                let result = std::fs::remove_file("temp.txt");
                // result is available for inspection
            }
        "#;

        let syntax = syn::parse_file(code).unwrap();
        let checker = E1303IgnoredErrors::default();

        let mut violations = Vec::new();
        for item in &syntax.items {
            violations.extend(checker.check_item(item, "test.rs").unwrap());
        }

        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_let_underscore_non_result_passes() {
        let code = r#"
            fn example() {
                let _ = 42; // Not a Result
            }
        "#;

        let syntax = syn::parse_file(code).unwrap();
        let checker = E1303IgnoredErrors::default();

        let mut violations = Vec::new();
        for item in &syntax.items {
            violations.extend(checker.check_item(item, "test.rs").unwrap());
        }

        assert_eq!(violations.len(), 0);
    }
}
