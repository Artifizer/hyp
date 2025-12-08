//! E1012: Unsafe auto trait implementation
//!
//! Detects manual implementation of unsafe auto traits (Send, Sync, Unpin).
//! These traits should be automatically derived by the compiler.

use crate::{checker::Checker, define_checker, violation::Violation};

use syn::visit::Visit;

define_checker! {
    /// Checker for E1012: Unsafe auto trait implementation
    E1012UnsafeAutoTrait,
    code = "E1012",
    name = "Unsafe auto trait implementation",
    suggestions = "Let the compiler automatically implement Send/Sync/Unpin. Use safe wrappers like Arc, Mutex instead of manual unsafe impl.",
    target_items = [Impl],
    config_entry_name = "e1012_unsafe_auto_trait",
    /// Configuration for E1012: Unsafe auto trait implementation checker
    config = E1012Config {
        /// Whether this checker is enabled
        enabled: bool = true,
        /// Severity level, defaults to High
        severity: crate::config::SeverityLevel = crate::config::SeverityLevel::High,
        /// Categories this checker belongs to
        categories: Vec<crate::config::CheckerCategory> = vec![crate::config::CheckerCategory::Operations],
    },
    check_item(self, item, file_path) {
        let mut visitor = UnsafeAutoTraitVisitor {
            violations: Vec::new(),
            file_path,
            checker: self,
        };
        visitor.visit_item(item);
        Ok(visitor.violations)
    }
}

struct UnsafeAutoTraitVisitor<'a> {
    violations: Vec<Violation>,
    file_path: &'a str,
    checker: &'a E1012UnsafeAutoTrait,
}

impl<'a> UnsafeAutoTraitVisitor<'a> {
    fn is_auto_trait(&self, path: &syn::Path) -> Option<String> {
        let trait_name = path.segments.last()?.ident.to_string();

        if trait_name == "Send" || trait_name == "Sync" || trait_name == "Unpin" {
            Some(trait_name)
        } else {
            None
        }
    }
}

impl<'a> Visit<'a> for UnsafeAutoTraitVisitor<'a> {
    fn visit_item_impl(&mut self, node: &'a syn::ItemImpl) {
        use syn::spanned::Spanned;

        // Check if this is implementing a trait
        if let Some((_, trait_path, _)) = &node.trait_ {
            if let Some(trait_name) = self.is_auto_trait(trait_path) {
                // Check if it's an unsafe impl
                if node.unsafety.is_some() {
                    let start = trait_path.span().start();
                    self.violations.push(
                        Violation::new(
                            self.checker.code(),
                            self.checker.name(),
                            self.checker.severity().into(),
                            format!(
                                "Manual unsafe implementation of auto trait '{}'. Auto traits should be automatically derived by the compiler. Manual implementation can cause undefined behavior if safety invariants are violated.",
                                trait_name
                            ),
                            self.file_path,
                            start.line,
                            start.column + 1,
                        )
                        .with_suggestion(self.checker.suggestions()),
                    );
                } else {
                    // Even safe impl of Unpin is suspicious
                    if trait_name == "Unpin" {
                        let start = trait_path.span().start();
                        self.violations.push(
                            Violation::new(
                                self.checker.code(),
                                self.checker.name(),
                                self.checker.severity().into(),
                                format!(
                                    "Manual implementation of auto trait '{}'. Auto traits should be automatically derived by the compiler. Manual implementation can cause subtle bugs with pinning.",
                                    trait_name
                                ),
                                self.file_path,
                                start.line,
                                start.column + 1,
                            )
                            .with_suggestion(self.checker.suggestions()),
                        );
                    }
                }
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
            struct MyType {
                ptr: *mut i32,
            }

            unsafe impl Send for MyType {}
        "#;

        let syntax = syn::parse_file(code).unwrap();
        let checker = E1012UnsafeAutoTrait::default();

        let mut violations = Vec::new();
        for item in &syntax.items {
            violations.extend(checker.check_item(item, "test.rs").unwrap());
        }

        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].code, "E1012");
        assert!(violations[0].message.contains("Send"));
    }

    #[test]
    fn test_detects_unsafe_sync_impl() {
        let code = r#"
            struct MyType {
                ptr: *mut i32,
            }

            unsafe impl Sync for MyType {}
        "#;

        let syntax = syn::parse_file(code).unwrap();
        let checker = E1012UnsafeAutoTrait::default();

        let mut violations = Vec::new();
        for item in &syntax.items {
            violations.extend(checker.check_item(item, "test.rs").unwrap());
        }

        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].code, "E1012");
        assert!(violations[0].message.contains("Sync"));
    }

    #[test]
    fn test_detects_unpin_impl() {
        let code = r#"
            struct MyType {
                data: *mut i32,
            }

            impl Unpin for MyType {}
        "#;

        let syntax = syn::parse_file(code).unwrap();
        let checker = E1012UnsafeAutoTrait::default();

        let mut violations = Vec::new();
        for item in &syntax.items {
            violations.extend(checker.check_item(item, "test.rs").unwrap());
        }

        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].code, "E1012");
        assert!(violations[0].message.contains("Unpin"));
    }

    #[test]
    fn test_normal_trait_impl_passes() {
        let code = r#"
            trait MyTrait {
                fn foo(&self);
            }

            struct MyType;

            impl MyTrait for MyType {
                fn foo(&self) {}
            }
        "#;

        let syntax = syn::parse_file(code).unwrap();
        let checker = E1012UnsafeAutoTrait::default();

        let mut violations = Vec::new();
        for item in &syntax.items {
            violations.extend(checker.check_item(item, "test.rs").unwrap());
        }

        assert_eq!(violations.len(), 0);
    }
}
