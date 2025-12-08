//! E1812: Public enum without `#[non_exhaustive]`
//!
//! Detects public enums that might grow in the future but lack the
//! `#[non_exhaustive]` attribute. Without it, adding variants is a breaking change.
//!
//! Example:
//! ```text
//! // Bad: Adding a variant breaks downstream matches
//! pub enum Status { Active, Inactive }
//!
//! // Good: Allows adding variants without breaking changes
//! #[non_exhaustive]
//! pub enum Status { Active, Inactive }
//! ```

use crate::{define_checker, violation::Violation};

define_checker! {
    /// Checker for E1812: Public enum without non_exhaustive
    E1812NonExhaustiveEnum,
    code = "E1812",
    name = "Public enum without #[non_exhaustive]",
    suggestions = "Add #[non_exhaustive] to public enums that might gain variants, or document that the enum is closed",
    target_items = [Enum],
    config_entry_name = "e1812_non_exhaustive_enum",
    config = E1812Config {
        enabled: bool = true,
        severity: crate::config::SeverityLevel = crate::config::SeverityLevel::Medium,
        categories: Vec<crate::config::CheckerCategory> = vec![crate::config::CheckerCategory::Complexity],
        /// Minimum number of variants to trigger (small enums might be intentionally closed)
        min_variants: usize = 2,
    },
    check_item(self, item, file_path) {
        let mut violations = Vec::new();

        if let syn::Item::Enum(enum_item) = item {
            // Only check public enums
            if !matches!(enum_item.vis, syn::Visibility::Public(_)) {
                return Ok(violations);
            }

            // Check if it already has non_exhaustive
            let has_non_exhaustive = enum_item.attrs.iter().any(|attr| {
                attr.path()
                    .segments
                    .last()
                    .is_some_and(|seg| seg.ident == "non_exhaustive")
            });

            if has_non_exhaustive {
                return Ok(violations);
            }

            // Check variant count
            if enum_item.variants.len() < self.config.min_variants {
                return Ok(violations);
            }

            // Check for common "closed" enum patterns (like Result-like or Option-like)
            let variant_names: Vec<_> = enum_item.variants.iter()
                .map(|v| v.ident.to_string())
                .collect();

            // Skip enums that look intentionally closed
            let closed_patterns = [
                vec!["Ok", "Err"],
                vec!["Some", "None"],
                vec!["True", "False"],
                vec!["Yes", "No"],
                vec!["On", "Off"],
            ];

            let is_closed_pattern = closed_patterns.iter().any(|pattern| {
                pattern.len() == variant_names.len()
                    && pattern.iter().all(|v| variant_names.iter().any(|name| name == *v))
            });

            if is_closed_pattern {
                return Ok(violations);
            }

            let span = enum_item.ident.span();
            violations.push(
                Violation::new(
                    self.code(),
                    self.name(),
                    self.severity().into(),
                    format!(
                        "Public enum '{}' has {} variants but no #[non_exhaustive]. Adding variants will break downstream code.",
                        enum_item.ident,
                        enum_item.variants.len()
                    ),
                    file_path,
                    span.start().line,
                    span.start().column + 1,
                )
                .with_suggestion(self.suggestions()),
            );
        }

        Ok(violations)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::checker::Checker;

    fn check_code(code: &str) -> Vec<Violation> {
        let checker = E1812NonExhaustiveEnum::default();
        let file = syn::parse_file(code).expect("Failed to parse");
        let mut violations = Vec::new();
        for item in &file.items {
            violations.extend(checker.check_item(item, "test.rs").unwrap());
        }
        violations
    }

    #[test]
    fn test_detects_public_enum_without_non_exhaustive() {
        let code = r#"
            pub enum Status {
                Active,
                Inactive,
                Pending,
            }
        "#;
        let violations = check_code(code);
        assert_eq!(violations.len(), 1);
    }

    #[test]
    fn test_non_exhaustive_enum_passes() {
        let code = r#"
            #[non_exhaustive]
            pub enum Status {
                Active,
                Inactive,
                Pending,
            }
        "#;
        let violations = check_code(code);
        assert!(violations.is_empty());
    }

    #[test]
    fn test_private_enum_passes() {
        let code = r#"
            enum Status {
                Active,
                Inactive,
                Pending,
            }
        "#;
        let violations = check_code(code);
        assert!(violations.is_empty());
    }

    #[test]
    fn test_result_like_enum_passes() {
        let code = r#"
            pub enum MyResult {
                Ok,
                Err,
            }
        "#;
        let violations = check_code(code);
        assert!(violations.is_empty());
    }

    #[test]
    fn test_small_enum_passes() {
        let code = r#"
            pub enum Toggle {
                On,
            }
        "#;
        let violations = check_code(code);
        assert!(violations.is_empty());
    }
}
