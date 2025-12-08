//! E1506: Deadlock from lock ordering
//!
//! Detects potential deadlock scenarios where multiple locks are acquired
//! in a function without a consistent ordering pattern.

use crate::{define_checker, violation::Violation};

use syn::visit::Visit;

define_checker! {
    /// Checker for E1506: Deadlock from lock ordering
    E1506DeadlockLockOrdering,
    code = "E1506",
    name = "Potential deadlock from lock ordering",
    suggestions = "Always acquire locks in a consistent order across all code paths to prevent deadlocks",
    target_items = [Function],
    config_entry_name = "e1506_deadlock_lock_ordering",
    config = E1506Config {
        enabled: bool = true,
        severity: crate::config::SeverityLevel = crate::config::SeverityLevel::High,
        categories: Vec<crate::config::CheckerCategory> = vec![crate::config::CheckerCategory::Operations],
    },
    check_item(self, item, file_path) {
        let mut visitor = LockOrderingVisitor {
            violations: Vec::new(),
            file_path,
            checker: self,
            lock_count: 0,
            locked_vars: std::collections::HashSet::new(),
        };
        visitor.visit_item(item);

        // If multiple locks are acquired in a single function, warn about potential deadlock
        if visitor.lock_count > 1 {
            if let syn::Item::Fn(func) = item {
                let start = func.sig.ident.span().start();
                visitor.violations.push(
                    Violation::new(
                        visitor.checker.code(),
                        visitor.checker.name(),
                        visitor.checker.severity().into(),
                        format!(
                            "Function '{}' acquires {} locks. Ensure consistent ordering to prevent deadlocks.",
                            func.sig.ident,
                            visitor.lock_count
                        ),
                        visitor.file_path,
                        start.line,
                        start.column + 1,
                    )
                    .with_suggestion(visitor.checker.suggestions()),
                );
            }
        }

        Ok(visitor.violations)
    }
}

struct LockOrderingVisitor<'a> {
    violations: Vec<Violation>,
    file_path: &'a str,
    checker: &'a E1506DeadlockLockOrdering,
    lock_count: usize,
    locked_vars: std::collections::HashSet<String>,
}

impl<'a> Visit<'a> for LockOrderingVisitor<'a> {
    fn visit_expr_method_call(&mut self, node: &'a syn::ExprMethodCall) {
        let method_name = node.method.to_string();

        // Count lock acquisitions on DIFFERENT variables only
        if matches!(method_name.as_str(), "lock" | "read" | "write" | "try_lock" | "try_read" | "try_write") {
            // Try to get the variable name being locked
            let var_name = get_receiver_name(&node.receiver);

            // Only count if this is a new variable (not relocking the same mutex)
            if self.locked_vars.insert(var_name.clone()) {
                self.lock_count += 1;
            }
        }

        syn::visit::visit_expr_method_call(self, node);
    }
}

fn get_receiver_name(expr: &syn::Expr) -> String {
    match expr {
        syn::Expr::Path(path) => {
            path.path.segments.last()
                .map(|s| s.ident.to_string())
                .unwrap_or_else(|| "unknown".to_string())
        }
        syn::Expr::Field(field) => {
            if let syn::Member::Named(ident) = &field.member {
                ident.to_string()
            } else {
                "unknown".to_string()
            }
        }
        _ => "unknown".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::checker::Checker;

    #[test]
    fn test_detects_multiple_locks() {
        let code = r#"
            use std::sync::Mutex;

            fn example(a: &Mutex<i32>, b: &Mutex<i32>) {
                let guard_a = a.lock().unwrap();
                let guard_b = b.lock().unwrap();
            }
        "#;

        let syntax = syn::parse_file(code).unwrap();
        let checker = E1506DeadlockLockOrdering::default();

        let mut violations = Vec::new();
        for item in &syntax.items {
            violations.extend(checker.check_item(item, "test.rs").unwrap());
        }

        assert_eq!(violations.len(), 1);
        assert!(violations[0].message.contains("2 locks"));
    }

    #[test]
    fn test_single_lock_passes() {
        let code = r#"
            use std::sync::Mutex;

            fn example(m: &Mutex<i32>) {
                let guard = m.lock().unwrap();
            }
        "#;

        let syntax = syn::parse_file(code).unwrap();
        let checker = E1506DeadlockLockOrdering::default();

        let mut violations = Vec::new();
        for item in &syntax.items {
            violations.extend(checker.check_item(item, "test.rs").unwrap());
        }

        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_no_locks_passes() {
        let code = r#"
            fn example() {
                let x = 42;
            }
        "#;

        let syntax = syn::parse_file(code).unwrap();
        let checker = E1506DeadlockLockOrdering::default();

        let mut violations = Vec::new();
        for item in &syntax.items {
            violations.extend(checker.check_item(item, "test.rs").unwrap());
        }

        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_rwlock_multiple() {
        let code = r#"
            use std::sync::RwLock;

            fn example(a: &RwLock<i32>, b: &RwLock<i32>) {
                let read_a = a.read().unwrap();
                let write_b = b.write().unwrap();
            }
        "#;

        let syntax = syn::parse_file(code).unwrap();
        let checker = E1506DeadlockLockOrdering::default();

        let mut violations = Vec::new();
        for item in &syntax.items {
            violations.extend(checker.check_item(item, "test.rs").unwrap());
        }

        assert_eq!(violations.len(), 1);
    }
}
