//! E1409: Partial initialization
//!
//! Detects patterns that might leave data partially initialized, such as
//! using MaybeUninit or uninitialized arrays.

use crate::{checker::Checker, define_checker, violation::Violation};

use syn::{spanned::Spanned, visit::Visit};

define_checker! {
    /// Checker for E1409: Partial initialization
    E1409PartialInitialization,
    code = "E1409",
    name = "Partial initialization",
    suggestions = "Use Default::default() or explicit initialization. For performance-critical code, document the initialization invariant.",
    target_items = [Function],
    config_entry_name = "e1409_partial_initialization",
    /// Configuration for E1409: Partial initialization checker
    config = E1409Config {
        /// Whether this checker is enabled
        enabled: bool = true,
        /// Severity level, defaults to Medium
        severity: crate::config::SeverityLevel = crate::config::SeverityLevel::Medium,
        /// Categories this checker belongs to
        categories: Vec<crate::config::CheckerCategory> = vec![crate::config::CheckerCategory::Operations],
    },
    check_item(self, item, file_path) {
        let mut visitor = InitializationVisitor {
            violations: Vec::new(),
            file_path,
            checker: self,
        };
        visitor.visit_item(item);
        Ok(visitor.violations)
    }
}

struct InitializationVisitor<'a> {
    violations: Vec<Violation>,
    file_path: &'a str,
    checker: &'a E1409PartialInitialization,
}

impl<'a> InitializationVisitor<'a> {
    fn create_violation(&self, span: proc_macro2::Span, pattern: &str) -> Violation {
        let start = span.start();
        Violation::new(
            self.checker.code(),
            self.checker.name(),
            self.checker.severity().into(),
            format!(
                "Use of {} can lead to reading uninitialized memory if not handled carefully.",
                pattern
            ),
            self.file_path,
            start.line,
            start.column + 1,
        )
        .with_suggestion(self.checker.suggestions())
    }
}

impl<'a> Visit<'a> for InitializationVisitor<'a> {
    fn visit_expr_call(&mut self, node: &'a syn::ExprCall) {
        // Check for MaybeUninit::uninit() or similar
        if let syn::Expr::Path(path) = &*node.func {
            let path_str = path.path.segments.iter()
                .map(|s| s.ident.to_string())
                .collect::<Vec<_>>()
                .join("::");

            if path_str.contains("uninit") || path_str.contains("MaybeUninit") {
                self.violations.push(self.create_violation(node.span(), "MaybeUninit"));
            }

            if path_str.contains("zeroed") {
                self.violations.push(self.create_violation(node.span(), "mem::zeroed()"));
            }
        }

        syn::visit::visit_expr_call(self, node);
    }

    fn visit_expr_method_call(&mut self, node: &'a syn::ExprMethodCall) {
        let method = node.method.to_string();

        // Check for assume_init() which is the dangerous part
        if method == "assume_init" || method == "assume_init_read" || method == "assume_init_ref" {
            self.violations.push(self.create_violation(node.span(), &format!("{}()", method)));
        }

        syn::visit::visit_expr_method_call(self, node);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detects_maybe_uninit() {
        let code = r#"
            use std::mem::MaybeUninit;

            fn example() {
                let x: MaybeUninit<i32> = MaybeUninit::uninit();
            }
        "#;

        let syntax = syn::parse_file(code).unwrap();
        let checker = E1409PartialInitialization::default();

        let mut violations = Vec::new();
        for item in &syntax.items {
            violations.extend(checker.check_item(item, "test.rs").unwrap());
        }

        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].code, "E1409");
    }

    #[test]
    fn test_detects_assume_init() {
        let code = r#"
            fn example() {
                let x = unsafe { uninit_value.assume_init() };
            }
        "#;

        let syntax = syn::parse_file(code).unwrap();
        let checker = E1409PartialInitialization::default();

        let mut violations = Vec::new();
        for item in &syntax.items {
            violations.extend(checker.check_item(item, "test.rs").unwrap());
        }

        assert_eq!(violations.len(), 1);
    }

    #[test]
    fn test_normal_initialization_passes() {
        let code = r#"
            fn example() {
                let x: i32 = 0;
                let y = Default::default();
                let arr = [0; 10];
            }
        "#;

        let syntax = syn::parse_file(code).unwrap();
        let checker = E1409PartialInitialization::default();

        let mut violations = Vec::new();
        for item in &syntax.items {
            violations.extend(checker.check_item(item, "test.rs").unwrap());
        }

        assert_eq!(violations.len(), 0);
    }
}
