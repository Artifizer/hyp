//! E1211: Trait object coercion complexity
//!
//! Detects complex trait object patterns that can be hard to understand:
//! - Multiple trait bounds on dyn objects
//! - Deeply nested trait objects
//! - Box<dyn ...> with complex bounds
//!
//! These patterns often indicate over-abstraction.

use crate::{checker::Checker, define_checker, violation::Violation};
use syn::visit::Visit;

define_checker! {
    /// Checker for E1211: Trait object coercion complexity
    E1211TraitObjectComplexity,
    code = "E1211",
    name = "Trait object coercion complexity",
    suggestions = "Consider using concrete types, generics, or enum dispatch instead of complex trait objects.",
    target_items = [Function, Struct, Type],
    config_entry_name = "e1211_trait_object_complexity",
    config = E1211Config {
        enabled: bool = true,
        severity: crate::config::SeverityLevel = crate::config::SeverityLevel::Medium,
        categories: Vec<crate::config::CheckerCategory> = vec![crate::config::CheckerCategory::Complexity],
        /// Maximum trait bounds on a single dyn object
        max_trait_bounds: usize = 2,
    },
    check_item(self, item, file_path) {
        let mut visitor = TraitObjectVisitor {
            violations: Vec::new(),
            file_path,
            checker: self,
        };
        visitor.visit_item(item);
        Ok(visitor.violations)
    }
}

struct TraitObjectVisitor<'a> {
    violations: Vec<Violation>,
    file_path: &'a str,
    checker: &'a E1211TraitObjectComplexity,
}

impl<'a> Visit<'a> for TraitObjectVisitor<'a> {
    fn visit_type_trait_object(&mut self, node: &'a syn::TypeTraitObject) {
        // Count trait bounds (excluding lifetime bounds)
        let trait_bound_count = node.bounds.iter().filter(|b| {
            matches!(b, syn::TypeParamBound::Trait(_))
        }).count();

        if trait_bound_count > self.checker.config.max_trait_bounds {
            use syn::spanned::Spanned;
            let start = node.span().start();
            self.violations.push(
                Violation::new(
                    self.checker.code(),
                    self.checker.name(),
                    self.checker.severity().into(),
                    format!(
                        "Trait object has {} bounds (max {}). Complex trait objects are hard to reason about.",
                        trait_bound_count, self.checker.config.max_trait_bounds
                    ),
                    self.file_path,
                    start.line,
                    start.column + 1,
                )
                .with_suggestion(self.checker.suggestions()),
            );
        }

        syn::visit::visit_type_trait_object(self, node);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::checker::Checker;

    #[test]
    fn test_detects_complex_trait_object() {
        let code = r#"
            trait A {}
            trait B {}
            trait C {}

            fn complex(x: Box<dyn A + B + C + Send + Sync>) {}
        "#;

        let syntax = syn::parse_file(code).unwrap();
        let checker = E1211TraitObjectComplexity::default();

        let mut violations = Vec::new();
        for item in &syntax.items {
            violations.extend(checker.check_item(item, "test.rs").unwrap());
        }

        assert_eq!(violations.len(), 1);
        assert!(violations[0].message.contains("bounds"));
    }

    #[test]
    fn test_simple_trait_object_passes() {
        let code = r#"
            trait A {}

            fn simple(x: Box<dyn A + Send>) {}
        "#;

        let syntax = syn::parse_file(code).unwrap();
        let checker = E1211TraitObjectComplexity::default();

        let mut violations = Vec::new();
        for item in &syntax.items {
            violations.extend(checker.check_item(item, "test.rs").unwrap());
        }

        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_detects_in_struct_field() {
        let code = r#"
            trait A {}
            trait B {}
            trait C {}

            struct Container {
                handler: Box<dyn A + B + C + Send>,
            }
        "#;

        let syntax = syn::parse_file(code).unwrap();
        let checker = E1211TraitObjectComplexity::default();

        let mut violations = Vec::new();
        for item in &syntax.items {
            violations.extend(checker.check_item(item, "test.rs").unwrap());
        }

        assert_eq!(violations.len(), 1);
    }
}
