//! E1011: Uninitialized memory access
//!
//! Detects use of MaybeUninit::uninit() or mem::uninitialized() which can lead to
//! undefined behavior if the memory is read before being initialized.

use crate::{checker::Checker, define_checker, violation::Violation};

use syn::visit::Visit;

define_checker! {
    /// Checker for E1011: Uninitialized memory
    E1011UninitializedMemory,
    code = "E1011",
    name = "Uninitialized memory access",
    suggestions = "Use MaybeUninit::zeroed() or proper initialization. Never read from MaybeUninit before calling assume_init().",
    target_items = [Function, Impl],
    config_entry_name = "e1011_uninitialized_memory",
    /// Configuration for E1011: Uninitialized memory checker
    config = E1011Config {
        /// Whether this checker is enabled
        enabled: bool = true,
        /// Severity level, defaults to High
        severity: crate::config::SeverityLevel = crate::config::SeverityLevel::High,
        /// Categories this checker belongs to
        categories: Vec<crate::config::CheckerCategory> = vec![crate::config::CheckerCategory::Operations],
    },
    check_item(self, item, file_path) {
        let mut visitor = UninitializedMemoryVisitor {
            violations: Vec::new(),
            file_path,
            checker: self,
        };
        visitor.visit_item(item);
        Ok(visitor.violations)
    }
}

struct UninitializedMemoryVisitor<'a> {
    violations: Vec<Violation>,
    file_path: &'a str,
    checker: &'a E1011UninitializedMemory,
}


impl<'a> Visit<'a> for UninitializedMemoryVisitor<'a> {
    fn visit_expr_call(&mut self, node: &'a syn::ExprCall) {
        use syn::spanned::Spanned;

        // Check for MaybeUninit::uninit() or mem::uninitialized()
        if let syn::Expr::Path(path) = &*node.func {
            let path_str = path.path.segments.iter()
                .map(|s| s.ident.to_string())
                .collect::<Vec<_>>()
                .join("::");

            if path_str.contains("uninit") && !path_str.contains("zeroed") {
                let start = node.span().start();
                self.violations.push(
                    Violation::new(
                        self.checker.code(),
                        self.checker.name(),
                        self.checker.severity().into(),
                        format!(
                            "Using uninitialized memory ({}). Reading uninitialized memory is undefined behavior. Use MaybeUninit::zeroed() or proper initialization.",
                            path_str
                        ),
                        self.file_path,
                        start.line,
                        start.column + 1,
                    )
                    .with_suggestion(self.checker.suggestions()),
                );
            }
        }

        syn::visit::visit_expr_call(self, node);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detects_maybe_uninit_uninit() {
        let code = r#"
            use std::mem::MaybeUninit;

            fn bad() {
                let x: MaybeUninit<i32> = MaybeUninit::uninit();
            }
        "#;

        let syntax = syn::parse_file(code).unwrap();
        let checker = E1011UninitializedMemory::default();

        let mut violations = Vec::new();
        for item in &syntax.items {
            violations.extend(checker.check_item(item, "test.rs").unwrap());
        }

        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].code, "E1011");
        assert!(violations[0].message.contains("uninit"));
    }

    #[test]
    fn test_zeroed_passes() {
        let code = r#"
            use std::mem::MaybeUninit;

            fn good() {
                let x: MaybeUninit<i32> = MaybeUninit::zeroed();
            }
        "#;

        let syntax = syn::parse_file(code).unwrap();
        let checker = E1011UninitializedMemory::default();

        let mut violations = Vec::new();
        for item in &syntax.items {
            violations.extend(checker.check_item(item, "test.rs").unwrap());
        }

        assert_eq!(violations.len(), 0);
    }
}
