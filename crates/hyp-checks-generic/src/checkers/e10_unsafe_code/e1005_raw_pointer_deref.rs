//! E1005: Raw pointer dereference
//!
//! Detects dereferencing of raw pointers which is inherently unsafe
//! and can cause undefined behavior.

use crate::{checker::Checker, define_checker, violation::Violation};

use syn::visit::Visit;

define_checker! {
    /// Checker for E1005: Raw pointer dereference
    E1005RawPointerDeref,
    code = "E1005",
    name = "Raw pointer dereference",
    suggestions = "Use references instead of raw pointers, or use safe abstractions like Box, Rc, Arc",
    target_items = [Function, Impl],
    config_entry_name = "e1005_raw_pointer_deref",
    config = E1005Config {
        enabled: bool = true,
        severity: crate::config::SeverityLevel = crate::config::SeverityLevel::High,
        categories: Vec<crate::config::CheckerCategory> = vec![crate::config::CheckerCategory::Operations],
    },
    check_item(self, item, file_path) {
        let mut visitor = RawPointerDerefVisitor {
            violations: Vec::new(),
            file_path,
            checker: self,
        };
        visitor.visit_item(item);
        Ok(visitor.violations)
    }
}

struct RawPointerDerefVisitor<'a> {
    violations: Vec<Violation>,
    file_path: &'a str,
    checker: &'a E1005RawPointerDeref,
}

impl<'a> Visit<'a> for RawPointerDerefVisitor<'a> {
    fn visit_expr_unary(&mut self, node: &'a syn::ExprUnary) {
        use syn::spanned::Spanned;

        // Check for dereference operator *
        if matches!(node.op, syn::UnOp::Deref(_)) {
            // Check if we're dereferencing a raw pointer expression
            // Look for patterns like *ptr, *raw_ptr, etc.
            if is_potential_raw_pointer(&node.expr) {
                let start = node.span().start();
                self.violations.push(
                    Violation::new(
                        self.checker.code(),
                        self.checker.name(),
                        self.checker.severity().into(),
                        "Dereferencing a raw pointer. This is unsafe and can cause undefined behavior if the pointer is invalid, null, or dangling.",
                        self.file_path,
                        start.line,
                        start.column + 1,
                    )
                    .with_suggestion(self.checker.suggestions()),
                );
            }
        }

        syn::visit::visit_expr_unary(self, node);
    }
}

fn is_potential_raw_pointer(expr: &syn::Expr) -> bool {
    match expr {
        // Variable names containing "ptr" or "pointer"
        syn::Expr::Path(path) => {
            let path_str = path.path.segments.iter()
                .map(|s| s.ident.to_string().to_lowercase())
                .collect::<Vec<_>>()
                .join("::");
            path_str.contains("ptr") || path_str.contains("pointer")
        }
        // Field access like self.ptr
        syn::Expr::Field(field) => {
            if let syn::Member::Named(ident) = &field.member {
                let name = ident.to_string().to_lowercase();
                name.contains("ptr") || name.contains("pointer")
            } else {
                false
            }
        }
        // Cast expressions like (ptr as *const T)
        syn::Expr::Cast(_) => true,
        // Paren expressions
        syn::Expr::Paren(paren) => is_potential_raw_pointer(&paren.expr),
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detects_raw_pointer_deref() {
        let code = r#"
            fn bad() {
                let ptr: *const i32 = std::ptr::null();
                unsafe {
                    let _ = *ptr;
                }
            }
        "#;

        let syntax = syn::parse_file(code).unwrap();
        let checker = E1005RawPointerDeref::default();

        let mut violations = Vec::new();
        for item in &syntax.items {
            violations.extend(checker.check_item(item, "test.rs").unwrap());
        }

        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].code, "E1005");
    }

    #[test]
    fn test_normal_deref_passes() {
        let code = r#"
            fn good() {
                let x = Box::new(42);
                let value = *x;
            }
        "#;

        let syntax = syn::parse_file(code).unwrap();
        let checker = E1005RawPointerDeref::default();

        let mut violations = Vec::new();
        for item in &syntax.items {
            violations.extend(checker.check_item(item, "test.rs").unwrap());
        }

        assert_eq!(violations.len(), 0);
    }
}
