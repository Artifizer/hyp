//! E1207: Complex user-defined generic constraints
//!
//! Detects functions with overly complex generic constraints including
//! too many type parameters or overly complex where clauses.
//!
//! Example:
//! ```text
//! // Bad: Too many generic parameters
//! fn complex<A, B, C, D, E, F, G>(a: A, b: B, c: C, d: D, e: E, f: F, g: G) { ... }
//! ```

use crate::{checker::Checker, define_checker, violation::Violation};

define_checker! {
    /// Checker for E1207: Complex user-defined generic constraints
    E1207ComplexConstraints,
    code = "E1207",
    name = "Complex user-defined generic constraints",
    suggestions = "Reduce the number of generic parameters. Consider using trait objects, associated types, or splitting into smaller functions.",
    target_items = [Function, Struct, Enum, Impl, Trait],
    config_entry_name = "e1207_complex_constraints",
    config = E1207Config {
        enabled: bool = true,
        severity: crate::config::SeverityLevel = crate::config::SeverityLevel::High,
        categories: Vec<crate::config::CheckerCategory> = vec![crate::config::CheckerCategory::Complexity],
        /// Maximum number of type parameters
        max_type_params: usize = 4,
        /// Maximum number of const generic parameters
        max_const_params: usize = 2,
    },
    check_item(self, item, file_path) {
        let mut violations = Vec::new();

        let generics = match item {
            syn::Item::Fn(func) => Some(&func.sig.generics),
            syn::Item::Struct(s) => Some(&s.generics),
            syn::Item::Enum(e) => Some(&e.generics),
            syn::Item::Impl(i) => Some(&i.generics),
            syn::Item::Trait(t) => Some(&t.generics),
            _ => None,
        };

        if let Some(generics) = generics {
            self.check_generics(generics, file_path, &mut violations);
        }

        // Also check impl block methods
        if let syn::Item::Impl(impl_block) = item {
            for item in &impl_block.items {
                if let syn::ImplItem::Fn(method) = item {
                    self.check_generics(&method.sig.generics, file_path, &mut violations);
                }
            }
        }

        Ok(violations)
    }
}

impl E1207ComplexConstraints {
    fn check_generics(
        &self,
        generics: &syn::Generics,
        file_path: &str,
        violations: &mut Vec<Violation>,
    ) {
        use syn::spanned::Spanned;

        let type_param_count = generics.type_params().count();
        let const_param_count = generics.const_params().count();

        if type_param_count > self.config.max_type_params {
            let start = generics.span().start();
            violations.push(
                Violation::new(
                    self.code(),
                    self.name(),
                    self.severity().into(),
                    format!(
                        "Function has {} type parameters (max {}). Too many generics increase complexity.",
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

        if const_param_count > self.config.max_const_params {
            let start = generics.span().start();
            violations.push(
                Violation::new(
                    self.code(),
                    self.name(),
                    self.severity().into(),
                    format!(
                        "Function has {} const generic parameters (max {}). Consider simplifying.",
                        const_param_count,
                        self.config.max_const_params
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
    fn test_detects_too_many_type_params() {
        let code = r#"
            fn too_many<A, B, C, D, E>(a: A, b: B, c: C, d: D, e: E) {
            }
        "#;

        let syntax = syn::parse_file(code).unwrap();
        let checker = E1207ComplexConstraints::default();

        let mut violations = Vec::new();
        for item in &syntax.items {
            violations.extend(checker.check_item(item, "test.rs").unwrap());
        }

        assert_eq!(violations.len(), 1);
        assert!(violations[0].message.contains("5 type parameters"));
    }

    #[test]
    fn test_detects_too_many_const_params() {
        let code = r#"
            fn many_const<const N: usize, const M: usize, const K: usize>() {
            }
        "#;

        let syntax = syn::parse_file(code).unwrap();
        let checker = E1207ComplexConstraints::default();

        let mut violations = Vec::new();
        for item in &syntax.items {
            violations.extend(checker.check_item(item, "test.rs").unwrap());
        }

        assert_eq!(violations.len(), 1);
        assert!(violations[0].message.contains("3 const generic parameters"));
    }

    #[test]
    fn test_moderate_params_pass() {
        let code = r#"
            fn moderate<T, U>(t: T, u: U) {
            }
        "#;

        let syntax = syn::parse_file(code).unwrap();
        let checker = E1207ComplexConstraints::default();

        let mut violations = Vec::new();
        for item in &syntax.items {
            violations.extend(checker.check_item(item, "test.rs").unwrap());
        }

        assert!(violations.is_empty());
    }

    #[test]
    fn test_detects_in_struct() {
        let code = r#"
            struct ManyGenerics<A, B, C, D, E> {
                a: A,
                b: B,
                c: C,
                d: D,
                e: E,
            }
        "#;

        let syntax = syn::parse_file(code).unwrap();
        let checker = E1207ComplexConstraints::default();

        let mut violations = Vec::new();
        for item in &syntax.items {
            violations.extend(checker.check_item(item, "test.rs").unwrap());
        }

        assert_eq!(violations.len(), 1);
    }
}
