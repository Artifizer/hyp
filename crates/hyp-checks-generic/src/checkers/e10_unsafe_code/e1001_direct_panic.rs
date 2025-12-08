//! E1001: Direct call of panic() in production code
//!
//! Detects direct calls to the `panic!()` macro which immediately crash the program.
//! Production code should return Result types instead to allow callers to handle errors.

use crate::{checker::Checker, define_checker, violation::Violation};

use syn::{spanned::Spanned, visit::Visit, Macro};

define_checker! {
    /// Checker for E1001: Direct panic calls
    E1001DirectPanic,
    code = "E1001",
    name = "Direct panic() call",
    suggestions = "Return Result<T, E> instead of panicking",
    target_items = [Function],
    config_entry_name = "e1001_direct_panic",
    /// Configuration for E1001: Direct panic checker
    config = E1001Config {
        /// Whether this checker is enabled
        enabled: bool = true,
        /// Severity level, defaults to High
        severity: crate::config::SeverityLevel = crate::config::SeverityLevel::High,
        /// Categories this checker belongs to, defaults to [Operations]
        categories: Vec<crate::config::CheckerCategory> = vec![crate::config::CheckerCategory::Operations],
    },
    // AST node item checker
    check_item(self, item, file_path) {
        let mut visitor = PanicVisitor {
            violations: Vec::new(),
            file_path,
            checker: self,
        };
        visitor.visit_item(item);
        Ok(visitor.violations)
    }
}

/// Visitor that traverses the AST looking for panic! macro calls
struct PanicVisitor<'a> {
    violations: Vec<Violation>,
    file_path: &'a str,
    checker: &'a E1001DirectPanic,
}

impl<'a> PanicVisitor<'a> {
    /// Create a violation for a panic macro at the given span
    fn create_violation(&self, span: proc_macro2::LineColumn) -> Violation {
        Violation::new(
            self.checker.code(),
            self.checker.name(),
            self.checker.severity().into(),
            "Direct call to panic!() crashes the program. Use Result<T, E> to allow callers to handle errors gracefully.",
            self.file_path,
            span.line,
            span.column.saturating_add(1), // Convert 0-indexed to 1-indexed
        )
        .with_suggestion(self.checker.suggestions())
    }
}

impl<'a> Visit<'a> for PanicVisitor<'a> {
    fn visit_stmt(&mut self, node: &'a syn::Stmt) {
        // Check if this is a macro statement (panic! is typically a statement)
        if let syn::Stmt::Macro(stmt_macro) = node {
            if is_panic_macro(&stmt_macro.mac) {
                let span = stmt_macro
                    .mac
                    .path
                    .segments
                    .first()
                    .map(|seg| seg.ident.span())
                    .unwrap_or_else(|| stmt_macro.mac.path.span());

                self.violations.push(self.create_violation(span.start()));
            }
        }

        // Continue visiting nested statements
        syn::visit::visit_stmt(self, node);
    }

    fn visit_expr(&mut self, node: &'a syn::Expr) {
        // Also check for panic! in expression context
        if let syn::Expr::Macro(expr_macro) = node {
            if is_panic_macro(&expr_macro.mac) {
                let span = expr_macro
                    .mac
                    .path
                    .segments
                    .first()
                    .map(|seg| seg.ident.span())
                    .unwrap_or_else(|| expr_macro.mac.path.span());

                self.violations.push(self.create_violation(span.start()));
            }
        }

        // Continue visiting nested expressions
        syn::visit::visit_expr(self, node);
    }
}

