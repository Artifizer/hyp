//! E1016: Mutex unwrap - lock poisoning and panic cascades
//!
//! Detects `.lock().unwrap()` and similar patterns on Mutex/RwLock which are
//! particularly dangerous due to lock poisoning behavior.
//!
//! ## Why Mutex unwrap is especially dangerous
//!
//! In Rust, when a thread panics while holding a mutex lock, the mutex becomes
//! "poisoned". This is a safety mechanism to indicate the protected data might
//! be in an inconsistent state. However, using `.unwrap()` on a poisoned lock
//! causes a **panic cascade**:
//!
//! ```text
//! Thread A: panics while holding lock
//!     ↓
//! Mutex becomes poisoned
//!     ↓
//! Thread B: calls lock().unwrap() → PANIC!
//!     ↓
//! Thread C: calls lock().unwrap() → PANIC!
//!     ↓
//! ... entire application crashes from one error
//! ```
//!
//! ## The Right Way to Handle Poisoned Locks
//!
//! ```text
//! // Option 1: Recover the data despite poisoning
//! let guard = mutex.lock().unwrap_or_else(|poisoned| {
//!     log::warn!("Lock was poisoned, recovering data");
//!     poisoned.into_inner()
//! });
//!
//! // Option 2: Propagate as an error
//! let guard = mutex.lock().map_err(|_| MyError::LockPoisoned)?;
//!
//! // Option 3: Use parking_lot which doesn't poison
//! use parking_lot::Mutex;
//! let guard = mutex.lock(); // No Result, no poisoning!
//! ```

use crate::{checker::Checker, define_checker, violation::Violation};

use syn::visit::Visit;

define_checker! {
    /// Checker for E1016: Mutex unwrap causing panic cascades
    E1016MutexUnwrap,
    code = "E1016",
    name = "Mutex raw lock().unwrap() (causes panic cascades)",
    suggestions = "Use lock().unwrap_or_else(|e| e.into_inner()) to recover, or propagate error with ?, or use parking_lot::Mutex",
    target_items = [Function],
    config_entry_name = "e1016_mutex_unwrap",
    config = E1016Config {
        enabled: bool = true,
        severity: crate::config::SeverityLevel = crate::config::SeverityLevel::High,
        categories: Vec<crate::config::CheckerCategory> = vec![crate::config::CheckerCategory::Operations],
    },
    check_item(self, item, file_path) {
        let mut visitor = MutexUnwrapVisitor {
            violations: Vec::new(),
            file_path,
            checker: self,
        };
        visitor.visit_item(item);
        Ok(visitor.violations)
    }
}

struct MutexUnwrapVisitor<'a> {
    violations: Vec<Violation>,
    file_path: &'a str,
    checker: &'a E1016MutexUnwrap,
}

