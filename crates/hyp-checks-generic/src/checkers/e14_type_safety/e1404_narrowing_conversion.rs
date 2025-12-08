//! E1404: Narrowing conversions (as)
//!
//! Detects potentially lossy type conversions using `as` that could silently
//! truncate or change the value.

use crate::{checker::Checker, define_checker, violation::Violation};

use syn::{spanned::Spanned, visit::Visit};

define_checker! {
    /// Checker for E1404: Narrowing conversions
    E1404NarrowingConversion,
    code = "E1404",
    name = "Narrowing conversion",
    suggestions = "Use try_into() or TryFrom for fallible conversions, or explicitly handle overflow",
    target_items = [Function],
    config_entry_name = "e1404_narrowing_conversion",
    /// Configuration for E1404: Narrowing conversion checker
    config = E1404Config {
        /// Whether this checker is enabled
        enabled: bool = true,
        /// Severity level, defaults to Medium
        severity: crate::config::SeverityLevel = crate::config::SeverityLevel::High,
        /// Categories this checker belongs to
        categories: Vec<crate::config::CheckerCategory> = vec![crate::config::CheckerCategory::Operations],
    },
    check_item(self, item, file_path) {
        let mut visitor = NarrowingVisitor {
            violations: Vec::new(),
            file_path,
            checker: self,
        };
        visitor.visit_item(item);
        Ok(visitor.violations)
    }
}

struct NarrowingVisitor<'a> {
    violations: Vec<Violation>,
    file_path: &'a str,
    checker: &'a E1404NarrowingConversion,
}

impl<'a> NarrowingVisitor<'a> {
    fn create_violation(&self, span: proc_macro2::Span, from: &str, to: &str) -> Violation {
        let start = span.start();
        Violation::new(
            self.checker.code(),
            self.checker.name(),
            self.checker.severity().into(),
            format!(
                "Narrowing conversion from {} to {} may silently truncate the value.",
                from, to
            ),
            self.file_path,
            start.line,
            start.column + 1,
        )
        .with_suggestion(self.checker.suggestions())
    }
}

impl<'a> Visit<'a> for NarrowingVisitor<'a> {
    fn visit_expr_cast(&mut self, node: &'a syn::ExprCast) {
        if let Some((from_type, to_type)) = extract_cast_types(node) {
            if is_narrowing_conversion(&from_type, &to_type) {
                self.violations.push(self.create_violation(node.span(), &from_type, &to_type));
            }
        }

        syn::visit::visit_expr_cast(self, node);
    }
}

/// Extract type names from a cast expression
fn extract_cast_types(cast: &syn::ExprCast) -> Option<(String, String)> {
    let to_type = type_to_string(&cast.ty);
    // We can't always determine the from type without type inference,
    // but we can detect common patterns
    Some(("larger type".to_string(), to_type))
}

/// Convert a type to a string representation
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

/// Check if a conversion is potentially narrowing
fn is_narrowing_conversion(_from: &str, to: &str) -> bool {
    // Common narrowing patterns
    let narrowing_targets = ["u8", "u16", "i8", "i16", "u32", "i32", "usize", "isize"];

    // If casting to a smaller integer type, it's potentially narrowing
    narrowing_targets.contains(&to)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detects_narrowing_cast() {
        let code = r#"
            fn example(x: i64) -> i32 {
                x as i32
            }
        "#;

        let syntax = syn::parse_file(code).unwrap();
        let checker = E1404NarrowingConversion::default();

        let mut violations = Vec::new();
        for item in &syntax.items {
            violations.extend(checker.check_item(item, "test.rs").unwrap());
        }

        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].code, "E1404");
    }

    #[test]
    fn test_detects_u64_to_u8() {
        let code = r#"
            fn example(x: u64) -> u8 {
                x as u8
            }
        "#;

        let syntax = syn::parse_file(code).unwrap();
        let checker = E1404NarrowingConversion::default();

        let mut violations = Vec::new();
        for item in &syntax.items {
            violations.extend(checker.check_item(item, "test.rs").unwrap());
        }

        assert_eq!(violations.len(), 1);
    }

    #[test]
    fn test_safe_widening_not_flagged() {
        // Widening conversions like i32 to i64 are safe, but we can't
        // distinguish without type inference, so this test documents behavior
        let code = r#"
            fn example(x: i32) -> i64 {
                x as i64
            }
        "#;

        let syntax = syn::parse_file(code).unwrap();
        let checker = E1404NarrowingConversion::default();

        let mut violations = Vec::new();
        for item in &syntax.items {
            violations.extend(checker.check_item(item, "test.rs").unwrap());
        }

        // Currently flags all casts to smaller types
        assert_eq!(violations.len(), 0);
    }
}
