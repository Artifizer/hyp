//! E1603: Dangling reference patterns
//!
//! Detects patterns that commonly lead to dangling references:
//! - Returning references to local variables (via unsafe)
//! - Creating references from raw pointers without lifetime guarantees
//! - Using transmute to extend lifetimes
//!
//! Note: Rust's borrow checker prevents most dangling references, but unsafe
//! code can bypass these protections. This checker catches common unsafe patterns.

use crate::{checker::Checker, define_checker, violation::Violation};

use syn::visit::Visit;

define_checker! {
    /// Checker for E1603: Dangling reference detection
    E1603DanglingReference,
    code = "E1603",
    name = "Dangling reference pattern",
    suggestions = "Ensure references outlive their referents. Use owned types or Arc/Rc for shared ownership.",
    target_items = [Function],
    config_entry_name = "e1603_dangling_reference",
    config = E1603Config {
        enabled: bool = true,
        severity: crate::config::SeverityLevel = crate::config::SeverityLevel::High,
        categories: Vec<crate::config::CheckerCategory> = vec![crate::config::CheckerCategory::Operations],
    },
    check_item(self, item, file_path) {
        let mut visitor = DanglingRefVisitor {
            violations: Vec::new(),
            file_path,
            checker: self,
            in_unsafe: false,
        };
        visitor.visit_item(item);
        Ok(visitor.violations)
    }
}

struct DanglingRefVisitor<'a> {
    violations: Vec<Violation>,
    file_path: &'a str,
    checker: &'a E1603DanglingReference,
    in_unsafe: bool,
}

impl<'a> Visit<'a> for DanglingRefVisitor<'a> {
    fn visit_expr_unsafe(&mut self, node: &'a syn::ExprUnsafe) {
        self.in_unsafe = true;
        syn::visit::visit_expr_unsafe(self, node);
        self.in_unsafe = false;
    }

    fn visit_expr_method_call(&mut self, node: &'a syn::ExprMethodCall) {
        let method_name = node.method.to_string();

        // Detect dangerous patterns that can create dangling references
        if self.in_unsafe
            && matches!(method_name.as_str(), "as_ref" | "as_mut") {
                // Raw pointer to reference conversion
                let start = node.method.span().start();
                self.violations.push(
                    Violation::new(
                        self.checker.code(),
                        self.checker.name(),
                        self.checker.severity().into(),
                        "Converting raw pointer to reference in unsafe block. Ensure the pointer is valid and properly aligned.",
                        self.file_path,
                        start.line,
                        start.column + 1,
                    )
                    .with_suggestion(self.checker.suggestions()),
                );
            }

        syn::visit::visit_expr_method_call(self, node);
    }

    fn visit_expr_call(&mut self, node: &'a syn::ExprCall) {
        // Check for transmute that might extend lifetimes
        if let syn::Expr::Path(path) = &*node.func {
            let path_str = path.path.segments.iter()
                .map(|s| s.ident.to_string())
                .collect::<Vec<_>>()
                .join("::");

            if path_str.contains("transmute") && self.in_unsafe {
                use syn::spanned::Spanned;
                let start = node.span().start();
                self.violations.push(
                    Violation::new(
                        self.checker.code(),
                        self.checker.name(),
                        self.checker.severity().into(),
                        "Using transmute in unsafe block. This can create dangling references if used to extend lifetimes.",
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
    use crate::checker::Checker;

    #[test]
    fn test_detects_as_ref_in_unsafe() {
        let code = r#"
            fn dangerous(ptr: *const i32) -> &'static i32 {
                unsafe {
                    ptr.as_ref().unwrap()
                }
            }
        "#;

        let syntax = syn::parse_file(code).unwrap();
        let checker = E1603DanglingReference::default();

        let mut violations = Vec::new();
        for item in &syntax.items {
            violations.extend(checker.check_item(item, "test.rs").unwrap());
        }

        assert_eq!(violations.len(), 1);
        assert!(violations[0].message.contains("raw pointer"));
    }

    #[test]
    fn test_detects_transmute_in_unsafe() {
        let code = r#"
            fn extend_lifetime<'a>(s: &'a str) -> &'static str {
                unsafe {
                    std::mem::transmute(s)
                }
            }
        "#;

        let syntax = syn::parse_file(code).unwrap();
        let checker = E1603DanglingReference::default();

        let mut violations = Vec::new();
        for item in &syntax.items {
            violations.extend(checker.check_item(item, "test.rs").unwrap());
        }

        assert_eq!(violations.len(), 1);
        assert!(violations[0].message.contains("transmute"));
    }

    #[test]
    fn test_safe_code_passes() {
        let code = r#"
            fn safe_ref(data: &[i32]) -> Option<&i32> {
                data.first()
            }
        "#;

        let syntax = syn::parse_file(code).unwrap();
        let checker = E1603DanglingReference::default();

        let mut violations = Vec::new();
        for item in &syntax.items {
            violations.extend(checker.check_item(item, "test.rs").unwrap());
        }

        assert_eq!(violations.len(), 0);
    }
}
