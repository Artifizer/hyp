//! E1014: Raw pointer arithmetic without bounds checking
//!
//! Detects raw pointer arithmetic operations (offset, add, sub) which can easily
//! lead to out-of-bounds access and undefined behavior.

use crate::{checker::Checker, define_checker, violation::Violation};

use syn::visit::Visit;

define_checker! {
    /// Checker for E1014: Raw pointer arithmetic without bounds checking
    E1014RawPointerArithmetic,
    code = "E1014",
    name = "Raw pointer arithmetic without bounds checking",
    suggestions = "Use slice iterators or checked pointer operations. Validate bounds before pointer arithmetic.",
    target_items = [Function],
    config_entry_name = "e1014_raw_pointer_arithmetic",
    /// Configuration for E1014: Raw pointer arithmetic checker
    config = E1014Config {
        /// Whether this checker is enabled
        enabled: bool = true,
        /// Severity level, defaults to High
        severity: crate::config::SeverityLevel = crate::config::SeverityLevel::High,
        /// Categories this checker belongs to
        categories: Vec<crate::config::CheckerCategory> = vec![crate::config::CheckerCategory::Operations],
    },
    check_item(self, item, file_path) {
        let mut visitor = PointerArithmeticVisitor {
            violations: Vec::new(),
            file_path,
            checker: self,
        };
        visitor.visit_item(item);
        Ok(visitor.violations)
    }
}

struct PointerArithmeticVisitor<'a> {
    violations: Vec<Violation>,
    file_path: &'a str,
    checker: &'a E1014RawPointerArithmetic,
}

impl<'a> PointerArithmeticVisitor<'a> {
    fn create_violation(&self, span: proc_macro2::Span, method: &str) -> Violation {
        let start = span.start();
        Violation::new(
            self.checker.code(),
            self.checker.name(),
            self.checker.severity().into(),
            format!(
                "Raw pointer arithmetic using {}(). This can lead to out-of-bounds access if not properly validated.",
                method
            ),
            self.file_path,
            start.line,
            start.column + 1,
        )
        .with_suggestion(self.checker.suggestions())
    }
}

impl<'a> Visit<'a> for PointerArithmeticVisitor<'a> {
    fn visit_expr_method_call(&mut self, node: &'a syn::ExprMethodCall) {
        let method_name = node.method.to_string();

        // Check for pointer arithmetic methods
        if matches!(method_name.as_str(), "offset" | "add" | "sub" | "wrapping_offset" | "wrapping_add" | "wrapping_sub") {
            self.violations.push(self.create_violation(node.method.span(), &method_name));
        }

        syn::visit::visit_expr_method_call(self, node);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detects_pointer_offset() {
        let code = r#"
            fn example() {
                let arr = [1, 2, 3, 4, 5];
                let ptr = arr.as_ptr();
                unsafe {
                    let _p = ptr.offset(2);
                }
            }
        "#;

        let syntax = syn::parse_file(code).unwrap();
        let checker = E1014RawPointerArithmetic::default();

        let mut violations = Vec::new();
        for item in &syntax.items {
            violations.extend(checker.check_item(item, "test.rs").unwrap());
        }

        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].code, "E1014");
        assert!(violations[0].message.contains("offset"));
    }

    #[test]
    fn test_detects_pointer_add() {
        let code = r#"
            fn example() {
                let arr = [1, 2, 3];
                let ptr = arr.as_ptr();
                unsafe {
                    let _p = ptr.add(1);
                }
            }
        "#;

        let syntax = syn::parse_file(code).unwrap();
        let checker = E1014RawPointerArithmetic::default();

        let mut violations = Vec::new();
        for item in &syntax.items {
            violations.extend(checker.check_item(item, "test.rs").unwrap());
        }

        assert_eq!(violations.len(), 1);
        assert!(violations[0].message.contains("add"));
    }

    #[test]
    fn test_detects_pointer_sub() {
        let code = r#"
            fn example() {
                let arr = [1, 2, 3];
                let ptr = arr.as_ptr();
                unsafe {
                    let _p = ptr.sub(1);
                }
            }
        "#;

        let syntax = syn::parse_file(code).unwrap();
        let checker = E1014RawPointerArithmetic::default();

        let mut violations = Vec::new();
        for item in &syntax.items {
            violations.extend(checker.check_item(item, "test.rs").unwrap());
        }

        assert_eq!(violations.len(), 1);
    }

    #[test]
    fn test_safe_slice_iteration_passes() {
        let code = r#"
            fn example() {
                let arr = [1, 2, 3, 4, 5];
                for item in arr.iter() {
                    println!("{}", item);
                }
            }
        "#;

        let syntax = syn::parse_file(code).unwrap();
        let checker = E1014RawPointerArithmetic::default();

        let mut violations = Vec::new();
        for item in &syntax.items {
            violations.extend(checker.check_item(item, "test.rs").unwrap());
        }

        assert_eq!(violations.len(), 0);
    }
}
