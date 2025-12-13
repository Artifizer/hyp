//! E1512: Prohibit std::thread::spawn in async codebases
//!
//! In tokio-based microservices, using `std::thread::spawn` instead of
//! `tokio::task::spawn_blocking` breaks the async runtime's cooperative
//! scheduling model, causing thread pool exhaustion, context loss, and
//! poor observability.
//!
//! Example:
//! ```text
//! // Bad: Creates OS thread outside tokio's control
//! std::thread::spawn(|| {
//!     blocking_io_operation();
//! });
//!
//! // Good: Uses tokio's blocking thread pool
//! tokio::task::spawn_blocking(|| {
//!     blocking_io_operation();
//! });
//! ```

use crate::{checker::Checker, define_checker, violation::Violation};

use syn::{spanned::Spanned, visit::Visit};

define_checker! {
    /// Checker for E1512: Prohibit std::thread::spawn
    E1512ProhibitStdThreadSpawn,
    code = "E1512",
    name = "std::thread::spawn is prohibited in async codebases",
    suggestions = "Use tokio::task::spawn_blocking() instead. This ensures blocking work is scheduled through tokio's thread pool, preserving runtime context and observability.",
    target_items = [Function],
    config_entry_name = "e1512_prohibit_std_thread_spawn",
    config = E1512Config {
        enabled: bool = true,
        severity: crate::config::SeverityLevel = crate::config::SeverityLevel::High,
        categories: Vec<crate::config::CheckerCategory> = vec![crate::config::CheckerCategory::Operations],
    },
    check_item(self, item, file_path) {
        let mut visitor = ThreadSpawnVisitor {
            violations: Vec::new(),
            file_path,
            checker: self,
        };
        visitor.visit_item(item);
        Ok(visitor.violations)
    }
}

struct ThreadSpawnVisitor<'a> {
    violations: Vec<Violation>,
    file_path: &'a str,
    checker: &'a E1512ProhibitStdThreadSpawn,
}

impl<'a> ThreadSpawnVisitor<'a> {
    /// Check if expression is std::thread::spawn or thread::spawn
    fn is_std_thread_spawn(expr: &syn::Expr) -> bool {
        if let syn::Expr::Call(call) = expr {
            if let syn::Expr::Path(path) = &*call.func {
                let segments: Vec<String> = path
                    .path
                    .segments
                    .iter()
                    .map(|s| s.ident.to_string())
                    .collect();

                // Match std::thread::spawn or thread::spawn
                if segments.len() >= 2 {
                    let last = segments.last().map(|s| s.as_str());
                    let second_last = segments.get(segments.len() - 2).map(|s| s.as_str());

                    if last == Some("spawn") && second_last == Some("thread") {
                        return true;
                    }
                }
            }
        }
        false
    }
}

impl<'a> Visit<'a> for ThreadSpawnVisitor<'a> {
    fn visit_expr(&mut self, node: &'a syn::Expr) {
        if Self::is_std_thread_spawn(node) {
            let span = node.span();
            self.violations.push(
                Violation::new(
                    self.checker.code(),
                    self.checker.name(),
                    self.checker.severity().into(),
                    "std::thread::spawn creates OS threads outside tokio's control. In async codebases, this causes thread pool exhaustion, context loss, and poor observability.",
                    self.file_path,
                    span.start().line,
                    span.start().column + 1,
                )
                .with_suggestion(self.checker.suggestions()),
            );
        }

        syn::visit::visit_expr(self, node);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::checker::Checker;

    fn check_code(code: &str) -> Vec<Violation> {
        let checker = E1512ProhibitStdThreadSpawn::default();
        let file = syn::parse_file(code).expect("Failed to parse");
        let mut violations = Vec::new();
        for item in &file.items {
            violations.extend(checker.check_item(item, "test.rs").unwrap());
        }
        violations
    }

    #[test]
    fn test_detects_std_thread_spawn_fully_qualified() {
        let code = r#"
            fn example() {
                std::thread::spawn(|| {
                    println!("blocking work");
                });
            }
        "#;
        let violations = check_code(code);
        assert_eq!(violations.len(), 1);
        assert!(violations[0].code == "E1512");
    }

    #[test]
    fn test_detects_thread_spawn_with_use() {
        let code = r#"
            use std::thread;

            fn example() {
                thread::spawn(|| {
                    println!("blocking work");
                });
            }
        "#;
        let violations = check_code(code);
        assert_eq!(violations.len(), 1);
    }

    #[test]
    fn test_tokio_spawn_blocking_passes() {
        let code = r#"
            async fn example() {
                tokio::task::spawn_blocking(|| {
                    println!("blocking work");
                });
            }
        "#;
        let violations = check_code(code);
        assert!(violations.is_empty());
    }

    #[test]
    fn test_tokio_spawn_passes() {
        let code = r#"
            async fn example() {
                tokio::spawn(async {
                    println!("async work");
                });
            }
        "#;
        let violations = check_code(code);
        assert!(violations.is_empty());
    }

    #[test]
    fn test_detects_multiple_spawns() {
        let code = r#"
            fn example() {
                std::thread::spawn(|| { work1(); });
                std::thread::spawn(|| { work2(); });
            }
        "#;
        let violations = check_code(code);
        assert_eq!(violations.len(), 2);
    }
}
