//! E1201: Overly complex generics
//!
//! Detects functions with too many generic type parameters or overly complex trait bounds,
//! which reduce code readability and maintainability.

use crate::{define_checker, violation::Violation};

use syn::spanned::Spanned;

define_checker! {
    /// Checker for E1201: Overly complex generics
    E1201ComplexGenerics,
    code = "E1201",
    name = "Overly complex generics",
    suggestions = "Reduce type parameters to 3 or fewer, use concrete types, or group related parameters into a trait",
    target_items = [Function],
    config_entry_name = "e1201_complex_generics",
    config = E1201Config {
        enabled: bool = true,
        severity: crate::config::SeverityLevel = crate::config::SeverityLevel::Medium,
        categories: Vec<crate::config::CheckerCategory> = vec![crate::config::CheckerCategory::Complexity],
        /// Maximum number of generic type parameters allowed
        max_type_params: usize = 3,
        /// Maximum number of trait bounds per type parameter
        max_bounds_per_param: usize = 4,
    },
    check_item(self, item, file_path) {
        let mut violations = Vec::new();

        if let syn::Item::Fn(func) = item {
            let generics = &func.sig.generics;
            let type_param_count = generics.type_params().count();

            // Check for too many type parameters
            if type_param_count > self.config.max_type_params {
                let start = func.sig.ident.span().start();
                violations.push(
                    Violation::new(
                        self.code(),
                        self.name(),
                        self.severity().into(),
                        format!(
                            "Function '{}' has {} type parameters (max {}). Consider simplifying.",
                            func.sig.ident,
                            type_param_count,
                            self.config.max_type_params
                        ),
                        file_path,
                        start.line,
                        start.column + 1,
                    )
                    .with_suggestion(self.suggestions()),
                );
            }

            // Check for overly complex bounds on each type parameter
            for type_param in generics.type_params() {
                let bound_count = type_param.bounds.len();
                if bound_count > self.config.max_bounds_per_param {
                    let start = type_param.span().start();
                    violations.push(
                        Violation::new(
                            self.code(),
                            self.name(),
                            self.severity().into(),
                            format!(
                                "Type parameter '{}' has {} trait bounds (max {}). Consider simplifying.",
                                type_param.ident,
                                bound_count,
                                self.config.max_bounds_per_param
                            ),
                            file_path,
                            start.line,
                            start.column + 1,
                        )
                        .with_suggestion(self.suggestions()),
                    );
                }
            }

            // Also check where clause bounds
            if let Some(where_clause) = &generics.where_clause {
                for predicate in &where_clause.predicates {
                    if let syn::WherePredicate::Type(type_pred) = predicate {
                        let bound_count = type_pred.bounds.len();
                        if bound_count > self.config.max_bounds_per_param {
                            let start = type_pred.span().start();
                            violations.push(
                                Violation::new(
                                    self.code(),
                                    self.name(),
                                    self.severity().into(),
                                    format!(
                                        "Where clause has {} trait bounds (max {}). Consider simplifying.",
                                        bound_count,
                                        self.config.max_bounds_per_param
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

        Ok(violations)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::checker::Checker;

    #[test]
    fn test_detects_too_many_type_params() {
        let code = r#"
            fn complex<T, U, V, W, X>(a: T, b: U, c: V, d: W, e: X) {}
        "#;

        let syntax = syn::parse_file(code).unwrap();
        let checker = E1201ComplexGenerics::default();

        let mut violations = Vec::new();
        for item in &syntax.items {
            violations.extend(checker.check_item(item, "test.rs").unwrap());
        }

        assert_eq!(violations.len(), 1);
        assert!(violations[0].message.contains("5 type parameters"));
    }

    #[test]
    fn test_detects_too_many_bounds() {
        let code = r#"
            fn complex<T: Clone + Send + Sync + Debug + Default>(a: T) {}
        "#;

        let syntax = syn::parse_file(code).unwrap();
        let checker = E1201ComplexGenerics::default();

        let mut violations = Vec::new();
        for item in &syntax.items {
            violations.extend(checker.check_item(item, "test.rs").unwrap());
        }

        assert_eq!(violations.len(), 1);
        assert!(violations[0].message.contains("5 trait bounds"));
    }

    #[test]
    fn test_detects_complex_where_clause() {
        let code = r#"
            fn complex<T>(a: T)
            where
                T: Clone + Send + Sync + Debug + Default + PartialEq
            {}
        "#;

        let syntax = syn::parse_file(code).unwrap();
        let checker = E1201ComplexGenerics::default();

        let mut violations = Vec::new();
        for item in &syntax.items {
            violations.extend(checker.check_item(item, "test.rs").unwrap());
        }

        assert_eq!(violations.len(), 1);
        assert!(violations[0].message.contains("6 trait bounds"));
    }

    #[test]
    fn test_simple_generics_pass() {
        let code = r#"
            fn simple<T: Clone>(a: T) {}
            fn two_params<T, U>(a: T, b: U) {}
            fn three_params<T, U, V>(a: T, b: U, c: V) {}
        "#;

        let syntax = syn::parse_file(code).unwrap();
        let checker = E1201ComplexGenerics::default();

        let mut violations = Vec::new();
        for item in &syntax.items {
            violations.extend(checker.check_item(item, "test.rs").unwrap());
        }

        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_no_generics_pass() {
        let code = r#"
            fn no_generics(a: i32, b: String) {}
        "#;

        let syntax = syn::parse_file(code).unwrap();
        let checker = E1201ComplexGenerics::default();

        let mut violations = Vec::new();
        for item in &syntax.items {
            violations.extend(checker.check_item(item, "test.rs").unwrap());
        }

        assert_eq!(violations.len(), 0);
    }
}
