//! E1508: Sleep instead of synchronization
//!
//! Detects use of thread::sleep or tokio::time::sleep for synchronization
//! instead of proper synchronization primitives.

use crate::{checker::Checker, define_checker, violation::Violation};

use syn::visit::Visit;

define_checker! {
    /// Checker for E1508: Sleep instead of synchronization
    E1508SleepInsteadOfSync,
    code = "E1508",
    name = "Sleep instead of synchronization",
    suggestions = "Use proper synchronization primitives (Mutex, RwLock, channels, condvars) instead of sleep",
    target_items = [Function],
    config_entry_name = "e1508_sleep_instead_of_sync",
    config = E1508Config {
        enabled: bool = true,
        severity: crate::config::SeverityLevel = crate::config::SeverityLevel::Low,
        categories: Vec<crate::config::CheckerCategory> = vec![crate::config::CheckerCategory::Operations],
    },
    check_item(self, item, file_path) {
        let mut visitor = SleepVisitor {
            violations: Vec::new(),
            file_path,
            checker: self,
        };
        visitor.visit_item(item);
        Ok(visitor.violations)
    }
}

struct SleepVisitor<'a> {
    violations: Vec<Violation>,
    file_path: &'a str,
    checker: &'a E1508SleepInsteadOfSync,
}

impl<'a> Visit<'a> for SleepVisitor<'a> {
    fn visit_expr_call(&mut self, node: &'a syn::ExprCall) {
        if let syn::Expr::Path(path) = &*node.func {
            let path_str = path
                .path
                .segments
                .iter()
                .map(|s| s.ident.to_string())
                .collect::<Vec<_>>()
                .join("::");

            // Check for thread::sleep, std::thread::sleep, tokio::time::sleep, etc.
            if path_str.ends_with("sleep") &&
               (path_str.contains("thread") || path_str.contains("time")) {
                use syn::spanned::Spanned;
                let start = node.span().start();
                self.violations.push(
                    Violation::new(
                        self.checker.code(),
                        self.checker.name(),
                        self.checker.severity().into(),
                        "Using sleep for synchronization is unreliable. Consider using proper synchronization primitives.",
                        self.file_path,
                        start.line,
                        start.column + 1,
                    )
                    .with_suggestion(self.checker.suggestions()),
                );
            }
        }

        syn::visit::visit_expr_call(self, node);
    }

    fn visit_expr_method_call(&mut self, node: &'a syn::ExprMethodCall) {
        let method_name = node.method.to_string();
        if method_name == "sleep" {
            use syn::spanned::Spanned;
            let start = node.span().start();
            self.violations.push(
                Violation::new(
                    self.checker.code(),
                    self.checker.name(),
                    self.checker.severity().into(),
                    "Using sleep for synchronization is unreliable. Consider using proper synchronization primitives.",
                    self.file_path,
                    start.line,
                    start.column + 1,
                )
                .with_suggestion(self.checker.suggestions()),
            );
        }

        syn::visit::visit_expr_method_call(self, node);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detects_thread_sleep() {
        let code = r#"
            use std::thread;
            use std::time::Duration;

            fn example() {
                thread::sleep(Duration::from_millis(100));
            }
        "#;

        let syntax = syn::parse_file(code).unwrap();
        let checker = E1508SleepInsteadOfSync::default();

        let mut violations = Vec::new();
        for item in &syntax.items {
            violations.extend(checker.check_item(item, "test.rs").unwrap());
        }

        assert_eq!(violations.len(), 1);
    }

    #[test]
    fn test_no_sleep_passes() {
        let code = r#"
            use std::sync::Mutex;

            fn example(m: &Mutex<i32>) {
                let guard = m.lock().unwrap();
                println!("{}", *guard);
            }
        "#;

        let syntax = syn::parse_file(code).unwrap();
        let checker = E1508SleepInsteadOfSync::default();

        let mut violations = Vec::new();
        for item in &syntax.items {
            violations.extend(checker.check_item(item, "test.rs").unwrap());
        }

        assert_eq!(violations.len(), 0);
    }
}
