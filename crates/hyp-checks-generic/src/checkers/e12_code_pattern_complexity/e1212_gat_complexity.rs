//! E1212: Generic Associated Types (GATs) complexity
//!
//! Detects GAT patterns which can be complex and hard to understand.
//!
//! Example:
//! ```text
//! trait Container {
//!     // GAT with generic parameter
//!     type Item<'a> where Self: 'a;
//! }
//! ```

use crate::{checker::Checker, define_checker, violation::Violation};
use syn::visit::Visit;

define_checker! {
    /// Checker for E1212: Generic Associated Types (GATs) complexity
    E1212GatComplexity,
    code = "E1212",
    name = "Generic Associated Types (GATs) complexity",
    suggestions = "GATs are powerful but complex. Ensure documentation explains the purpose. Consider if a simpler design would work.",
    target_items = [Trait, Impl],
    config_entry_name = "e1212_gat_complexity",
    config = E1212Config {
        enabled: bool = true,
        severity: crate::config::SeverityLevel = crate::config::SeverityLevel::Low,
        categories: Vec<crate::config::CheckerCategory> = vec![crate::config::CheckerCategory::Complexity],
        /// Warn on GAT declarations
        warn_on_gat: bool = true,
        /// Higher severity if GAT has where clause
        strict_on_bounded_gat: bool = true,
    },
    check_item(self, item, file_path) {
        let mut visitor = GatVisitor {
            violations: Vec::new(),
            file_path,
            checker: self,
        };
        visitor.visit_item(item);
        Ok(visitor.violations)
    }
}

struct GatVisitor<'a> {
    violations: Vec<Violation>,
    file_path: &'a str,
    checker: &'a E1212GatComplexity,
}

impl<'a> GatVisitor<'a> {
    fn check_trait_item_type(&mut self, item: &syn::TraitItemType) {
        // Check if this is a GAT (has generics)
        let has_generics = !item.generics.params.is_empty();
        let has_where_clause = item.generics.where_clause.is_some();

        if has_generics && self.checker.config.warn_on_gat {
            let severity: crate::violation::CheckerSeverity = if has_where_clause && self.checker.config.strict_on_bounded_gat {
                crate::violation::CheckerSeverity::Medium
            } else {
                self.checker.severity()
            };

            let message = if has_where_clause {
                format!(
                    "GAT '{}' with where clause detected. Bounded GATs add significant complexity.",
                    item.ident
                )
            } else {
                format!(
                    "Generic Associated Type '{}' detected. GATs increase API complexity.",
                    item.ident
                )
            };

            let start = item.ident.span().start();
            self.violations.push(
                Violation::new(
                    self.checker.code(),
                    self.checker.name(),
                    severity.into(),
                    &message,
                    self.file_path,
                    start.line,
                    start.column + 1,
                )
                .with_suggestion(self.checker.suggestions()),
            );
        }
    }

    fn check_impl_item_type(&mut self, item: &syn::ImplItemType) {
        // Check if this is a GAT implementation
        let has_generics = !item.generics.params.is_empty();
        let has_where_clause = item.generics.where_clause.is_some();

        if has_generics && self.checker.config.warn_on_gat {
            let severity: crate::violation::CheckerSeverity = if has_where_clause && self.checker.config.strict_on_bounded_gat {
                crate::violation::CheckerSeverity::Medium
            } else {
                self.checker.severity()
            };

            let start = item.ident.span().start();
            self.violations.push(
                Violation::new(
                    self.checker.code(),
                    self.checker.name(),
                    severity.into(),
                    format!("GAT implementation '{}' detected.", item.ident),
                    self.file_path,
                    start.line,
                    start.column + 1,
                )
                .with_suggestion(self.checker.suggestions()),
            );
        }
    }
}

impl<'a> Visit<'a> for GatVisitor<'a> {
    fn visit_trait_item_type(&mut self, node: &'a syn::TraitItemType) {
        self.check_trait_item_type(node);
        syn::visit::visit_trait_item_type(self, node);
    }

    fn visit_impl_item_type(&mut self, node: &'a syn::ImplItemType) {
        self.check_impl_item_type(node);
        syn::visit::visit_impl_item_type(self, node);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::checker::Checker;

    #[test]
    fn test_detects_simple_gat() {
        let code = r#"
            trait Container {
                type Item<'a> where Self: 'a;
            }
        "#;

        let syntax = syn::parse_file(code).unwrap();
        let checker = E1212GatComplexity::default();

        let mut violations = Vec::new();
        for item in &syntax.items {
            violations.extend(checker.check_item(item, "test.rs").unwrap());
        }

        assert_eq!(violations.len(), 1);
        assert!(violations[0].message.contains("GAT"));
    }

    #[test]
    fn test_detects_gat_with_type_param() {
        let code = r#"
            trait Lending {
                type Item<T>;
            }
        "#;

        let syntax = syn::parse_file(code).unwrap();
        let checker = E1212GatComplexity::default();

        let mut violations = Vec::new();
        for item in &syntax.items {
            violations.extend(checker.check_item(item, "test.rs").unwrap());
        }

        assert_eq!(violations.len(), 1);
    }

    #[test]
    fn test_regular_associated_type_passes() {
        let code = r#"
            trait Simple {
                type Item;
            }
        "#;

        let syntax = syn::parse_file(code).unwrap();
        let checker = E1212GatComplexity::default();

        let mut violations = Vec::new();
        for item in &syntax.items {
            violations.extend(checker.check_item(item, "test.rs").unwrap());
        }

        assert!(violations.is_empty());
    }

    #[test]
    fn test_detects_bounded_gat_higher_severity() {
        let code = r#"
            trait Complex {
                type Item<'a, T> where T: Clone, Self: 'a;
            }
        "#;

        let syntax = syn::parse_file(code).unwrap();
        let checker = E1212GatComplexity::default();

        let mut violations = Vec::new();
        for item in &syntax.items {
            violations.extend(checker.check_item(item, "test.rs").unwrap());
        }

        assert_eq!(violations.len(), 1);
        assert!(violations[0].message.contains("where clause"));
    }
}
