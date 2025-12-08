//! E1406: Signed/unsigned mismatch
//!
//! Detects conversions between signed and unsigned integers that could
//! produce unexpected results.

use crate::{checker::Checker, define_checker, violation::Violation};

use syn::{spanned::Spanned, visit::Visit};

define_checker! {
    /// Checker for E1406: Signed/unsigned mismatch
    E1406SignedUnsignedMismatch,
    code = "E1406",
    name = "Signed/unsigned mismatch",
    suggestions = "Use try_into() for fallible conversion, or explicitly handle the sign",
    target_items = [Function],
    config_entry_name = "e1406_signed_unsigned_mismatch",
    /// Configuration for E1406: Signed/unsigned mismatch checker
    config = E1406Config {
        /// Whether this checker is enabled
        enabled: bool = true,
        /// Severity level, defaults to Medium
        severity: crate::config::SeverityLevel = crate::config::SeverityLevel::High,
        /// Categories this checker belongs to
        categories: Vec<crate::config::CheckerCategory> = vec![crate::config::CheckerCategory::Operations],
    },
    check_item(self, item, file_path) {
        let mut visitor = SignMismatchVisitor {
            violations: Vec::new(),
            file_path,
            checker: self,
        };
        visitor.visit_item(item);
        Ok(visitor.violations)
    }
}

struct SignMismatchVisitor<'a> {
    violations: Vec<Violation>,
    file_path: &'a str,
    checker: &'a E1406SignedUnsignedMismatch,
}

impl<'a> SignMismatchVisitor<'a> {
    fn create_violation(&self, span: proc_macro2::Span, to_type: &str) -> Violation {
        let start = span.start();
        Violation::new(
            self.checker.code(),
            self.checker.name(),
            self.checker.severity().into(),
            format!(
                "Casting to {} may change the sign interpretation of the value.",
                to_type
            ),
            self.file_path,
            start.line,
            start.column + 1,
        )
        .with_suggestion(self.checker.suggestions())
    }
}

impl<'a> Visit<'a> for SignMismatchVisitor<'a> {
    fn visit_expr_cast(&mut self, node: &'a syn::ExprCast) {
        let to_type = type_to_string(&node.ty);

        // Check if casting to a type with different signedness
        if is_sign_changing_target(&to_type) {
            self.violations.push(self.create_violation(node.span(), &to_type));
        }

        syn::visit::visit_expr_cast(self, node);
    }
}

fn type_to_string(ty: &syn::Type) -> String {
    match ty {
        syn::Type::Path(type_path) => {
            type_path.path.segments.iter()
                .map(|s| s.ident.to_string())
                .collect::<Vec<_>>()
                .join("::")
        }
        _ => "unknown".to_string()
    }
}

/// Check if the target type could involve a sign change
fn is_sign_changing_target(to_type: &str) -> bool {
    // Unsigned types that might receive signed values
    matches!(to_type, "u8" | "u16" | "u32" | "u64" | "u128" | "usize")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detects_signed_to_unsigned() {
        let code = r#"
            fn example(x: i32) -> u32 {
                x as u32  // -1i32 becomes 4294967295u32
            }
        "#;

        let syntax = syn::parse_file(code).unwrap();
        let checker = E1406SignedUnsignedMismatch::default();

        let mut violations = Vec::new();
        for item in &syntax.items {
            violations.extend(checker.check_item(item, "test.rs").unwrap());
        }

        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].code, "E1406");
    }

    #[test]
    fn test_detects_to_usize() {
        let code = r#"
            fn example(x: i64) -> usize {
                x as usize
            }
        "#;

        let syntax = syn::parse_file(code).unwrap();
        let checker = E1406SignedUnsignedMismatch::default();

        let mut violations = Vec::new();
        for item in &syntax.items {
            violations.extend(checker.check_item(item, "test.rs").unwrap());
        }

        assert_eq!(violations.len(), 1);
    }

    #[test]
    fn test_signed_to_signed_passes() {
        let code = r#"
            fn example(x: i32) -> i64 {
                x as i64
            }
        "#;

        let syntax = syn::parse_file(code).unwrap();
        let checker = E1406SignedUnsignedMismatch::default();

        let mut violations = Vec::new();
        for item in &syntax.items {
            violations.extend(checker.check_item(item, "test.rs").unwrap());
        }

        assert_eq!(violations.len(), 0);
    }
}
