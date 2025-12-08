//! E1609: Invalid slice creation
//!
//! Detects patterns that can create invalid slices:
//! - slice::from_raw_parts with unchecked parameters
//! - Creating slices from null or dangling pointers
//! - slice::from_raw_parts_mut misuse
//!
//! Creating a slice from invalid memory is undefined behavior.

use crate::{checker::Checker, define_checker, violation::Violation};
use syn::visit::Visit;

define_checker! {
    /// Checker for E1609: Invalid slice creation
    E1609InvalidSlice,
    code = "E1609",
    name = "Invalid slice creation",
    suggestions = "Validate pointer is non-null, properly aligned, and points to valid memory before creating slice.",
    target_items = [Function],
    config_entry_name = "e1609_invalid_slice",
    config = E1609Config {
        enabled: bool = true,
        severity: crate::config::SeverityLevel = crate::config::SeverityLevel::High,
        categories: Vec<crate::config::CheckerCategory> = vec![crate::config::CheckerCategory::Operations],
    },
    check_item(self, item, file_path) {
        let mut visitor = SliceVisitor {
            violations: Vec::new(),
            file_path,
            checker: self,
        };
        visitor.visit_item(item);
        Ok(visitor.violations)
    }
}

struct SliceVisitor<'a> {
    violations: Vec<Violation>,
    file_path: &'a str,
    checker: &'a E1609InvalidSlice,
}

impl<'a> Visit<'a> for SliceVisitor<'a> {
    fn visit_expr_call(&mut self, node: &'a syn::ExprCall) {
        if let syn::Expr::Path(path) = &*node.func {
            let path_str = path.path.segments.iter()
                .map(|s| s.ident.to_string())
                .collect::<Vec<_>>()
                .join("::");

            if path_str.contains("from_raw_parts") {
                use syn::spanned::Spanned;
                let start = node.span().start();
                let is_mut = path_str.contains("from_raw_parts_mut");
                self.violations.push(
                    Violation::new(
                        self.checker.code(),
                        self.checker.name(),
                        self.checker.severity().into(),
                        format!(
                            "Using slice::from_raw_parts{}(). This requires: (1) valid non-null pointer, (2) proper alignment, (3) valid memory for entire slice length.",
                            if is_mut { "_mut" } else { "" }
                        ),
                        self.file_path,
                        start.line,
                        start.column + 1,
                    )
                    .with_suggestion(self.checker.suggestions()),
                );
            }

            // Also check for str::from_utf8_unchecked
            if path_str.contains("from_utf8_unchecked") {
                use syn::spanned::Spanned;
                let start = node.span().start();
                self.violations.push(
                    Violation::new(
                        self.checker.code(),
                        self.checker.name(),
                        self.checker.severity().into(),
                        "Using str::from_utf8_unchecked(). Ensure the bytes are valid UTF-8 or use from_utf8() instead.",
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
    fn test_detects_from_raw_parts() {
        let code = r#"
            fn dangerous(ptr: *const i32, len: usize) -> &'static [i32] {
                unsafe {
                    std::slice::from_raw_parts(ptr, len)
                }
            }
        "#;

        let syntax = syn::parse_file(code).unwrap();
        let checker = E1609InvalidSlice::default();

        let mut violations = Vec::new();
        for item in &syntax.items {
            violations.extend(checker.check_item(item, "test.rs").unwrap());
        }

        assert_eq!(violations.len(), 1);
        assert!(violations[0].message.contains("from_raw_parts"));
    }

    #[test]
    fn test_detects_from_raw_parts_mut() {
        let code = r#"
            fn dangerous_mut(ptr: *mut i32, len: usize) -> &'static mut [i32] {
                unsafe {
                    std::slice::from_raw_parts_mut(ptr, len)
                }
            }
        "#;

        let syntax = syn::parse_file(code).unwrap();
        let checker = E1609InvalidSlice::default();

        let mut violations = Vec::new();
        for item in &syntax.items {
            violations.extend(checker.check_item(item, "test.rs").unwrap());
        }

        assert_eq!(violations.len(), 1);
        assert!(violations[0].message.contains("from_raw_parts_mut"));
    }

    #[test]
    fn test_detects_from_utf8_unchecked() {
        let code = r#"
            fn dangerous_str(bytes: &[u8]) -> &str {
                unsafe {
                    std::str::from_utf8_unchecked(bytes)
                }
            }
        "#;

        let syntax = syn::parse_file(code).unwrap();
        let checker = E1609InvalidSlice::default();

        let mut violations = Vec::new();
        for item in &syntax.items {
            violations.extend(checker.check_item(item, "test.rs").unwrap());
        }

        assert_eq!(violations.len(), 1);
        assert!(violations[0].message.contains("from_utf8_unchecked"));
    }

    #[test]
    fn test_safe_slice_passes() {
        let code = r#"
            fn safe(arr: &[i32]) -> &[i32] {
                &arr[1..3]
            }
        "#;

        let syntax = syn::parse_file(code).unwrap();
        let checker = E1609InvalidSlice::default();

        let mut violations = Vec::new();
        for item in &syntax.items {
            violations.extend(checker.check_item(item, "test.rs").unwrap());
        }

        assert_eq!(violations.len(), 0);
    }
}
