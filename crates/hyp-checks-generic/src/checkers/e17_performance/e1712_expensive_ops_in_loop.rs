//! E1712: Expensive operations inside loops
//!
//! Detects expensive operations like Regex::new(), File::open(), or other
//! initialization code inside loops that should be hoisted outside.
//!
//! Example:
//! ```text
//! // Bad: Compiles regex on every iteration
//! for line in lines {
//!     let re = Regex::new(r"\d+").unwrap();
//!     if re.is_match(line) { ... }
//! }
//!
//! // Good: Compile once outside loop
//! let re = Regex::new(r"\d+").unwrap();
//! for line in lines {
//!     if re.is_match(line) { ... }
//! }
//! ```

use crate::{checker::Checker, define_checker, violation::Violation};

use syn::{spanned::Spanned, visit::Visit};

define_checker! {
    /// Checker for E1712: Expensive operations in loops
    E1712ExpensiveOpsInLoop,
    code = "E1712",
    name = "Expensive operation inside loop",
    suggestions = "Move expensive initialization (Regex::new, File::open, etc.) outside the loop",
    target_items = [Function],
    config_entry_name = "e1712_expensive_ops_in_loop",
    config = E1712Config {
        enabled: bool = true,
        severity: crate::config::SeverityLevel = crate::config::SeverityLevel::Medium,
        categories: Vec<crate::config::CheckerCategory> = vec![crate::config::CheckerCategory::Complexity],
    },
    check_item(self, item, file_path) {
        let mut visitor = ExpensiveOpsVisitor {
            violations: Vec::new(),
            file_path,
            checker: self,
            loop_depth: 0,
        };
        visitor.visit_item(item);
        Ok(visitor.violations)
    }
}

struct ExpensiveOpsVisitor<'a> {
    violations: Vec<Violation>,
    file_path: &'a str,
    checker: &'a E1712ExpensiveOpsInLoop,
    loop_depth: usize,
}

impl<'a> ExpensiveOpsVisitor<'a> {
    // Expensive operations to detect
    const EXPENSIVE_CALLS: &'static [(&'static str, &'static str)] = &[
        ("Regex::new", "Regex compilation is expensive - compile once outside the loop"),
        ("RegexSet::new", "RegexSet compilation is expensive - compile once outside the loop"),
        ("File::open", "File operations are expensive - consider batching or caching"),
        ("File::create", "File operations are expensive - consider batching or caching"),
        ("TcpStream::connect", "Network connections are expensive - use connection pooling"),
        ("Client::new", "HTTP client creation is expensive - reuse clients"),
        ("Connection::open", "Database connections are expensive - use connection pooling"),
        ("Mutex::new", "Creating Mutex inside loop is usually wrong - create once outside"),
        ("RwLock::new", "Creating RwLock inside loop is usually wrong - create once outside"),
        ("Arc::new", "Arc::new in loop may indicate data should be shared differently"),
    ];

    fn check_expensive_call(expr: &syn::Expr) -> Option<(&'static str, &'static str)> {
        if let syn::Expr::Call(call) = expr {
            if let syn::Expr::Path(path) = &*call.func {
                let path_str: String = path
                    .path
                    .segments
                    .iter()
                    .map(|s| s.ident.to_string())
                    .collect::<Vec<_>>()
                    .join("::");

                for (pattern, msg) in Self::EXPENSIVE_CALLS {
                    if path_str.ends_with(pattern) || path_str == *pattern {
                        return Some((pattern, msg));
                    }
                }
            }
        }

        // Also check method calls
        if let syn::Expr::MethodCall(method) = expr {
            let method_name = method.method.to_string();
            if method_name == "new" || method_name == "open" || method_name == "connect" {
                // Check if receiver looks like an expensive type
                if let syn::Expr::Path(path) = &*method.receiver {
                    let type_name = path
                        .path
                        .segments
                        .last()
                        .map(|s| s.ident.to_string())
                        .unwrap_or_default();

                    if type_name == "Regex" || type_name == "File" || type_name == "TcpStream" {
                        return Some(("expensive::new", "Expensive initialization inside loop"));
                    }
                }
            }
        }

        None
    }
}

impl<'a> Visit<'a> for ExpensiveOpsVisitor<'a> {
    fn visit_expr(&mut self, node: &'a syn::Expr) {
        // Track loop depth
        let in_loop = matches!(
            node,
            syn::Expr::ForLoop(_) | syn::Expr::While(_) | syn::Expr::Loop(_)
        );

        if in_loop {
            self.loop_depth += 1;
        }

        // Check for expensive operations inside loop
        if self.loop_depth > 0 {
            if let Some((pattern, msg)) = Self::check_expensive_call(node) {
                let span = node.span();
                self.violations.push(
                    Violation::new(
                        self.checker.code(),
                        self.checker.name(),
                        self.checker.severity().into(),
                        format!("{} ({})", msg, pattern),
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
        let checker = E1712ExpensiveOpsInLoop::default();
        let file = syn::parse_file(code).expect("Failed to parse");
        let mut violations = Vec::new();
        for item in &file.items {
            violations.extend(checker.check_item(item, "test.rs").unwrap());
        }
        violations
    }

    #[test]
    fn test_detects_regex_in_loop() {
        let code = r#"
            fn example(lines: Vec<&str>) {
                for line in lines {
                    let re = Regex::new(r"\d+").unwrap();
                    println!("{:?}", re);
                }
            }
        "#;
        let violations = check_code(code);
        assert_eq!(violations.len(), 1);
        assert!(violations[0].message.contains("Regex"));
    }

    #[test]
    fn test_detects_file_open_in_loop() {
        let code = r#"
            fn example(paths: Vec<&str>) {
                for path in paths {
                    let f = File::open(path).unwrap();
                    println!("{:?}", f);
                }
            }
        "#;
        let violations = check_code(code);
        assert_eq!(violations.len(), 1);
    }

    #[test]
    fn test_regex_outside_loop_passes() {
        let code = r#"
            fn example(lines: Vec<&str>) {
                let re = Regex::new(r"\d+").unwrap();
                for line in lines {
                    println!("{:?}", re);
                }
            }
        "#;
        let violations = check_code(code);
        assert!(violations.is_empty());
    }

    #[test]
    fn test_detects_mutex_new_in_loop() {
        let code = r#"
            fn example(items: Vec<i32>) {
                for item in items {
                    let mutex = Mutex::new(item);
                    println!("{:?}", mutex);
                }
            }
        "#;
        let violations = check_code(code);
        assert_eq!(violations.len(), 1);
    }
}
