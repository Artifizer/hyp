//! E1017: Prohibit std::mem::transmute unconditionally
//!
//! This checker takes a zero-tolerance approach by completely prohibiting ALL uses
//! of std::mem::transmute, regardless of documentation or validation.
//!
//! Unlike E1006 which checks for missing size/alignment validation, E1017 bans
//! transmute entirely as there are almost always safer alternatives.

use crate::{checker::Checker, define_checker, violation::Violation};
use syn::{spanned::Spanned, visit::Visit};

define_checker! {
    /// Checker for E1017: Prohibit std::mem::transmute unconditionally
    E1017ProhibitTransmute,
    code = "E1017",
    name = "Prohibit std::mem::transmute",
    suggestions = "Use safe alternatives: f32::from_bits(), to_bits(), TryFrom::try_from(), or proper conversions",
    target_items = [Function, Impl],
    config_entry_name = "e1017_prohibit_transmute",
    /// Configuration for E1017: Prohibit transmute checker
    config = E1017Config {
        /// Whether this checker is enabled
        enabled: bool = true,
        /// Severity level (default: High)
        severity: crate::config::SeverityLevel = crate::config::SeverityLevel::High,
        /// Categories this checker belongs to
        categories: Vec<crate::config::CheckerCategory> = vec![
            crate::config::CheckerCategory::Operations,
        ],
    },
    check_item(self, item, file_path) {
        let mut visitor = TransmuteProhibitVisitor {
            violations: Vec::new(),
            file_path,
            checker: self,
        };
        visitor.visit_item(item);
        Ok(visitor.violations)
    }
}

struct TransmuteProhibitVisitor<'a> {
    violations: Vec<Violation>,
    file_path: &'a str,
    checker: &'a E1017ProhibitTransmute,
}

impl<'a> TransmuteProhibitVisitor<'a> {
    fn is_transmute_call(&self, path: &syn::Path) -> bool {
        // Check for std::mem::transmute, core::mem::transmute, or just transmute
        if path.segments.len() >= 3 {
            let segments: Vec<String> = path.segments.iter().map(|s| s.ident.to_string()).collect();

            // Check for std::mem::transmute or core::mem::transmute
            if segments.len() >= 3 {
                let n = segments.len();
                if (segments[n - 3] == "std" || segments[n - 3] == "core")
                    && segments[n - 2] == "mem"
                    && segments[n - 1] == "transmute"
                {
                    return true;
                }
            }
        }

        // Check for just "transmute" (imported)
        if path.segments.len() == 1 && path.segments[0].ident == "transmute" {
            return true;
        }

        // Check for mem::transmute
        if path.segments.len() == 2
            && path.segments[0].ident == "mem"
            && path.segments[1].ident == "transmute"
        {
            return true;
        }

        false
    }
}

impl<'a> Visit<'a> for TransmuteProhibitVisitor<'a> {
    fn visit_expr_call(&mut self, node: &'a syn::ExprCall) {
        // Check if this is a transmute call
        if let syn::Expr::Path(expr_path) = &*node.func {
            if self.is_transmute_call(&expr_path.path) {
                let span = expr_path.span();
                self.violations.push(
                    Violation::new(
                        self.checker.code(),
                        self.checker.name(),
                        self.checker.severity().into(),
                        "std::mem::transmute is prohibited - use safe alternatives",
                        self.file_path,
                        span.start().line,
                        span.start().column + 1,
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

    fn check_code(code: &str) -> Vec<Violation> {
        let checker = E1017ProhibitTransmute::default();
        let file = syn::parse_file(code).expect("Failed to parse");
        let mut violations = Vec::new();
        for item in &file.items {
            violations.extend(checker.check_item(item, "test.rs").unwrap());
        }
        violations
    }

    #[test]
    fn test_detects_std_mem_transmute() {
        let code = r#"
            fn example() {
                let x: u32 = 42;
                let y: f32 = unsafe { std::mem::transmute(x) };
            }
        "#;
        let violations = check_code(code);
        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].code, "E1017");
    }

    #[test]
    fn test_detects_core_mem_transmute() {
        let code = r#"
            fn example() {
                let x: u32 = 42;
                let y: f32 = unsafe { core::mem::transmute(x) };
            }
        "#;
        let violations = check_code(code);
        assert_eq!(violations.len(), 1);
    }

    #[test]
    fn test_detects_imported_transmute() {
        let code = r#"
            use std::mem::transmute;

            fn example() {
                let x: u32 = 42;
                let y: f32 = unsafe { transmute(x) };
            }
        "#;
        let violations = check_code(code);
        assert_eq!(violations.len(), 1);
    }

    #[test]
    fn test_allows_from_bits() {
        let code = r#"
            fn example() {
                let x: u32 = 42;
                let y: f32 = f32::from_bits(x);
            }
        "#;
        let violations = check_code(code);
        assert!(violations.is_empty());
    }

    #[test]
    fn test_allows_to_bits() {
        let code = r#"
            fn example() {
                let x: f32 = 42.0;
                let y: u32 = x.to_bits();
            }
        "#;
        let violations = check_code(code);
        assert!(violations.is_empty());
    }

    #[test]
    fn test_allows_try_from() {
        let code = r#"
            fn example() -> Result<i32, std::num::TryFromIntError> {
                let x: u64 = 42;
                i32::try_from(x)
            }
        "#;
        let violations = check_code(code);
        assert!(violations.is_empty());
    }
}