impl<'a> Visit<'a> for MutexUnwrapVisitor<'a> {
    fn visit_expr_method_call(&mut self, node: &'a syn::ExprMethodCall) {
        let method_name = node.method.to_string();

        // Check for .unwrap() or .expect() after lock-related methods
        if matches!(method_name.as_str(), "unwrap" | "expect") {
            if let syn::Expr::MethodCall(inner) = &*node.receiver {
                let inner_method = inner.method.to_string();

                // Check if it's a lock/read/write call on a mutex/rwlock
                if matches!(inner_method.as_str(), "lock" | "try_lock" | "read" | "write" | "try_read" | "try_write") {
                    let lock_type = match inner_method.as_str() {
                        "lock" | "try_lock" => "Mutex",
                        "read" | "try_read" | "write" | "try_write" => "RwLock",
                        _ => "Lock",
                    };

                    let message = format!(
                        "{}().{}() causes panic cascades. If any thread panicked while holding this {}, \
                        calling {}() here will ALSO panic, spreading the failure across all threads. \
                        Use unwrap_or_else(|e| e.into_inner()) to recover, or handle the PoisonError explicitly.",
                        inner_method,
                        method_name,
                        lock_type,
                        method_name
                    );

                    let start = node.method.span().start();
                    self.violations.push(
                        Violation::new(
                            self.checker.code(),
                            self.checker.name(),
                            self.checker.severity().into(),
                            &message,
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

    #[test]
    fn test_detects_mutex_lock_unwrap() {
        let code = r#"
            use std::sync::Mutex;

            fn example(m: &Mutex<i32>) {
                let guard = m.lock().unwrap();
            }
        "#;

        let syntax = syn::parse_file(code).unwrap();
        let checker = E1016MutexUnwrap::default();

        let mut violations = Vec::new();
        for item in &syntax.items {
            violations.extend(checker.check_item(item, "test.rs").unwrap());
        }

        assert_eq!(violations.len(), 1);
        assert!(violations[0].message.contains("panic cascades"));
        assert!(violations[0].message.contains("Mutex"));
    }

    #[test]
    fn test_detects_rwlock_read_unwrap() {
        let code = r#"
            use std::sync::RwLock;

            fn example(rw: &RwLock<i32>) {
                let guard = rw.read().unwrap();
            }
        "#;

        let syntax = syn::parse_file(code).unwrap();
        let checker = E1016MutexUnwrap::default();

        let mut violations = Vec::new();
        for item in &syntax.items {
            violations.extend(checker.check_item(item, "test.rs").unwrap());
        }

        assert_eq!(violations.len(), 1);
        assert!(violations[0].message.contains("RwLock"));
    }

    #[test]
    fn test_detects_rwlock_write_expect() {
        let code = r#"
            use std::sync::RwLock;

            fn example(rw: &RwLock<i32>) {
                let guard = rw.write().expect("should be able to write");
            }
        "#;

        let syntax = syn::parse_file(code).unwrap();
        let checker = E1016MutexUnwrap::default();

        let mut violations = Vec::new();
        for item in &syntax.items {
            violations.extend(checker.check_item(item, "test.rs").unwrap());
        }

        assert_eq!(violations.len(), 1);
        assert!(violations[0].message.contains("expect()"));
    }

    #[test]
    fn test_unwrap_or_else_passes() {
        let code = r#"
            use std::sync::Mutex;

            fn example(m: &Mutex<i32>) {
                let guard = m.lock().unwrap_or_else(|e| e.into_inner());
            }
        "#;

        let syntax = syn::parse_file(code).unwrap();
        let checker = E1016MutexUnwrap::default();

        let mut violations = Vec::new();
        for item in &syntax.items {
            violations.extend(checker.check_item(item, "test.rs").unwrap());
        }

        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_question_mark_passes() {
        let code = r#"
            use std::sync::{Mutex, PoisonError, MutexGuard};

            fn example(m: &Mutex<i32>) -> Result<(), Box<dyn std::error::Error>> {
                let guard = m.lock().map_err(|e| "lock poisoned")?;
                Ok(())
            }
        "#;

        let syntax = syn::parse_file(code).unwrap();
        let checker = E1016MutexUnwrap::default();

        let mut violations = Vec::new();
        for item in &syntax.items {
            violations.extend(checker.check_item(item, "test.rs").unwrap());
        }

        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_multiple_lock_unwraps() {
        let code = r#"
            use std::sync::{Mutex, RwLock};

            fn example(m: &Mutex<i32>, rw: &RwLock<String>) {
                let guard1 = m.lock().unwrap();
                let guard2 = rw.read().unwrap();
                let guard3 = rw.write().unwrap();
            }
        "#;

        let syntax = syn::parse_file(code).unwrap();
        let checker = E1016MutexUnwrap::default();

        let mut violations = Vec::new();
        for item in &syntax.items {
            violations.extend(checker.check_item(item, "test.rs").unwrap());
        }

        assert_eq!(violations.len(), 3);
    }
}