/// Check if a macro is a panic! macro
fn is_panic_macro(mac: &Macro) -> bool {
    mac.path
        .segments
        .last()
        .map(|seg| seg.ident == "panic")
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::checker::Checker;

    #[test]
    fn test_detects_panic_in_if_statement() {
        let code = r#"
            fn example() {
                if x > 10 {
                    panic!("Value too large");
                }
            }
        "#;

        let syntax = syn::parse_file(code).unwrap();
        let checker = E1001DirectPanic::default();

        let mut violations = Vec::new();
        for item in &syntax.items {
            violations.extend(checker.check_item(item, "test.rs").unwrap());
        }

        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].code, "E1001");
    }

    #[test]
    fn test_detects_panic_at_top_level() {
        let code = r#"
            fn example() {
                panic!("Always fails");
            }
        "#;

        let syntax = syn::parse_file(code).unwrap();
        let checker = E1001DirectPanic::default();

        let mut violations = Vec::new();
        for item in &syntax.items {
            violations.extend(checker.check_item(item, "test.rs").unwrap());
        }

        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].code, "E1001");
    }

    #[test]
    fn test_detects_panic_in_match_arm() {
        let code = r#"
            fn example(x: Option<i32>) {
                match x {
                    Some(v) => println!("{}", v),
                    None => panic!("No value provided"),
                }
            }
        "#;

        let syntax = syn::parse_file(code).unwrap();
        let checker = E1001DirectPanic::default();

        let mut violations = Vec::new();
        for item in &syntax.items {
            violations.extend(checker.check_item(item, "test.rs").unwrap());
        }

        assert_eq!(violations.len(), 1);
    }

    #[test]
    fn test_detects_panic_in_else_block() {
        let code = r#"
            fn example(x: i32) {
                if x > 0 {
                    println!("Positive");
                } else {
                    panic!("Not positive");
                }
            }
        "#;

        let syntax = syn::parse_file(code).unwrap();
        let checker = E1001DirectPanic::default();

        let mut violations = Vec::new();
        for item in &syntax.items {
            violations.extend(checker.check_item(item, "test.rs").unwrap());
        }

        assert_eq!(violations.len(), 1);
    }

    #[test]
    fn test_detects_multiple_panics() {
        let code = r#"
            fn example(x: i32) {
                if x < 0 {
                    panic!("Negative value");
                }
                if x > 100 {
                    panic!("Value too large");
                }
            }
        "#;

        let syntax = syn::parse_file(code).unwrap();
        let checker = E1001DirectPanic::default();

        let mut violations = Vec::new();
        for item in &syntax.items {
            violations.extend(checker.check_item(item, "test.rs").unwrap());
        }

        assert_eq!(violations.len(), 2);
    }

    #[test]
    fn test_detects_panic_in_nested_block() {
        let code = r#"
            fn example() {
                {
                    {
                        panic!("Deeply nested");
                    }
                }
            }
        "#;

        let syntax = syn::parse_file(code).unwrap();
        let checker = E1001DirectPanic::default();

        let mut violations = Vec::new();
        for item in &syntax.items {
            violations.extend(checker.check_item(item, "test.rs").unwrap());
        }

        assert_eq!(violations.len(), 1);
    }

    #[test]
    fn test_detects_panic_in_loop() {
        let code = r#"
            fn example() {
                loop {
                    panic!("Infinite panic");
                }
            }
        "#;

        let syntax = syn::parse_file(code).unwrap();
        let checker = E1001DirectPanic::default();

        let mut violations = Vec::new();
        for item in &syntax.items {
            violations.extend(checker.check_item(item, "test.rs").unwrap());
        }

        assert_eq!(violations.len(), 1);
    }

    #[test]
    fn test_no_panic_with_result() {
        let code = r#"
            fn example() -> Result<(), String> {
                if x > 10 {
                    return Err("Value too large".to_string());
                }
                Ok(())
            }
        "#;

        let syntax = syn::parse_file(code).unwrap();
        let checker = E1001DirectPanic::default();

        let mut violations = Vec::new();
        for item in &syntax.items {
            violations.extend(checker.check_item(item, "test.rs").unwrap());
        }

        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_no_panic_empty_function() {
        let code = r#"
            fn example() {
            }
        "#;

        let syntax = syn::parse_file(code).unwrap();
        let checker = E1001DirectPanic::default();

        let mut violations = Vec::new();
        for item in &syntax.items {
            violations.extend(checker.check_item(item, "test.rs").unwrap());
        }

        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_ignores_other_macros() {
        let code = r#"
            fn example() {
                println!("Not a panic");
                assert!(true);
                vec![1, 2, 3];
            }
        "#;

        let syntax = syn::parse_file(code).unwrap();
        let checker = E1001DirectPanic::default();

        let mut violations = Vec::new();
        for item in &syntax.items {
            violations.extend(checker.check_item(item, "test.rs").unwrap());
        }

        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_checker_disabled() {
        let code = r#"
            fn example() {
                panic!("Should not be detected");
            }
        "#;

        let syntax = syn::parse_file(code).unwrap();
        let config = E1001Config {
            enabled: false,
            severity: crate::config::SeverityLevel::High,
            categories: vec![crate::config::CheckerCategory::Operations],
        };
        let checker = {
            let mut c = E1001DirectPanic::default();
            c.set_config(Box::new(config)).unwrap();
            c
        };

        assert!(!checker.is_enabled());

        let mut violations = Vec::new();
        for item in &syntax.items {
            violations.extend(checker.check_item(item, "test.rs").unwrap());
        }

        // Checker still finds violations, but analyzer won't run disabled checkers
        assert_eq!(violations.len(), 1);
    }

    #[test]
    fn test_panic_with_format_args() {
        let code = r#"
            fn example(x: i32) {
                panic!("Value {} is invalid", x);
            }
        "#;

        let syntax = syn::parse_file(code).unwrap();
        let checker = E1001DirectPanic::default();

        let mut violations = Vec::new();
        for item in &syntax.items {
            violations.extend(checker.check_item(item, "test.rs").unwrap());
        }

        assert_eq!(violations.len(), 1);
    }

    #[test]
    fn test_panic_in_closure() {
        let code = r#"
            fn example() {
                let f = || {
                    panic!("Closure panic");
                };
            }
        "#;

        let syntax = syn::parse_file(code).unwrap();
        let checker = E1001DirectPanic::default();

        let mut violations = Vec::new();
        for item in &syntax.items {
            violations.extend(checker.check_item(item, "test.rs").unwrap());
        }

        assert_eq!(violations.len(), 1);
    }
}
