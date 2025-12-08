//! E1202: Complex lifetime annotations
//!
//! Detects functions with overly complex lifetime annotations that make code
//! hard to understand and maintain.
//!
//! Triggers:
//! - 3+ distinct lifetime parameters
//! - Lifetime bounds with 2+ bounds (e.g., `'a: 'b + 'c`)
//!
//! Example:
//! ```text
//! // Bad: Too many lifetimes
//! fn complex<'a, 'b, 'c, 'd>(x: &'a str, y: &'b str, z: &'c str) -> &'d str
//! where
//!     'a: 'b + 'c,
//!     'b: 'd,
//! { ... }
//! ```

use crate::{checker::Checker, define_checker, violation::Violation};

define_checker! {
    /// Checker for E1202: Complex lifetime annotations
    E1202ComplexLifetimes,
    code = "E1202",
    name = "Complex lifetime annotations",
    suggestions = "Consider simplifying lifetime annotations. Use elision where possible, or restructure to reduce lifetime relationships.",
    target_items = [Function, Impl],
    config_entry_name = "e1202_complex_lifetimes",
    config = E1202Config {
        enabled: bool = true,
        severity: crate::config::SeverityLevel = crate::config::SeverityLevel::Medium,
        categories: Vec<crate::config::CheckerCategory> = vec![crate::config::CheckerCategory::Complexity],
        /// Maximum number of lifetime parameters before flagging
        max_lifetime_params: usize = 3,
        /// Maximum number of bounds per lifetime
        max_bounds_per_lifetime: usize = 2,
    },
    check_item(self, item, file_path) {
        let mut violations = Vec::new();

        match item {
            syn::Item::Fn(func) => {
                self.check_generics(&func.sig.generics, file_path, &mut violations);
            }
            syn::Item::Impl(impl_block) => {
                self.check_generics(&impl_block.generics, file_path, &mut violations);

                // Check methods within impl block
                for item in &impl_block.items {
                    if let syn::ImplItem::Fn(method) = item {
                        self.check_generics(&method.sig.generics, file_path, &mut violations);
                    }
                }
            }
            _ => {}
        }

        Ok(violations)
    }
}

impl E1202ComplexLifetimes {
    fn check_generics(
        &self,
        generics: &syn::Generics,
        file_path: &str,
        violations: &mut Vec<Violation>,
    ) {
        use syn::spanned::Spanned;

        // Count lifetime parameters
        let lifetime_count = generics.lifetimes().count();

        if lifetime_count >= self.config.max_lifetime_params {
            let start = generics.span().start();
            violations.push(
                Violation::new(
                    self.code(),
                    self.name(),
                    self.severity().into(),
                    format!(
                        "Found {} lifetime parameters (threshold {}). Complex lifetime annotations reduce readability.",
                        lifetime_count,
                        self.config.max_lifetime_params
                    ),
                    file_path,
                    start.line,
                    start.column + 1,
                )
                .with_suggestion(self.suggestions()),
            );
        }

        // Check lifetime bounds complexity
        for lifetime_def in generics.lifetimes() {
            let bounds_count = lifetime_def.bounds.len();

            if bounds_count >= self.config.max_bounds_per_lifetime {
                let start = lifetime_def.span().start();
                violations.push(
                    Violation::new(
                        self.code(),
                        self.name(),
                        self.severity().into(),
                        format!(
                            "Lifetime '{}' has {} bounds. Complex lifetime relationships are hard to reason about.",
                            lifetime_def.lifetime.ident,
                            bounds_count
                        ),
                        file_path,
                        start.line,
                        start.column + 1,
                    )
                    .with_suggestion(self.suggestions()),
                );
            }
        }

        // Check where clause for additional lifetime bounds
        if let Some(where_clause) = &generics.where_clause {
            for predicate in &where_clause.predicates {
                if let syn::WherePredicate::Lifetime(lifetime_pred) = predicate {
                    let bounds_count = lifetime_pred.bounds.len();
                    if bounds_count >= self.config.max_bounds_per_lifetime {
                        let start = lifetime_pred.span().start();
                        violations.push(
                            Violation::new(
                                self.code(),
                                self.name(),
                                self.severity().into(),
                                format!(
                                    "Lifetime '{}' has {} bounds in where clause. Consider simplifying.",
                                    lifetime_pred.lifetime.ident,
                                    bounds_count
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
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::checker::Checker;

    #[test]
    fn test_detects_many_lifetime_params() {
        let code = r#"
            fn many_lifetimes<'a, 'b, 'c>(x: &'a str, y: &'b str, z: &'c str) -> &'a str {
                x
            }
        "#;

        let syntax = syn::parse_file(code).unwrap();
        let checker = E1202ComplexLifetimes::default();

        let mut violations = Vec::new();
        for item in &syntax.items {
            violations.extend(checker.check_item(item, "test.rs").unwrap());
        }

        assert_eq!(violations.len(), 1);
        assert!(violations[0].message.contains("3 lifetime parameters"));
    }

    #[test]
    fn test_detects_complex_lifetime_bounds() {
        let code = r#"
            fn bounded<'a, 'b, 'c>(x: &'a str) -> &'a str
            where
                'a: 'b + 'c
            {
                x
            }
        "#;

        let syntax = syn::parse_file(code).unwrap();
        let checker = E1202ComplexLifetimes::default();

        let mut violations = Vec::new();
        for item in &syntax.items {
            violations.extend(checker.check_item(item, "test.rs").unwrap());
        }

        assert!(!violations.is_empty());
    }

    #[test]
    fn test_simple_lifetimes_pass() {
        let code = r#"
            fn simple<'a>(x: &'a str) -> &'a str {
                x
            }
        "#;

        let syntax = syn::parse_file(code).unwrap();
        let checker = E1202ComplexLifetimes::default();

        let mut violations = Vec::new();
        for item in &syntax.items {
            violations.extend(checker.check_item(item, "test.rs").unwrap());
        }

        assert!(violations.is_empty());
    }

    #[test]
    fn test_elided_lifetimes_pass() {
        let code = r#"
            fn elided(x: &str, y: &str) -> &str {
                x
            }
        "#;

        let syntax = syn::parse_file(code).unwrap();
        let checker = E1202ComplexLifetimes::default();

        let mut violations = Vec::new();
        for item in &syntax.items {
            violations.extend(checker.check_item(item, "test.rs").unwrap());
        }

        assert!(violations.is_empty());
    }
}
