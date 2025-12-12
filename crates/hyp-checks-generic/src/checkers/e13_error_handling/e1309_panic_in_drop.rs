//! E1309: Panic in Drop implementation
//!
//! Detects panic-inducing code in Drop implementations. Panicking during
//! drop can cause double-panic which aborts the program.
//!
//! Example:
//! ```text
//! impl Drop for Resource {
//!     fn drop(&mut self) {
//!         // Bad: Panicking in drop
//!         self.handle.close().unwrap();  // Can panic!
//!
//!         // Bad: Direct panic
//!         if self.dirty {
//!             panic!("Resource not saved!");
//!         }
//!     }
//! }
//! ```

use crate::{checker::Checker, define_checker, violation::Violation};
use syn::visit::Visit;

define_checker! {
    /// Checker for E1309: Panic in Drop implementation
    E1309PanicInDrop,
    code = "E1309",
    name = "Panic in Drop implementation",
    suggestions = "Never panic in Drop. Use Result-returning cleanup methods, log errors, or silently ignore non-critical failures.",
    target_items = [Impl],
    config_entry_name = "e1309_panic_in_drop",
    config = E1309Config {
        enabled: bool = true,
        severity: crate::config::SeverityLevel = crate::config::SeverityLevel::High,
        categories: Vec<crate::config::CheckerCategory> = vec![crate::config::CheckerCategory::Operations],
    },
    check_item(self, item, file_path) {
        let mut violations = Vec::new();

        if let syn::Item::Impl(impl_block) = item {
            // Check if this is a Drop implementation
            if Self::is_drop_impl(impl_block) {
                // Find the drop method and check it
                for item in &impl_block.items {
                    if let syn::ImplItem::Fn(method) = item {
                        if method.sig.ident == "drop" {
                            let mut visitor = PanicVisitor {
                                violations: Vec::new(),
                                file_path,
                                checker: self,
                            };
                            visitor.visit_block(&method.block);
                            violations.extend(visitor.violations);
                        }
                    }
                }
            }
        }

        Ok(violations)
    }
}

impl E1309PanicInDrop {
    fn is_drop_impl(impl_block: &syn::ItemImpl) -> bool {
        if let Some((_, trait_path, _)) = &impl_block.trait_ {
            let trait_name = trait_path
                .segments
                .last()
                .map(|s| s.ident.to_string())
                .unwrap_or_default();

            return trait_name == "Drop";
        }
        false
    }
}

struct PanicVisitor<'a> {
    violations: Vec<Violation>,
    file_path: &'a str,
    checker: &'a E1309PanicInDrop,
}

