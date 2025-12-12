//! E1306: Swallowing errors without logging
//!
//! Detects when `.ok()` is called to convert Result to Option without logging
//! the error, causing silent failures that are hard to debug.

use crate::{checker::Checker, define_checker, violation::Violation};

use syn::visit::Visit;

define_checker! {
    /// Checker for E1306: Swallowing errors without logging
    E1306SwallowedErrors,
    code = "E1306",
    name = "Swallowing errors without logging",
    suggestions = "Log errors before converting to Option using inspect_err(), or return Result to preserve error info",
    target_items = [Function],
    config_entry_name = "e1306_swallowed_errors",
    config = E1306Config {
        enabled: bool = true,
        severity: crate::config::SeverityLevel = crate::config::SeverityLevel::Medium,
        categories: Vec<crate::config::CheckerCategory> = vec![crate::config::CheckerCategory::Operations],
    },
    check_item(self, item, file_path) {
        let mut visitor = SwallowedErrorVisitor {
            violations: Vec::new(),
            file_path,
            checker: self,
        };
        visitor.visit_item(item);
        Ok(visitor.violations)
    }
}

struct SwallowedErrorVisitor<'a> {
    violations: Vec<Violation>,
    file_path: &'a str,
    checker: &'a E1306SwallowedErrors,
}

impl<'a> Visit<'a> for SwallowedErrorVisitor<'a> {
    fn visit_expr_method_call(&mut self, node: &'a syn::ExprMethodCall) {
        let method_name = node.method.to_string();

        // Check for .ok() calls without preceding inspect_err()
        if method_name == "ok" && node.args.is_empty() {
            // Check if the receiver is a Result-returning expression
            if is_likely_result_returning(&node.receiver) {
                // Check if there's no inspect_err() in the chain
                if !has_inspect_err_before(&node.receiver) {
                    use syn::spanned::Spanned;
                    let start = node.span().start();
                    self.violations.push(
                        Violation::new(
                            self.checker.code(),
                            self.checker.name(),
                            self.checker.severity().into(),
                            "Converting Result to Option with .ok() loses error information. Log errors before discarding.",
                            self.file_path,
                            start.line,
                            start.column + 1,
                        )
                        .with_suggestion(self.checker.suggestions()),
                    );
                }
            }
        }

        syn::visit::visit_expr_method_call(self, node);
    }
}

fn is_likely_result_returning(expr: &syn::Expr) -> bool {
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

                return path_str.contains("read")
                    || path_str.contains("write")
                    || path_str.contains("parse")
                    || path_str.contains("fs::");
            }
            false
        }
        syn::Expr::MethodCall(method) => {
            let method_name = method.method.to_string();
            matches!(
                method_name.as_str(),
                "read" | "write" | "parse" | "read_to_string" | "send" | "recv" | "lock"
            )
        }
        _ => false,
    }
}

fn has_inspect_err_before(expr: &syn::Expr) -> bool {
    if let syn::Expr::MethodCall(method) = expr {
        if method.method == "inspect_err" || method.method == "map_err" {
            return true;
        }
        return has_inspect_err_before(&method.receiver);
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::checker::Checker;

    #[test]
    fn test_detects_swallowed_error() {
        let code = r#"
            fn example() -> Option<String> {
                std::fs::read_to_string("file.txt").ok()
            }
        "#;

        let syntax = syn::parse_file(code).unwrap();
        let checker = E1306SwallowedErrors::default();

        let mut violations = Vec::new();
        for item in &syntax.items {
            violations.extend(checker.check_item(item, "test.rs").unwrap());
        }

        assert_eq!(violations.len(), 1);
    }

    #[test]
    fn test_inspect_err_before_ok_passes() {
        let code = r#"
            fn example() -> Option<String> {
                std::fs::read_to_string("file.txt")
                    .inspect_err(|e| eprintln!("Error: {}", e))
                    .ok()
            }
        "#;

        let syntax = syn::parse_file(code).unwrap();
        let checker = E1306SwallowedErrors::default();

        let mut violations = Vec::new();
        for item in &syntax.items {
            violations.extend(checker.check_item(item, "test.rs").unwrap());
        }

        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_returning_result_passes() {
        let code = r#"
            fn example() -> std::io::Result<String> {
                std::fs::read_to_string("file.txt")
            }
        "#;

        let syntax = syn::parse_file(code).unwrap();
        let checker = E1306SwallowedErrors::default();

        let mut violations = Vec::new();
        for item in &syntax.items {
            violations.extend(checker.check_item(item, "test.rs").unwrap());
        }

        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_ok_on_option_passes() {
        let code = r#"
            fn example(opt: Option<i32>) -> bool {
                opt.is_some()
            }
        "#;

        let syntax = syn::parse_file(code).unwrap();
        let checker = E1306SwallowedErrors::default();

        let mut violations = Vec::new();
        for item in &syntax.items {
            violations.extend(checker.check_item(item, "test.rs").unwrap());
        }

        assert_eq!(violations.len(), 0);
    }
}
