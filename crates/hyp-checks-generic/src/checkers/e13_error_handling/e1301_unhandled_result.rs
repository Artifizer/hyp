//! E1301: Unhandled Result values
//!
//! Detects function calls that return Result but the return value is not handled,
//! leading to silently ignored errors.

use crate::{checker::Checker, define_checker, violation::Violation};

use syn::visit::Visit;

define_checker! {
    /// Checker for E1301: Unhandled Result values
    E1301UnhandledResult,
    code = "E1301",
    name = "Unhandled Result value",
    suggestions = "Handle Results with match, if let, ?, or .unwrap() with a comment explaining why panicking is acceptable",
    target_items = [Function],
    config_entry_name = "e1301_unhandled_result",
    config = E1301Config {
        enabled: bool = true,
        severity: crate::config::SeverityLevel = crate::config::SeverityLevel::High,
        categories: Vec<crate::config::CheckerCategory> = vec![crate::config::CheckerCategory::Operations],
    },
    check_item(self, item, file_path) {
        let mut visitor = UnhandledResultVisitor {
            violations: Vec::new(),
            file_path,
            checker: self,
        };
        visitor.visit_item(item);
        Ok(visitor.violations)
    }
}

struct UnhandledResultVisitor<'a> {
    violations: Vec<Violation>,
    file_path: &'a str,
    checker: &'a E1301UnhandledResult,
}

impl<'a> Visit<'a> for UnhandledResultVisitor<'a> {
    fn visit_stmt(&mut self, node: &'a syn::Stmt) {
        // Check for expression statements (function calls as statements)
        // Both with and without semicolon - any expression used as a statement
        // where the result is not bound to a variable
        let expr_opt = match node {
            syn::Stmt::Expr(expr, _) => Some(expr),
            _ => None,
        };

        if let Some(expr) = expr_opt {
            // Check if this is a function/method call that might return Result
            if is_potential_result_call(expr) {
                use syn::spanned::Spanned;
                let start = expr.span().start();
                self.violations.push(
                    Violation::new(
                        self.checker.code(),
                        self.checker.name(),
                        self.checker.severity().into(),
                        "Function call may return Result but the result is not handled.",
                        self.file_path,
                        start.line,
                        start.column + 1,
                    )
                    .with_suggestion(self.checker.suggestions()),
                );
            }
        }

        syn::visit::visit_stmt(self, node);
    }
}

fn is_potential_result_call(expr: &syn::Expr) -> bool {
    match expr {
        syn::Expr::Call(call) => {
            // Check for common Result-returning functions
            if let syn::Expr::Path(path) = &*call.func {
                let path_str = path
                    .path
                    .segments
                    .iter()
                    .map(|s| s.ident.to_string())
                    .collect::<Vec<_>>()
                    .join("::");

                // Common std functions that return Result
                return path_str.contains("read")
                    || path_str.contains("write")
                    || path_str.contains("open")
                    || path_str.contains("create")
                    || path_str.contains("connect")
                    || path_str.contains("send")
                    || path_str.contains("recv")
                    || path_str.contains("parse")
                    || path_str.contains("remove")
                    || path_str.contains("rename")
                    || path_str.ends_with("fs::read")
                    || path_str.ends_with("fs::write")
                    || path_str.ends_with("fs::read_to_string");
            }
            false
        }
        syn::Expr::MethodCall(method) => {
            let method_name = method.method.to_string();
            // Common methods that return Result
            matches!(
                method_name.as_str(),
                "read" | "write" | "read_to_string" | "read_line" | "flush"
                    | "send" | "recv" | "connect" | "accept"
                    | "parse" | "try_into" | "try_from"
                    | "lock" | "try_lock"
            )
        }
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::checker::Checker;

    #[test]
    fn test_detects_unhandled_result() {
        let code = r#"
            fn example() {
                std::fs::read_to_string("config.txt");
            }
        "#;

        let syntax = syn::parse_file(code).unwrap();
        let checker = E1301UnhandledResult::default();

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
                let _content = std::fs::read_to_string("config.txt")?;
                Ok(())
            }
        "#;

        let syntax = syn::parse_file(code).unwrap();
        let checker = E1301UnhandledResult::default();

        let mut violations = Vec::new();
        for item in &syntax.items {
            violations.extend(checker.check_item(item, "test.rs").unwrap());
        }

        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_let_binding_passes() {
        let code = r#"
            fn example() {
                let result = std::fs::read_to_string("config.txt");
            }
        "#;

        let syntax = syn::parse_file(code).unwrap();
        let checker = E1301UnhandledResult::default();

        let mut violations = Vec::new();
        for item in &syntax.items {
            violations.extend(checker.check_item(item, "test.rs").unwrap());
        }

        assert_eq!(violations.len(), 0);
    }
}