impl<'a> PanicVisitor<'a> {
    fn check_macro(&mut self, mac: &syn::Macro, span: proc_macro2::Span) {
        let macro_name = mac
            .path
            .segments
            .last()
            .map(|s| s.ident.to_string())
            .unwrap_or_default();

        let is_panic_macro = matches!(
            macro_name.as_str(),
            "panic" | "unreachable" | "unimplemented" | "todo"
        );

        if is_panic_macro {
            let start = span.start();
            self.violations.push(
                Violation::new(
                    self.checker.code(),
                    self.checker.name(),
                    self.checker.severity().into(),
                    format!(
                        "{}!() in Drop implementation. Panicking during drop can cause abort.",
                        macro_name
                    ),
                    self.file_path,
                    start.line,
                    start.column + 1,
                )
                .with_suggestion(self.checker.suggestions()),
            );
            return;
        }

        // Check assert macros (these can panic)
        let is_assert_macro = matches!(
            macro_name.as_str(),
            "assert" | "assert_eq" | "assert_ne" | "debug_assert" | "debug_assert_eq" | "debug_assert_ne"
        );

        if is_assert_macro {
            let start = span.start();
            self.violations.push(
                Violation::new(
                    self.checker.code(),
                    self.checker.name(),
                    crate::violation::CheckerSeverity::Medium.into(),
                    format!(
                        "{}!() in Drop implementation. Assertions can panic.",
                        macro_name
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

impl<'a> Visit<'a> for PanicVisitor<'a> {
    fn visit_expr_macro(&mut self, node: &'a syn::ExprMacro) {
        use syn::spanned::Spanned;
        self.check_macro(&node.mac, node.span());
        syn::visit::visit_expr_macro(self, node);
    }

    fn visit_stmt_macro(&mut self, node: &'a syn::StmtMacro) {
        use syn::spanned::Spanned;
        self.check_macro(&node.mac, node.span());
        syn::visit::visit_stmt_macro(self, node);
    }

    fn visit_expr_method_call(&mut self, node: &'a syn::ExprMethodCall) {
        use syn::spanned::Spanned;

        let method_name = node.method.to_string();

        // Check for unwrap/expect calls
        if matches!(method_name.as_str(), "unwrap" | "expect") {
            let start = node.span().start();
            self.violations.push(
                Violation::new(
                    self.checker.code(),
                    self.checker.name(),
                    self.checker.severity().into(),
                    format!(
                        ".{}() in Drop implementation. This can panic and cause abort.",
                        method_name
                    ),
                    self.file_path,
                    start.line,
                    start.column + 1,
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
    use crate::checker::Checker;

    #[test]
    fn test_detects_panic_in_drop() {
        let code = r#"
            struct Resource;

            impl Drop for Resource {
                fn drop(&mut self) {
                    panic!("cleanup failed");
                }
            }
        "#;

        let syntax = syn::parse_file(code).unwrap();
        let checker = E1309PanicInDrop::default();

        let mut violations = Vec::new();
        for item in &syntax.items {
            violations.extend(checker.check_item(item, "test.rs").unwrap());
        }

        assert_eq!(violations.len(), 1);
        assert!(violations[0].message.contains("panic!()"));
    }

    #[test]
    fn test_detects_unwrap_in_drop() {
        let code = r#"
            struct Resource {
                handle: Option<i32>,
            }

            impl Drop for Resource {
                fn drop(&mut self) {
                    let _ = self.handle.unwrap();
                }
            }
        "#;

        let syntax = syn::parse_file(code).unwrap();
        let checker = E1309PanicInDrop::default();

        let mut violations = Vec::new();
        for item in &syntax.items {
            violations.extend(checker.check_item(item, "test.rs").unwrap());
        }

        assert_eq!(violations.len(), 1);
        assert!(violations[0].message.contains(".unwrap()"));
    }

    #[test]
    fn test_detects_assert_in_drop() {
        let code = r#"
            struct Resource {
                valid: bool,
            }

            impl Drop for Resource {
                fn drop(&mut self) {
                    assert!(self.valid);
                }
            }
        "#;

        let syntax = syn::parse_file(code).unwrap();
        let checker = E1309PanicInDrop::default();

        let mut violations = Vec::new();
        for item in &syntax.items {
            violations.extend(checker.check_item(item, "test.rs").unwrap());
        }

        assert_eq!(violations.len(), 1);
        assert!(violations[0].message.contains("assert!()"));
    }

    #[test]
    fn test_safe_drop_passes() {
        let code = r#"
            struct Resource {
                handle: Option<i32>,
            }

            impl Drop for Resource {
                fn drop(&mut self) {
                    if let Some(h) = self.handle.take() {
                        // Safe cleanup
                        let _ = h;
                    }
                }
            }
        "#;

        let syntax = syn::parse_file(code).unwrap();
        let checker = E1309PanicInDrop::default();

        let mut violations = Vec::new();
        for item in &syntax.items {
            violations.extend(checker.check_item(item, "test.rs").unwrap());
        }

        assert!(violations.is_empty());
    }

    #[test]
    fn test_non_drop_impl_passes() {
        let code = r#"
            struct Resource;

            impl Resource {
                fn cleanup(&mut self) {
                    panic!("failed");  // OK in non-Drop
                }
            }
        "#;

        let syntax = syn::parse_file(code).unwrap();
        let checker = E1309PanicInDrop::default();

        let mut violations = Vec::new();
        for item in &syntax.items {
            violations.extend(checker.check_item(item, "test.rs").unwrap());
        }

        assert!(violations.is_empty());
    }
}
