//! E1511: Unbounded task/thread spawning in loops
//!
//! Detects spawning threads or async tasks inside loops without bounds.
//! This pattern can lead to resource exhaustion and denial of service.
//!
//! Example:
//! ```text
//! // Bad: Unbounded spawning
//! for item in items {
//!     tokio::spawn(async move { process(item).await });
//! }
//!
//! // Good: Use bounded concurrency
//! let semaphore = Arc::new(Semaphore::new(10));
//! for item in items {
//!     let permit = semaphore.clone().acquire_owned().await;
//!     tokio::spawn(async move { let _p = permit; process(item).await });
//! }
//! ```

use crate::{checker::Checker, define_checker, violation::Violation};

use syn::{spanned::Spanned, visit::Visit};

define_checker! {
    /// Checker for E1511: Unbounded task/thread spawning
    E1511UnboundedSpawning,
    code = "E1511",
    name = "Unbounded task/thread spawning in loop",
    suggestions = "Use bounded concurrency with Semaphore, thread pools, or stream::buffer_unordered()",
    target_items = [Function],
    config_entry_name = "e1511_unbounded_spawning",
    config = E1511Config {
        enabled: bool = true,
        severity: crate::config::SeverityLevel = crate::config::SeverityLevel::High,
        categories: Vec<crate::config::CheckerCategory> = vec![crate::config::CheckerCategory::Operations],
    },
    check_item(self, item, file_path) {
        let mut visitor = SpawnVisitor {
            violations: Vec::new(),
            file_path,
            checker: self,
            loop_depth: 0,
        };
        visitor.visit_item(item);
        Ok(visitor.violations)
    }
}

struct SpawnVisitor<'a> {
    violations: Vec<Violation>,
    file_path: &'a str,
    checker: &'a E1511UnboundedSpawning,
    loop_depth: usize,
}

impl<'a> SpawnVisitor<'a> {
    fn is_spawn_call(expr: &syn::Expr) -> Option<&str> {
        if let syn::Expr::Call(call) = expr {
            if let syn::Expr::Path(path) = &*call.func {
                let path_str: String = path
                    .path
                    .segments
                    .iter()
                    .map(|s| s.ident.to_string())
                    .collect::<Vec<_>>()
                    .join("::");

                // Common spawn patterns
                if path_str.ends_with("spawn")
                    || path_str.ends_with("spawn_blocking")
                    || path_str.ends_with("spawn_local")
                {
                    return Some("spawn");
                }
            }
        }

        // Also check method calls like thread::spawn
        if let syn::Expr::MethodCall(method) = expr {
            let method_name = method.method.to_string();
            if method_name == "spawn" || method_name == "spawn_blocking" {
                return Some("spawn");
            }
        }

        None
    }
}

impl<'a> Visit<'a> for SpawnVisitor<'a> {
    fn visit_expr(&mut self, node: &'a syn::Expr) {
        // Track loop depth
        let in_loop = matches!(
            node,
            syn::Expr::ForLoop(_) | syn::Expr::While(_) | syn::Expr::Loop(_)
        );

        if in_loop {
            self.loop_depth += 1;
        }

        // Check for spawn inside loop
        if self.loop_depth > 0 {
            if let Some(_spawn_type) = Self::is_spawn_call(node) {
                let span = node.span();
                self.violations.push(
                    Violation::new(
                        self.checker.code(),
                        self.checker.name(),
                        self.checker.severity().into(),
                        "Spawning tasks/threads in a loop without bounds can exhaust system resources. Use bounded concurrency.",
                        self.file_path,
                        span.start().line,
                        span.start().column + 1,
                    )
                    .with_suggestion(self.checker.suggestions()),
                );
            }
        }

        syn::visit::visit_expr(self, node);

        if in_loop {
            self.loop_depth -= 1;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::checker::Checker;

    fn check_code(code: &str) -> Vec<Violation> {
        let checker = E1511UnboundedSpawning::default();
        let file = syn::parse_file(code).expect("Failed to parse");
        let mut violations = Vec::new();
        for item in &file.items {
            violations.extend(checker.check_item(item, "test.rs").unwrap());
        }
        violations
    }

    #[test]
    fn test_detects_spawn_in_for_loop() {
        let code = r#"
            fn example(items: Vec<i32>) {
                for item in items {
                    std::thread::spawn(move || {
                        println!("{}", item);
                    });
                }
            }
        "#;
        let violations = check_code(code);
        assert_eq!(violations.len(), 1);
    }

    #[test]
    fn test_detects_tokio_spawn_in_loop() {
        let code = r#"
            async fn example(items: Vec<i32>) {
                for item in items {
                    tokio::spawn(async move {
                        println!("{}", item);
                    });
                }
            }
        "#;
        let violations = check_code(code);
        assert_eq!(violations.len(), 1);
    }

    #[test]
    fn test_spawn_outside_loop_passes() {
        let code = r#"
            fn example() {
                std::thread::spawn(|| {
                    println!("hello");
                });
            }
        "#;
        let violations = check_code(code);
        assert!(violations.is_empty());
    }

    #[test]
    fn test_detects_while_loop() {
        let code = r#"
            fn example(items: &mut Vec<i32>) {
                while let Some(item) = items.pop() {
                    std::thread::spawn(move || {
                        println!("{}", item);
                    });
                }
            }
        "#;
        let violations = check_code(code);
        assert_eq!(violations.len(), 1);
    }
}
