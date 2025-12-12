//! E1604: Buffer overflow patterns
//!
//! Detects patterns that can lead to buffer overflows:
//! - Unchecked slice indexing in unsafe blocks
//! - Manual buffer manipulation without bounds checking
//! - get_unchecked usage
//! - set_len without ensuring capacity
//!
//! Note: Safe Rust prevents buffer overflows, but unsafe code can bypass these protections.

use crate::{checker::Checker, define_checker, violation::Violation};

use syn::visit::Visit;

define_checker! {
    /// Checker for E1604: Buffer overflow detection
    E1604BufferOverflow,
    code = "E1604",
    name = "Buffer overflow pattern",
    suggestions = "Use bounds-checked methods like .get() or .get_mut(). Validate indices before unchecked access.",
    target_items = [Function],
    config_entry_name = "e1604_buffer_overflow",
    config = E1604Config {
        enabled: bool = true,
        severity: crate::config::SeverityLevel = crate::config::SeverityLevel::High,
        categories: Vec<crate::config::CheckerCategory> = vec![crate::config::CheckerCategory::Operations],
    },
    check_item(self, item, file_path) {
        let mut visitor = BufferOverflowVisitor {
            violations: Vec::new(),
            file_path,
            checker: self,
        };
        visitor.visit_item(item);
        Ok(visitor.violations)
    }
}

struct BufferOverflowVisitor<'a> {
    violations: Vec<Violation>,
    file_path: &'a str,
    checker: &'a E1604BufferOverflow,
}

impl<'a> Visit<'a> for BufferOverflowVisitor<'a> {
    fn visit_expr_method_call(&mut self, node: &'a syn::ExprMethodCall) {
        let method_name = node.method.to_string();

        // Detect dangerous unchecked methods
        match method_name.as_str() {
            "get_unchecked" | "get_unchecked_mut" => {
                let start = node.method.span().start();
                self.violations.push(
                    Violation::new(
                        self.checker.code(),
                        self.checker.name(),
                        self.checker.severity().into(),
                        format!(
                            "Using {}() bypasses bounds checking and can cause buffer overflow. Ensure index is valid.",
                            method_name
                        ),
                        self.file_path,
                        start.line,
                        start.column + 1,
                    )
                    .with_suggestion(self.checker.suggestions()),
                );
            }
            "set_len" => {
                let start = node.method.span().start();
                self.violations.push(
                    Violation::new(
                        self.checker.code(),
                        self.checker.name(),
                        self.checker.severity().into(),
                        "Using set_len() without ensuring capacity can cause undefined behavior. Verify capacity >= new length.",
                        self.file_path,
                        start.line,
                        start.column + 1,
                    )
                    .with_suggestion(self.checker.suggestions()),
                );
            }
            "copy_nonoverlapping" | "copy" => {
                let start = node.method.span().start();
                self.violations.push(
                    Violation::new(
                        self.checker.code(),
                        self.checker.name(),
                        self.checker.severity().into(),
                        format!(
                            "Using ptr::{}() requires careful bounds checking. Ensure source and destination have sufficient capacity.",
                            method_name
                        ),
                        self.file_path,
                        start.line,
                        start.column + 1,
                    )
                    .with_suggestion(self.checker.suggestions()),
                );
            }
            _ => {}
        }

        syn::visit::visit_expr_method_call(self, node);
    }

    fn visit_expr_call(&mut self, node: &'a syn::ExprCall) {
        // Check for ptr::copy and similar
        if let syn::Expr::Path(path) = &*node.func {
            let path_str = path.path.segments.iter()
                .map(|s| s.ident.to_string())
                .collect::<Vec<_>>()
                .join("::");

            if path_str.contains("copy_nonoverlapping") ||
               path_str.contains("write_bytes") ||
               path_str.ends_with("::copy") {
                use syn::spanned::Spanned;
                let start = node.span().start();
                self.violations.push(
                    Violation::new(
                        self.checker.code(),
                        self.checker.name(),
                        self.checker.severity().into(),
                        "Raw memory copy operation. Ensure bounds are checked to prevent buffer overflow.",
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
    fn test_detects_get_unchecked() {
        let code = r#"
            fn dangerous(arr: &[i32], idx: usize) -> i32 {
                unsafe { *arr.get_unchecked(idx) }
            }
        "#;

        let syntax = syn::parse_file(code).unwrap();
        let checker = E1604BufferOverflow::default();

        let mut violations = Vec::new();
        for item in &syntax.items {
            violations.extend(checker.check_item(item, "test.rs").unwrap());
        }

        assert_eq!(violations.len(), 1);
        assert!(violations[0].message.contains("get_unchecked"));
    }

    #[test]
    fn test_detects_set_len() {
        let code = r#"
            fn dangerous(mut v: Vec<i32>) {
                unsafe { v.set_len(1000) }
            }
        "#;

        let syntax = syn::parse_file(code).unwrap();
        let checker = E1604BufferOverflow::default();

        let mut violations = Vec::new();
        for item in &syntax.items {
            violations.extend(checker.check_item(item, "test.rs").unwrap());
        }

        assert_eq!(violations.len(), 1);
        assert!(violations[0].message.contains("set_len"));
    }

    #[test]
    fn test_safe_indexing_passes() {
        let code = r#"
            fn safe(arr: &[i32], idx: usize) -> Option<&i32> {
                arr.get(idx)
            }
        "#;

        let syntax = syn::parse_file(code).unwrap();
        let checker = E1604BufferOverflow::default();

        let mut violations = Vec::new();
        for item in &syntax.items {
            violations.extend(checker.check_item(item, "test.rs").unwrap());
        }

        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_detects_copy_nonoverlapping() {
        let code = r#"
            fn copy_data(src: *const u8, dst: *mut u8, len: usize) {
                unsafe {
                    std::ptr::copy_nonoverlapping(src, dst, len);
                }
            }
        "#;

        let syntax = syn::parse_file(code).unwrap();
        let checker = E1604BufferOverflow::default();

        let mut violations = Vec::new();
        for item in &syntax.items {
            violations.extend(checker.check_item(item, "test.rs").unwrap());
        }

        assert_eq!(violations.len(), 1);
    }
}
