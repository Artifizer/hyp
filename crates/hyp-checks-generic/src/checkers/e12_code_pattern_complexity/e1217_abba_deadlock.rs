//! E1217: Classical ABBA Deadlock pattern
//!
//! Detects potential ABBA deadlock patterns where multiple locks are acquired
//! in different orders, which can lead to deadlocks.
//!
//! Example of ABBA deadlock:
//! ```text
//! // Thread 1:            // Thread 2:
//! let _a = lock_a.lock(); let _b = lock_b.lock();
//! let _b = lock_b.lock(); let _a = lock_a.lock();  // Deadlock!
//! ```
//!
//! This checker looks for functions that acquire multiple locks and flags
//! potential ordering issues.

use crate::{define_checker, violation::Violation};
use syn::visit::Visit;

define_checker! {
    /// Checker for E1217: Classical ABBA Deadlock
    E1217AbbaDeadlock,
    code = "E1217",
    name = "Classical ABBA deadlock pattern",
    suggestions = "Always acquire locks in a consistent order. Consider using parking_lot::lock_api or a lock ordering convention.",
    target_items = [Function],
    config_entry_name = "e1217_abba_deadlock",
    config = E1217Config {
        enabled: bool = true,
        severity: crate::config::SeverityLevel = crate::config::SeverityLevel::High,
        categories: Vec<crate::config::CheckerCategory> = vec![crate::config::CheckerCategory::Operations],
        /// Maximum locks acquired in a single function before warning
        max_locks_per_function: usize = 2,
    },
    check_item(self, item, file_path) {
        let mut visitor = LockOrderVisitor {
            violations: Vec::new(),
            file_path,
            checker: self,
            lock_calls: Vec::new(),
            locked_vars: std::collections::HashSet::new(),
        };
        visitor.visit_item(item);

        // Check if multiple locks are acquired
        if visitor.lock_calls.len() > self.config.max_locks_per_function {
            use syn::spanned::Spanned;
            let first_lock = &visitor.lock_calls[0];
            let start = first_lock.span().start();
            visitor.violations.push(
                Violation::new(
                    self.code(),
                    self.name(),
                    self.severity().into(),
                    format!(
                        "Function acquires {} locks. Multiple lock acquisition risks ABBA deadlock if not ordered consistently.",
                        visitor.lock_calls.len()
                    ),
                    file_path,
                    start.line,
                    start.column + 1,
                )
                .with_suggestion(self.suggestions()),
            );
        }

        Ok(visitor.violations)
    }
}

#[allow(dead_code)]
struct LockOrderVisitor<'a> {
    violations: Vec<Violation>,
    file_path: &'a str,
    checker: &'a E1217AbbaDeadlock,
    lock_calls: Vec<proc_macro2::Span>,
    locked_vars: std::collections::HashSet<String>,
}

impl<'a> Visit<'a> for LockOrderVisitor<'a> {
    fn visit_expr_method_call(&mut self, node: &'a syn::ExprMethodCall) {
        let method_name = node.method.to_string();

        // Detect lock acquisition methods on DIFFERENT variables only
        if matches!(method_name.as_str(),
            "lock" | "try_lock" | "read" | "write" | "try_read" | "try_write"
        ) {
            // Get the variable name being locked
            let var_name = get_receiver_name(&node.receiver);

            // Only record if this is a new variable (not relocking the same mutex)
            if self.locked_vars.insert(var_name) {
                use syn::spanned::Spanned;
                self.lock_calls.push(node.span());
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

            fn potential_deadlock(a: &Mutex<i32>, b: &Mutex<i32>, c: &Mutex<i32>) {
                let _guard_a = a.lock();
                let _guard_b = b.lock();
                let _guard_c = c.lock();
            }
        "#;

        let syntax = syn::parse_file(code).unwrap();
        let checker = E1217AbbaDeadlock::default();

        let mut violations = Vec::new();
        for item in &syntax.items {
            violations.extend(checker.check_item(item, "test.rs").unwrap());
        }

        assert_eq!(violations.len(), 1);
        assert!(violations[0].message.contains("ABBA deadlock"));
    }

    #[test]
    fn test_single_lock_passes() {
        let code = r#"
            use std::sync::Mutex;

            fn single_lock(a: &Mutex<i32>) {
                let _guard = a.lock();
            }
        "#;

        let syntax = syn::parse_file(code).unwrap();
        let checker = E1217AbbaDeadlock::default();

        let mut violations = Vec::new();
        for item in &syntax.items {
            violations.extend(checker.check_item(item, "test.rs").unwrap());
        }

        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_detects_rwlock_multiple() {
        let code = r#"
            use std::sync::RwLock;

            fn multiple_rw(a: &RwLock<i32>, b: &RwLock<i32>, c: &RwLock<i32>) {
                let _r1 = a.read();
                let _r2 = b.read();
                let _w = c.write();
            }
        "#;

        let syntax = syn::parse_file(code).unwrap();
        let checker = E1217AbbaDeadlock::default();

        let mut violations = Vec::new();
        for item in &syntax.items {
            violations.extend(checker.check_item(item, "test.rs").unwrap());
        }

        assert_eq!(violations.len(), 1);
    }
}
