//! E1205: Complex handler with nested trait bounds
//!
//! Detects where clauses with excessive nested trait bounds that make
//! function signatures hard to understand.
//!
//! Example:
//! ```text
//! // Bad: Too many bounds
//! fn complex<T, U, V, W, X, Y>(t: T)
//! where
//!     T: Trait1 + Trait2,
//!     U: Trait3 + Trait4,
//!     V: Trait5,
//!     W: Trait6,
//!     X: Trait7,
//!     Y: Trait8,
//! { ... }
//! ```

use crate::{checker::Checker, define_checker, violation::Violation};

define_checker! {
    /// Checker for E1205: Complex handler with nested trait bounds
    E1205NestedTraitBounds,
    code = "E1205",
    name = "Complex handler with nested trait bounds",
    suggestions = "Consider extracting bounds into a trait alias, or splitting the function into smaller pieces with simpler bounds.",
    target_items = [Function, Impl],
    config_entry_name = "e1205_nested_trait_bounds",
    config = E1205Config {
        enabled: bool = true,
        severity: crate::config::SeverityLevel = crate::config::SeverityLevel::Medium,
        categories: Vec<crate::config::CheckerCategory> = vec![crate::config::CheckerCategory::Complexity],
        /// Maximum number of where clause predicates
        max_where_predicates: usize = 5,
        /// Maximum total bounds across all type parameters
        max_total_bounds: usize = 8,
    },
    check_item(self, item, file_path) {
        let mut violations = Vec::new();

        match item {
            syn::Item::Fn(func) => {
                self.check_generics(&func.sig.generics, file_path, &mut violations);
            }
            syn::Item::Impl(impl_block) => {
                self.check_generics(&impl_block.generics, file_path, &mut violations);

                for item in &impl_block.items {
                    if let syn::ImplItem::Fn(method) = item {
                        self.check_generics(&method.sig.generics, file_path, &mut violations);
                    }
                }
            }
            syn::Item::Trait(trait_def) => {
                self.check_generics(&trait_def.generics, file_path, &mut violations);
            }
            _ => {}
        }

        Ok(violations)
    }
}

impl E1205NestedTraitBounds {
    fn check_generics(
        &self,
        generics: &syn::Generics,
        file_path: &str,
        violations: &mut Vec<Violation>,
    ) {
        use syn::spanned::Spanned;

        // Count total bounds in type parameters
        let mut total_bounds = 0;
        for type_param in generics.type_params() {
            total_bounds += type_param.bounds.len();
        }

        // Count where clause predicates and their bounds
        let mut where_predicate_count = 0;
        if let Some(where_clause) = &generics.where_clause {
            where_predicate_count = where_clause.predicates.len();

            for predicate in &where_clause.predicates {
                if let syn::WherePredicate::Type(type_pred) = predicate {
                    total_bounds += type_pred.bounds.len();
                }
            }
        }

        if where_predicate_count > self.config.max_where_predicates {
            if let Some(where_clause) = &generics.where_clause {
                let start = where_clause.span().start();
                violations.push(
                    Violation::new(
                        self.code(),
                        self.name(),
                        self.severity().into(),
                        format!(
                            "Where clause has {} predicates (max {}). Complex bounds reduce readability.",
                            where_predicate_count,
                            self.config.max_where_predicates
                        ),
                        file_path,
                        start.line,
                        start.column + 1,
                    )
                    .with_suggestion(self.suggestions()),
                );
            }
        }

        if total_bounds > self.config.max_total_bounds {
            let start = generics.span().start();
            violations.push(
                Violation::new(
                    self.code(),
                    self.name(),
                    self.severity().into(),
                    format!(
                        "Function has {} total trait bounds (max {}). Consider using trait aliases to simplify.",
                        total_bounds,
                        self.config.max_total_bounds
                    ),
                    file_path,
                    start.line,
                    start.column + 1,
                )
                .with_suggestion(self.suggestions()),
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::checker::Checker;

    #[test]
    fn test_detects_many_where_predicates() {
        let code = r#"
            fn many_bounds<T, U, V, W, X, Y>(t: T, u: U, v: V, w: W, x: X, y: Y)
            where
                T: Clone,
                U: Clone,
                V: Clone,
                W: Clone,
                X: Clone,
                Y: Clone,
            {
            }
        "#;

        let syntax = syn::parse_file(code).unwrap();
        let checker = E1205NestedTraitBounds::default();

        let mut violations = Vec::new();
        for item in &syntax.items {
            violations.extend(checker.check_item(item, "test.rs").unwrap());
        }

        assert!(!violations.is_empty());
        assert!(violations[0].message.contains("6 predicates"));
    }

    #[test]
    fn test_detects_many_total_bounds() {
        let code = r#"
            fn bound_heavy<T: Clone + Send + Sync + Default, U: Clone + Send + Sync + Default + Debug>(t: T, u: U) {
            }
        "#;

        let syntax = syn::parse_file(code).unwrap();
        let checker = E1205NestedTraitBounds::default();

        let mut violations = Vec::new();
        for item in &syntax.items {
            violations.extend(checker.check_item(item, "test.rs").unwrap());
        }

        assert!(!violations.is_empty());
        assert!(violations[0].message.contains("total trait bounds"));
    }

    #[test]
    fn test_simple_bounds_pass() {
        let code = r#"
            fn simple<T: Clone>(t: T) {
            }
        "#;

        let syntax = syn::parse_file(code).unwrap();
        let checker = E1205NestedTraitBounds::default();

        let mut violations = Vec::new();
        for item in &syntax.items {
            violations.extend(checker.check_item(item, "test.rs").unwrap());
        }

        assert!(violations.is_empty());
    }

    #[test]
    fn test_moderate_bounds_pass() {
        let code = r#"
            fn moderate<T, U>(t: T, u: U)
            where
                T: Clone + Send,
                U: Clone + Sync,
            {
            }
        "#;

        let syntax = syn::parse_file(code).unwrap();
        let checker = E1205NestedTraitBounds::default();

        let mut violations = Vec::new();
        for item in &syntax.items {
            violations.extend(checker.check_item(item, "test.rs").unwrap());
        }

        assert!(violations.is_empty());
    }
}
