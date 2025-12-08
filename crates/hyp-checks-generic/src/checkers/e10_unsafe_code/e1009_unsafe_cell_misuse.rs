//! E1009: UnsafeCell misuse and interior mutability violations
//!
//! Detects direct use of UnsafeCell without proper safety wrappers.
//! UnsafeCell is Rust's primitive for interior mutability but provides NO safety guarantees.

use crate::{checker::Checker, define_checker, violation::Violation};

use syn::visit::Visit;

define_checker! {
    /// Checker for E1009: UnsafeCell misuse
    E1009UnsafeCellMisuse,
    code = "E1009",
    name = "UnsafeCell misuse",
    suggestions = "Use safe wrappers like Cell<T>, RefCell<T>, Mutex<T>, or RwLock<T> instead of UnsafeCell directly",
    target_items = [Struct, Function, Impl],
    config_entry_name = "e1009_unsafe_cell_misuse",
    /// Configuration for E1009: UnsafeCell misuse checker
    config = E1009Config {
        /// Whether this checker is enabled
        enabled: bool = true,
        /// Severity level, defaults to High
        severity: crate::config::SeverityLevel = crate::config::SeverityLevel::High,
        /// Categories this checker belongs to
        categories: Vec<crate::config::CheckerCategory> = vec![crate::config::CheckerCategory::Operations],
    },
    check_item(self, item, file_path) {
        let mut visitor = UnsafeCellVisitor {
            violations: Vec::new(),
            file_path,
            checker: self,
        };
        visitor.visit_item(item);
        Ok(visitor.violations)
    }
}

struct UnsafeCellVisitor<'a> {
    violations: Vec<Violation>,
    file_path: &'a str,
    checker: &'a E1009UnsafeCellMisuse,
}

impl<'a> UnsafeCellVisitor<'a> {
    fn is_unsafe_cell(&self, ty: &syn::Type) -> bool {
        if let syn::Type::Path(type_path) = ty {
            let path_str = type_path
                .path
                .segments
                .iter()
                .map(|s| s.ident.to_string())
                .collect::<Vec<_>>()
                .join("::");

            return path_str == "UnsafeCell"
                || path_str == "std::cell::UnsafeCell"
                || path_str == "core::cell::UnsafeCell"
                || path_str.ends_with("::UnsafeCell");
        }
        false
    }
}

impl<'a> Visit<'a> for UnsafeCellVisitor<'a> {
    fn visit_field(&mut self, node: &'a syn::Field) {
        use syn::spanned::Spanned;

        if self.is_unsafe_cell(&node.ty) {
            let start = node.ty.span().start();
            let field_name = node.ident.as_ref()
                .map(|i| i.to_string())
                .unwrap_or_else(|| "unnamed".to_string());

            self.violations.push(
                Violation::new(
                    self.checker.code(),
                    self.checker.name(),
                    self.checker.severity().into(),
                    format!(
                        "Field '{}' uses UnsafeCell directly. UnsafeCell provides no safety guarantees and can create aliasing mutable references. Use Cell, RefCell, Mutex, or RwLock instead.",
                        field_name
                    ),
                    self.file_path,
                    start.line,
                    start.column + 1,
                )
                .with_suggestion(self.checker.suggestions()),
            );
        }

        syn::visit::visit_field(self, node);
    }

    fn visit_local(&mut self, node: &'a syn::Local) {
        use syn::spanned::Spanned;

        // Check if local variable has UnsafeCell type
        if let syn::Pat::Type(pat_type) = &node.pat {
            if self.is_unsafe_cell(&pat_type.ty) {
                let start = pat_type.ty.span().start();
                self.violations.push(
                    Violation::new(
                        self.checker.code(),
                        self.checker.name(),
                        self.checker.severity().into(),
                        "Using UnsafeCell directly. UnsafeCell provides no safety guarantees. Use Cell, RefCell, Mutex, or RwLock instead.",
                        self.file_path,
                        start.line,
                        start.column + 1,
                    )
                    .with_suggestion(self.checker.suggestions()),
                );
            }
        }

        syn::visit::visit_local(self, node);
    }

    fn visit_expr_method_call(&mut self, node: &'a syn::ExprMethodCall) {
        use syn::spanned::Spanned;

        // Detect calls to .get() which is the UnsafeCell method
        let method_name = node.method.to_string();
        if method_name == "get" {
            // Check if receiver might be an UnsafeCell (heuristic: field named "value" or contains "cell")
            if let syn::Expr::Field(field) = &*node.receiver {
                if let syn::Member::Named(ident) = &field.member {
                    let field_name = ident.to_string().to_lowercase();
                    if field_name.contains("value") || field_name.contains("cell") {
                        let start = node.span().start();
                        self.violations.push(
                            Violation::new(
                                self.checker.code(),
                                self.checker.name(),
                                self.checker.severity().into(),
                                "Calling .get() on UnsafeCell. This returns a raw pointer that can create aliasing mutable references. Use Cell, RefCell, Mutex, or RwLock instead.",
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

        syn::visit::visit_expr_method_call(self, node);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detects_unsafe_cell_field() {
        let code = r#"
            use std::cell::UnsafeCell;

            struct BadCell<T> {
                value: UnsafeCell<T>,
            }
        "#;

        let syntax = syn::parse_file(code).unwrap();
        let checker = E1009UnsafeCellMisuse::default();

        let mut violations = Vec::new();
        for item in &syntax.items {
            violations.extend(checker.check_item(item, "test.rs").unwrap());
        }

        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].code, "E1009");
        assert!(violations[0].message.contains("UnsafeCell"));
    }

    #[test]
    fn test_safe_wrappers_pass() {
        let code = r#"
            use std::cell::{Cell, RefCell};
            use std::sync::Mutex;

            struct SafeCell<T> {
                value: RefCell<T>,
                counter: Cell<i32>,
                data: Mutex<T>,
            }
        "#;

        let syntax = syn::parse_file(code).unwrap();
        let checker = E1009UnsafeCellMisuse::default();

        let mut violations = Vec::new();
        for item in &syntax.items {
            violations.extend(checker.check_item(item, "test.rs").unwrap());
        }

        assert_eq!(violations.len(), 0);
    }
}
