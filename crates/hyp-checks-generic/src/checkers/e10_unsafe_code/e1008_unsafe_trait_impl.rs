//! E1008: Unsafe trait implementation
//!
//! Detects implementations of unsafe traits like `Send`, `Sync`, or custom unsafe traits.
//! These require careful review as they make promises about type safety.

use crate::{checker::Checker, define_checker, violation::Violation};

use syn::{spanned::Spanned, visit::Visit};

define_checker! {
    /// Checker for E1008: Unsafe trait implementation
    E1008UnsafeTraitImpl,
    code = "E1008",
    name = "Unsafe trait implementation",
    suggestions = "Ensure the type truly satisfies the unsafe trait's safety requirements. Document why this is safe.",
    target_items = [Impl],
    config_entry_name = "e1008_unsafe_trait_impl",
    /// Configuration for E1008: Unsafe trait implementation checker
    config = E1008Config {
        /// Whether this checker is enabled
        enabled: bool = true,
        /// Severity level, defaults to High
        severity: crate::config::SeverityLevel = crate::config::SeverityLevel::High,
        /// Categories this checker belongs to
        categories: Vec<crate::config::CheckerCategory> = vec![crate::config::CheckerCategory::Operations],
    },
    check_item(self, item, file_path) {
        let mut visitor = UnsafeTraitVisitor {
            violations: Vec::new(),
            file_path,
            checker: self,
        };
        visitor.visit_item(item);
        Ok(visitor.violations)
    }
}

struct UnsafeTraitVisitor<'a> {
    violations: Vec<Violation>,
    file_path: &'a str,
    checker: &'a E1008UnsafeTraitImpl,
}

impl<'a> UnsafeTraitVisitor<'a> {
    fn create_violation(&self, span: proc_macro2::Span, trait_name: &str) -> Violation {
        let start = span.start();
        Violation::new(
            self.checker.code(),
            self.checker.name(),
            self.checker.severity().into(),
            format!(
                "Unsafe implementation of trait '{}'. This requires the implementor to uphold safety invariants.",
                trait_name
            ),
            self.file_path,
            start.line,
            start.column + 1,
        )
        .with_suggestion(self.checker.suggestions())
    }
}

impl<'a> Visit<'a> for UnsafeTraitVisitor<'a> {
    fn visit_item_impl(&mut self, node: &'a syn::ItemImpl) {
        // Check if this is an unsafe impl
        if node.unsafety.is_some() {
            if let Some((_, trait_path, _)) = &node.trait_ {
                let trait_name = trait_path
                    .segments
                    .iter()
                    .map(|s| s.ident.to_string())
                    .collect::<Vec<_>>()
                    .join("::");
                self.violations.push(self.create_violation(trait_path.span(), &trait_name));
            } else {
                // Unsafe impl without a trait (inherent impl) - unusual but flag it
                self.violations.push(self.create_violation(
                    node.impl_token.span,
                    "<inherent>",
                ));
            }
        }

        syn::visit::visit_item_impl(self, node);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detects_unsafe_send_impl() {
        let code = r#"
            struct MyType(*mut u8);
            unsafe impl Send for MyType {}
        "#;

        let syntax = syn::parse_file(code).unwrap();
        let checker = E1008UnsafeTraitImpl::default();

        let mut violations = Vec::new();
        for item in &syntax.items {
            violations.extend(checker.check_item(item, "test.rs").unwrap());
        }

        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].code, "E1008");
        assert!(violations[0].message.contains("Send"));
    }

    #[test]
    fn test_detects_unsafe_sync_impl() {
        let code = r#"
            struct MyType(*mut u8);
            unsafe impl Sync for MyType {}
        "#;

        let syntax = syn::parse_file(code).unwrap();
        let checker = E1008UnsafeTraitImpl::default();

        let mut violations = Vec::new();
        for item in &syntax.items {
            violations.extend(checker.check_item(item, "test.rs").unwrap());
        }

        assert_eq!(violations.len(), 1);
        assert!(violations[0].message.contains("Sync"));
    }

    #[test]
    fn test_safe_impl_passes() {
        let code = r#"
            struct MyType(i32);
            impl Clone for MyType {
                fn clone(&self) -> Self {
                    MyType(self.0)
                }
            }
        "#;

        let syntax = syn::parse_file(code).unwrap();
        let checker = E1008UnsafeTraitImpl::default();

        let mut violations = Vec::new();
        for item in &syntax.items {
            violations.extend(checker.check_item(item, "test.rs").unwrap());
        }

        assert_eq!(violations.len(), 0);
    }
}
