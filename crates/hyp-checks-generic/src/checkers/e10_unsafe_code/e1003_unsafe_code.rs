//! E1003: Direct use of unsafe code
//!
//! Detects direct use of `unsafe` blocks and functions.
//! Unsafe code bypasses Rust's safety guarantees and requires careful auditing.

use crate::{checker::Checker, define_checker, violation::Violation};

use syn::visit::Visit;

define_checker! {
    /// Checker for E1003: Direct use of unsafe code
    E1003UnsafeCode,
    code = "E1003",
    name = "Direct use of unsafe code",
    suggestions = "Avoid unsafe code in production code",
    target_items = [Function],
    config_entry_name = "e1003_unsafe_code",
    /// Configuration for E1003: Unsafe code checker
    config = E1003Config {
        /// Whether this checker is enabled
        enabled: bool = true,
        /// Severity level, defaults to High
        severity: crate::config::SeverityLevel = crate::config::SeverityLevel::High,
        /// Categories this checker belongs to
        categories: Vec<crate::config::CheckerCategory> = vec![crate::config::CheckerCategory::Operations],
    },
    // AST node item checker
    check_item(self, item, file_path) {
        let mut visitor = UnsafeVisitor {
            violations: Vec::new(),
            file_path,
            checker: self,
        };
        visitor.visit_item(item);
        Ok(visitor.violations)
    }
}

/// Visitor that traverses the AST looking for unsafe blocks
struct UnsafeVisitor<'a> {
    violations: Vec<Violation>,
    file_path: &'a str,
    checker: &'a E1003UnsafeCode,
}

impl<'a> UnsafeVisitor<'a> {
    fn create_violation(&self, span: proc_macro2::Span, message: &str) -> Violation {
        let start = span.start();
        Violation::new(
            self.checker.code(),
            self.checker.name(),
            self.checker.severity().into(),
            message,
            self.file_path,
            start.line,
            start.column.saturating_add(1),
        )
        .with_suggestion(self.checker.suggestions())
    }
}

impl<'a> Visit<'a> for UnsafeVisitor<'a> {
    fn visit_expr_unsafe(&mut self, node: &'a syn::ExprUnsafe) {
        self.violations.push(self.create_violation(
            node.unsafe_token.span,
            "Direct use of unsafe block. Unsafe code bypasses Rust's safety guarantees and can lead to undefined behavior.",
        ));

        // Continue visiting nested expressions
        syn::visit::visit_expr_unsafe(self, node);
    }

    fn visit_item_fn(&mut self, node: &'a syn::ItemFn) {
        // Check if the function itself is marked unsafe
        if node.sig.unsafety.is_some() {
            self.violations.push(self.create_violation(
                node.sig.ident.span(),
                &format!(
                    "Function '{}' is declared unsafe. Unsafe functions require callers to uphold safety invariants.",
                    node.sig.ident
                ),
            ));
        }

        // Continue visiting the function body
        syn::visit::visit_item_fn(self, node);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detects_unsafe_block() {
        let code = r#"
            fn example() {
                let x = 42;
                let ptr = &x as *const i32;
                unsafe {
                    let _value = *ptr;
                }
            }
        "#;

        let syntax = syn::parse_file(code).unwrap();
        let checker = E1003UnsafeCode::default();

        let mut violations = Vec::new();
        for item in &syntax.items {
            violations.extend(checker.check_item(item, "test.rs").unwrap());
        }

        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].code, "E1003");
        assert!(violations[0].message.contains("unsafe block"));
    }

    #[test]
    fn test_detects_unsafe_function() {
        let code = r#"
            unsafe fn dangerous() {
                // Unsafe operations here
            }
        "#;

        let syntax = syn::parse_file(code).unwrap();
        let checker = E1003UnsafeCode::default();

        let mut violations = Vec::new();
        for item in &syntax.items {
            violations.extend(checker.check_item(item, "test.rs").unwrap());
        }

        assert_eq!(violations.len(), 1);
        assert!(violations[0].message.contains("declared unsafe"));
    }

    #[test]
    fn test_detects_multiple_unsafe_blocks() {
        let code = r#"
            fn example() {
                unsafe { let _ = 1; }
                unsafe { let _ = 2; }
            }
        "#;

        let syntax = syn::parse_file(code).unwrap();
        let checker = E1003UnsafeCode::default();

        let mut violations = Vec::new();
        for item in &syntax.items {
            violations.extend(checker.check_item(item, "test.rs").unwrap());
        }

        assert_eq!(violations.len(), 2);
    }

    #[test]
    fn test_safe_code_passes() {
        let code = r#"
            fn example() {
                let x = 42;
                let y = x + 1;
                println!("{}", y);
            }
        "#;

        let syntax = syn::parse_file(code).unwrap();
        let checker = E1003UnsafeCode::default();

        let mut violations = Vec::new();
        for item in &syntax.items {
            violations.extend(checker.check_item(item, "test.rs").unwrap());
        }

        assert_eq!(violations.len(), 0);
    }
}
