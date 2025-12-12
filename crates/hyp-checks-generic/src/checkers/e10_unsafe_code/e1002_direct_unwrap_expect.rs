//! E1002: Direct use of unwrap() and expect()
//!
//! Detects usage of `.unwrap()` and `.expect()` which crash the program instead of
//! returning errors to callers. This is problematic because:
//!
//! 1. **Caller has no control**: The caller cannot decide how to handle the error
//! 2. **No recovery possible**: The entire program/thread terminates
//! 3. **Poor user experience**: Users see crashes instead of helpful error messages
//! 4. **Panic cascades**: In concurrent code, one panic can cause others (e.g., mutex poisoning)
//! 5. **Resource leaks**: Resources may not be properly cleaned up on panic
//!
//! ## Examples of dangerous unwrap():
//!
//! ```text
//! // Lock poisoning cascade - if another thread panicked while holding
//! // this lock, THIS code will ALSO panic!
//! let guard = mutex.lock().unwrap();
//!
//! // File operations - crashes if file doesn't exist, permissions denied, etc.
//! let content = std::fs::read_to_string("config.txt").unwrap();
//!
//! // User input parsing - crashes on invalid input
//! let port: u16 = args[1].parse().unwrap();
//! ```
//!
//! Prefer `?` operator, `if let`, `match`, or combinators like `unwrap_or_default()`.

use crate::{checker::Checker, define_checker, violation::Violation};

use syn::visit::Visit;

define_checker! {
    /// Checker for E1002: Direct unwrap/expect usage
    E1002DirectUnwrapExpect,
    code = "E1002",
    name = "Direct use of unwrap/expect crashes program",
    suggestions = "Return Result to caller with ?, use if let/match, or unwrap_or_default(). Never use unwrap() on lock() - it causes panic cascades.",
    target_items = [Function],
    config_entry_name = "e1002_direct_unwrap_expect",
    config = E1002Config {
        enabled: bool = true,
        severity: crate::config::SeverityLevel = crate::config::SeverityLevel::High,
        categories: Vec<crate::config::CheckerCategory> = vec![crate::config::CheckerCategory::Operations],
        /// Allow unwrap in test code
        allow_in_tests: bool = true,
    },
    check_item(self, item, file_path) {
        // Skip test code if configured
        if self.config.allow_in_tests && file_path.contains("test") {
            return Ok(Vec::new());
        }

        let mut visitor = UnwrapVisitor {
            violations: Vec::new(),
            file_path,
            checker: self,
        };
        visitor.visit_item(item);
        Ok(visitor.violations)
    }
}

struct UnwrapVisitor<'a> {
    violations: Vec<Violation>,
    file_path: &'a str,
    checker: &'a E1002DirectUnwrapExpect,
}

impl<'a> Visit<'a> for UnwrapVisitor<'a> {
    fn visit_expr_method_call(&mut self, node: &'a syn::ExprMethodCall) {
        let method_name = node.method.to_string();

        if method_name == "unwrap" {
            // Check if this is a lock().unwrap() pattern (extra dangerous!)
            let is_lock_unwrap = if let syn::Expr::MethodCall(inner) = &*node.receiver {
                let inner_method = inner.method.to_string();
                matches!(inner_method.as_str(), "lock" | "try_lock" | "read" | "write")
            } else {
                false
            };

            let message = if is_lock_unwrap {
                "Using unwrap() on lock() causes panic cascades. If any thread panicked while holding this lock, this code will ALSO panic, spreading the failure."
            } else {
                "Using unwrap() crashes the program on None/Err. Return errors to callers instead of crashing."
            };

            let start = node.method.span().start();
            self.violations.push(
                Violation::new(
                    self.checker.code(),
                    self.checker.name(),
                    self.checker.severity().into(),
                    message,
                    self.file_path,
                    start.line,
                    start.column.saturating_add(1),
                )
                .with_suggestion(self.checker.suggestions()),
            );
        } else if method_name == "expect" {
            let start = node.method.span().start();
            self.violations.push(
                Violation::new(
                    self.checker.code(),
                    self.checker.name(),
                    self.checker.severity().into(),
                    "Using expect() still crashes the program - it just adds a message. Return errors to callers instead.",
                    self.file_path,
                    start.line,
                    start.column.saturating_add(1),
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
    fn test_detects_unwrap() {
        let code = r#"
            fn example() {
                let data = Some(42);
                let _value = data.unwrap();
            }
        "#;

        let syntax = syn::parse_file(code).unwrap();
        let checker = E1002DirectUnwrapExpect::default();

        let mut violations = Vec::new();
        for item in &syntax.items {
            violations.extend(checker.check_item(item, "src/lib.rs").unwrap());
        }

        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].code, "E1002");
    }

    #[test]
    fn test_detects_expect() {
        let code = r#"
            fn example() {
                let data = Some(42);
                let _value = data.expect("should exist");
            }
        "#;

        let syntax = syn::parse_file(code).unwrap();
        let checker = E1002DirectUnwrapExpect::default();

        let mut violations = Vec::new();
        for item in &syntax.items {
            violations.extend(checker.check_item(item, "src/lib.rs").unwrap());
        }

        assert_eq!(violations.len(), 1);
        assert!(violations[0].message.contains("expect()"));
    }

    #[test]
    fn test_detects_lock_unwrap() {
        let code = r#"
            fn example(mutex: &std::sync::Mutex<i32>) {
                let guard = mutex.lock().unwrap();
            }
        "#;

        let syntax = syn::parse_file(code).unwrap();
        let checker = E1002DirectUnwrapExpect::default();

        let mut violations = Vec::new();
        for item in &syntax.items {
            violations.extend(checker.check_item(item, "src/lib.rs").unwrap());
        }

        assert_eq!(violations.len(), 1);
        assert!(violations[0].message.contains("panic cascades"));
    }

    #[test]
    fn test_detects_chained_unwrap() {
        let code = r#"
            fn example() {
                let nested = Some(Some(42));
                let _value = nested.unwrap().unwrap();
            }
        "#;

        let syntax = syn::parse_file(code).unwrap();
        let checker = E1002DirectUnwrapExpect::default();

        let mut violations = Vec::new();
        for item in &syntax.items {
            violations.extend(checker.check_item(item, "src/lib.rs").unwrap());
        }

        assert_eq!(violations.len(), 2);
    }

    #[test]
    fn test_unwrap_or_passes() {
        let code = r#"
            fn example() {
                let data = Some(42);
                let _value = data.unwrap_or(0);
                let _value2 = data.unwrap_or_default();
                let _value3 = data.unwrap_or_else(|| 0);
            }
        "#;

        let syntax = syn::parse_file(code).unwrap();
        let checker = E1002DirectUnwrapExpect::default();

        let mut violations = Vec::new();
        for item in &syntax.items {
            violations.extend(checker.check_item(item, "src/lib.rs").unwrap());
        }

        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_question_mark_passes() {
        let code = r#"
            fn example() -> Result<i32, &'static str> {
                let data: Result<i32, &'static str> = Ok(42);
                let value = data?;
                Ok(value)
            }
        "#;

        let syntax = syn::parse_file(code).unwrap();
        let checker = E1002DirectUnwrapExpect::default();

        let mut violations = Vec::new();
        for item in &syntax.items {
            violations.extend(checker.check_item(item, "src/lib.rs").unwrap());
        }

        assert_eq!(violations.len(), 0);
    }
}
