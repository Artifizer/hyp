//! E1215: Type-level programming with const evaluation
//!
//! Detects const generics used for type-level programming which can
//! be complex and hard to understand.
//!
//! Example:
//! ```text
//! // Const generic arithmetic
//! struct Array<T, const N: usize, const M: usize> {
//!     data: [[T; N]; M],
//! }
//!
//! // Complex const bounds
//! fn process<const N: usize>() where [(); N + 1]: Sized { ... }
//! ```

use crate::{checker::Checker, define_checker, violation::Violation};
use syn::visit::Visit;

define_checker! {
    /// Checker for E1215: Type-level programming with const evaluation
    E1215TypeLevelProgramming,
    code = "E1215",
    name = "Type-level programming with const evaluation",
    suggestions = "Document the purpose of const generics. Consider if runtime values would be simpler.",
    target_items = [Function, Struct, Enum, Impl, Trait],
    config_entry_name = "e1215_type_level_programming",
    config = E1215Config {
        enabled: bool = true,
        severity: crate::config::SeverityLevel = crate::config::SeverityLevel::Low,
        categories: Vec<crate::config::CheckerCategory> = vec![crate::config::CheckerCategory::Complexity],
        /// Maximum const generic params before warning
        max_const_params: usize = 2,
        /// Warn on const generic expressions (arithmetic)
        warn_on_const_expr: bool = true,
    },
    check_item(self, item, file_path) {
        let mut visitor = ConstGenericVisitor {
            violations: Vec::new(),
            file_path,
            checker: self,
        };
        visitor.visit_item(item);
        Ok(visitor.violations)
    }
}

struct ConstGenericVisitor<'a> {
    violations: Vec<Violation>,
    file_path: &'a str,
    checker: &'a E1215TypeLevelProgramming,
}

impl<'a> ConstGenericVisitor<'a> {
    fn check_generics(&mut self, generics: &syn::Generics, context: &str) {
        use syn::spanned::Spanned;

        let const_params: Vec<_> = generics.const_params().collect();
        let const_count = const_params.len();

        if const_count > self.checker.config.max_const_params {
            let start = generics.span().start();
            self.violations.push(
                Violation::new(
                    self.checker.code(),
                    self.checker.name(),
                    crate::violation::CheckerSeverity::Medium.into(),
                    format!(
                        "{} has {} const generic parameters. Multiple const generics indicate type-level programming.",
                        context, const_count
                    ),
                    self.file_path,
                    start.line,
                    start.column + 1,
                )
                .with_suggestion(self.checker.suggestions()),
            );
        }

        // Check for const generic expressions in where clause
        if self.checker.config.warn_on_const_expr {
            if let Some(where_clause) = &generics.where_clause {
                for predicate in &where_clause.predicates {
                    if let syn::WherePredicate::Type(type_pred) = predicate {
                        // Look for array expressions with const generic math
                        if self.contains_const_expr(&type_pred.bounded_ty) {
                            let start = type_pred.span().start();
                            self.violations.push(
                                Violation::new(
                                    self.checker.code(),
                                    self.checker.name(),
                                    crate::violation::CheckerSeverity::Medium.into(),
                                    "Const generic expression in where clause detected. This is advanced type-level programming.",
                                    self.file_path,
                                    start.line,
                                    start.column + 1,
                                )
                                .with_suggestion(self.checker.suggestions()),
                            );
                        }
                    }
                }
            }
        }
    }

    fn contains_const_expr(&self, ty: &syn::Type) -> bool {
        // Check for [(); EXPR]: pattern which is used for const assertions
        if let syn::Type::Array(arr) = ty {
            // Check if the length contains binary operations
            if let syn::Expr::Binary(_) = &arr.len {
                return true;
            }
        }

        false
    }
}

impl<'a> Visit<'a> for ConstGenericVisitor<'a> {
    fn visit_item_fn(&mut self, node: &'a syn::ItemFn) {
        self.check_generics(&node.sig.generics, "Function");
        syn::visit::visit_item_fn(self, node);
    }

    fn visit_item_struct(&mut self, node: &'a syn::ItemStruct) {
        self.check_generics(&node.generics, "Struct");
        syn::visit::visit_item_struct(self, node);
    }

    fn visit_item_enum(&mut self, node: &'a syn::ItemEnum) {
        self.check_generics(&node.generics, "Enum");
        syn::visit::visit_item_enum(self, node);
    }

    fn visit_item_impl(&mut self, node: &'a syn::ItemImpl) {
        self.check_generics(&node.generics, "Impl block");
        syn::visit::visit_item_impl(self, node);
    }

    fn visit_item_trait(&mut self, node: &'a syn::ItemTrait) {
        self.check_generics(&node.generics, "Trait");
        syn::visit::visit_item_trait(self, node);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::checker::Checker;

    #[test]
    fn test_detects_many_const_params() {
        let code = r#"
            struct Matrix<const N: usize, const M: usize, const K: usize> {
                data: [[[f64; N]; M]; K],
            }
        "#;

        let syntax = syn::parse_file(code).unwrap();
        let checker = E1215TypeLevelProgramming::default();

        let mut violations = Vec::new();
        for item in &syntax.items {
            violations.extend(checker.check_item(item, "test.rs").unwrap());
        }

        assert_eq!(violations.len(), 1);
        assert!(violations[0].message.contains("3 const generic"));
    }

    #[test]
    fn test_single_const_param_passes() {
        let code = r#"
            struct Array<const N: usize> {
                data: [i32; N],
            }
        "#;

        let syntax = syn::parse_file(code).unwrap();
        let checker = E1215TypeLevelProgramming::default();

        let mut violations = Vec::new();
        for item in &syntax.items {
            violations.extend(checker.check_item(item, "test.rs").unwrap());
        }

        assert!(violations.is_empty());
    }

    #[test]
    fn test_detects_const_expr_in_where() {
        let code = r#"
            fn complex<const N: usize>()
            where
                [(); N + 1]: Sized
            {
            }
        "#;

        let syntax = syn::parse_file(code).unwrap();
        let checker = E1215TypeLevelProgramming::default();

        let mut violations = Vec::new();
        for item in &syntax.items {
            violations.extend(checker.check_item(item, "test.rs").unwrap());
        }

        assert!(!violations.is_empty());
    }

    #[test]
    fn test_no_const_generics_passes() {
        let code = r#"
            fn simple<T>(x: T) -> T {
                x
            }
        "#;

        let syntax = syn::parse_file(code).unwrap();
        let checker = E1215TypeLevelProgramming::default();

        let mut violations = Vec::new();
        for item in &syntax.items {
            violations.extend(checker.check_item(item, "test.rs").unwrap());
        }

        assert!(violations.is_empty());
    }
}
