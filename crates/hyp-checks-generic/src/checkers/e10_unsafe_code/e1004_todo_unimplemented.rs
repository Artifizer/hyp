//! E1004: `todo!()` and `unimplemented!()` macros in production code
//!
//! Detects `todo!()` and `unimplemented!()` macros which panic at runtime.
//! These are placeholder macros that should be replaced with actual implementations
//! or proper error handling before deployment.
//!
//! LLMs frequently leave these placeholders in generated code.

use crate::{checker::Checker, define_checker, violation::Violation};

use syn::{spanned::Spanned, visit::Visit, Macro};

define_checker! {
    /// Checker for E1004: todo!/unimplemented! in production code
    E1004TodoUnimplemented,
    code = "E1004",
    name = "todo!/unimplemented! macro in code",
    suggestions = "Replace with actual implementation or return Result/Option for incomplete features",
    target_items = [Function],
    config_entry_name = "e1004_todo_unimplemented",
    config = E1004Config {
        enabled: bool = true,
        severity: crate::config::SeverityLevel = crate::config::SeverityLevel::High,
        categories: Vec<crate::config::CheckerCategory> = vec![crate::config::CheckerCategory::Operations],
        /// Also detect unreachable!() macro
        detect_unreachable: bool = false,
    },
    check_item(self, item, file_path) {
        let mut visitor = TodoVisitor {
            violations: Vec::new(),
            file_path,
            checker: self,
        };
        visitor.visit_item(item);
        Ok(visitor.violations)
    }
}

struct TodoVisitor<'a> {
    violations: Vec<Violation>,
    file_path: &'a str,
    checker: &'a E1004TodoUnimplemented,
}

impl<'a> TodoVisitor<'a> {
    fn check_macro(&mut self, mac: &Macro) {
        let macro_name = mac
            .path
            .segments
            .last()
            .map(|seg| seg.ident.to_string())
            .unwrap_or_default();

        let is_todo = macro_name == "todo" || macro_name == "unimplemented";
        let is_unreachable = macro_name == "unreachable" && self.checker.config.detect_unreachable;

        if is_todo || is_unreachable {
            let span = mac
                .path
                .segments
                .first()
                .map(|seg| seg.ident.span())
                .unwrap_or_else(|| mac.path.span());

            let message = if macro_name == "todo" {
                "todo!() is a placeholder that panics at runtime. Replace with actual implementation."
            } else if macro_name == "unimplemented" {
                "unimplemented!() panics at runtime. Implement the feature or return an error."
            } else {
                "unreachable!() panics if reached. Ensure code path is truly unreachable."
            };

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
    }
}

impl<'a> Visit<'a> for TodoVisitor<'a> {
    fn visit_stmt(&mut self, node: &'a syn::Stmt) {
        if let syn::Stmt::Macro(stmt_macro) = node {
            self.check_macro(&stmt_macro.mac);
        }
        syn::visit::visit_stmt(self, node);
    }

    fn visit_expr(&mut self, node: &'a syn::Expr) {
        if let syn::Expr::Macro(expr_macro) = node {
            self.check_macro(&expr_macro.mac);
        }
        syn::visit::visit_expr(self, node);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::checker::Checker;

    fn check_code(code: &str) -> Vec<Violation> {
        let checker = E1004TodoUnimplemented::default();
        let file = syn::parse_file(code).expect("Failed to parse");
        let mut violations = Vec::new();
        for item in &file.items {
            violations.extend(checker.check_item(item, "test.rs").unwrap());
        }
        violations
    }

    #[test]
    fn test_detects_todo() {
        let code = r#"
            fn example() {
                todo!();
            }
        "#;
        let violations = check_code(code);
        assert_eq!(violations.len(), 1);
        assert!(violations[0].message.contains("todo!()"));
    }

    #[test]
    fn test_detects_todo_with_message() {
        let code = r#"
            fn example() {
                todo!("implement this later");
            }
        "#;
        let violations = check_code(code);
        assert_eq!(violations.len(), 1);
    }

    #[test]
    fn test_detects_unimplemented() {
        let code = r#"
            fn example() {
                unimplemented!();
            }
        "#;
        let violations = check_code(code);
        assert_eq!(violations.len(), 1);
        assert!(violations[0].message.contains("unimplemented!()"));
    }

    #[test]
    fn test_no_false_positive_on_other_macros() {
        let code = r#"
            fn example() {
                println!("hello");
                assert!(true);
            }
        "#;
        let violations = check_code(code);
        assert!(violations.is_empty());
    }

    #[test]
    fn test_detects_in_match_arm() {
        let code = r#"
            fn example(x: i32) -> i32 {
                match x {
                    1 => 1,
                    _ => todo!(),
                }
            }
        "#;
        let violations = check_code(code);
        assert_eq!(violations.len(), 1);
    }
}
