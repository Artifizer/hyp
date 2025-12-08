//! E1503: Lock poisoning mishandled
//!
//! Detects when .lock().unwrap() is used on a Mutex, which will panic if the mutex
//! is poisoned (another thread panicked while holding the lock).

use crate::{checker::Checker, define_checker, violation::Violation};

use syn::visit::Visit;

define_checker! {
    /// Checker for E1503: Lock poisoning mishandled
    E1503LockPoisoning,
    code = "E1503",
    name = "Lock poisoning not handled",
    suggestions = "Use lock().unwrap_or_else(|e| e.into_inner()) to recover from poisoned locks, or handle explicitly",
    target_items = [Function],
    config_entry_name = "e1503_lock_poisoning",
    config = E1503Config {
        enabled: bool = true,
        severity: crate::config::SeverityLevel = crate::config::SeverityLevel::Medium,
        categories: Vec<crate::config::CheckerCategory> = vec![crate::config::CheckerCategory::Operations],
    },
    check_item(self, item, file_path) {
        let mut visitor = LockPoisoningVisitor {
            violations: Vec::new(),
            file_path,
            checker: self,
        };
        visitor.visit_item(item);
        Ok(visitor.violations)
    }
}

struct LockPoisoningVisitor<'a> {
    violations: Vec<Violation>,
    file_path: &'a str,
    checker: &'a E1503LockPoisoning,
}

impl<'a> Visit<'a> for LockPoisoningVisitor<'a> {
    fn visit_expr_method_call(&mut self, node: &'a syn::ExprMethodCall) {
        let method_name = node.method.to_string();

        // Check for .unwrap() or .expect() after .lock()
        if matches!(method_name.as_str(), "unwrap" | "expect") {
            if let syn::Expr::MethodCall(inner) = &*node.receiver {
                let inner_method = inner.method.to_string();
                if inner_method == "lock" || inner_method == "try_lock" {
                    use syn::spanned::Spanned;
                    let start = node.span().start();
                    self.violations.push(
                        Violation::new(
                            self.checker.code(),
                            self.checker.name(),
                            self.checker.severity().into(),
                            format!(
                                "Using {}() on lock() will panic if the Mutex is poisoned. Handle poisoning explicitly.",
                                method_name
                            ),
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::checker::Checker;

    #[test]
    fn test_detects_lock_unwrap() {
        let code = r#"
            use std::sync::Mutex;

            fn example(m: &Mutex<i32>) {
                let guard = m.lock().unwrap();
            }
        "#;

        let syntax = syn::parse_file(code).unwrap();
        let checker = E1503LockPoisoning::default();

        let mut violations = Vec::new();
        for item in &syntax.items {
            violations.extend(checker.check_item(item, "test.rs").unwrap());
        }

        assert_eq!(violations.len(), 1);
    }

    #[test]
    fn test_detects_lock_expect() {
        let code = r#"
            use std::sync::Mutex;

            fn example(m: &Mutex<i32>) {
                let guard = m.lock().expect("lock failed");
            }
        "#;

        let syntax = syn::parse_file(code).unwrap();
        let checker = E1503LockPoisoning::default();

        let mut violations = Vec::new();
        for item in &syntax.items {
            violations.extend(checker.check_item(item, "test.rs").unwrap());
        }

        assert_eq!(violations.len(), 1);
    }

    #[test]
    fn test_proper_handling_passes() {
        let code = r#"
            use std::sync::Mutex;

            fn example(m: &Mutex<i32>) {
                let guard = m.lock().unwrap_or_else(|e| e.into_inner());
            }
        "#;

        let syntax = syn::parse_file(code).unwrap();
        let checker = E1503LockPoisoning::default();

        let mut violations = Vec::new();
        for item in &syntax.items {
            violations.extend(checker.check_item(item, "test.rs").unwrap());
        }

        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_try_lock_unwrap() {
        let code = r#"
            use std::sync::Mutex;

            fn example(m: &Mutex<i32>) {
                let guard = m.try_lock().unwrap();
            }
        "#;

        let syntax = syn::parse_file(code).unwrap();
        let checker = E1503LockPoisoning::default();

        let mut violations = Vec::new();
        for item in &syntax.items {
            violations.extend(checker.check_item(item, "test.rs").unwrap());
        }

        assert_eq!(violations.len(), 1);
    }
}
