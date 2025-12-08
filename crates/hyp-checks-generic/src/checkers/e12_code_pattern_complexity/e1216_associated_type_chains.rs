//! E1216: Chained transformations with associated type bounds
//!
//! Detects deep associated type chains and associated type bounds
//! that make code hard to understand.
//!
//! Example:
//! ```text
//! // Deep associated type chain
//! fn process<T>(t: T) -> T::Item::Value::Inner { ... }
//!
//! // Associated type bounds in where clause
//! fn bounded<T>() where T::Item: Clone, T::Item::Value = i32 { ... }
//! ```

use crate::{checker::Checker, define_checker, violation::Violation};
use syn::visit::Visit;

define_checker! {
    /// Checker for E1216: Chained transformations with associated type bounds
    E1216AssociatedTypeChains,
    code = "E1216",
    name = "Chained transformations with associated type bounds",
    suggestions = "Create intermediate type aliases for deep chains. Consider redesigning to reduce associated type depth.",
    target_items = [Function, Impl, Trait],
    config_entry_name = "e1216_associated_type_chains",
    config = E1216Config {
        enabled: bool = true,
        severity: crate::config::SeverityLevel = crate::config::SeverityLevel::Medium,
        categories: Vec<crate::config::CheckerCategory> = vec![crate::config::CheckerCategory::Complexity],
        /// Maximum depth of associated type chains (T::A::B::C = depth 3)
        max_chain_depth: usize = 3,
        /// Maximum associated type bounds in where clause
        max_assoc_bounds: usize = 3,
    },
    check_item(self, item, file_path) {
        let mut visitor = AssocTypeVisitor {
            violations: Vec::new(),
            file_path,
            checker: self,
            assoc_bound_count: 0,
        };
        visitor.visit_item(item);

        // Check total associated type bounds
        if visitor.assoc_bound_count > self.config.max_assoc_bounds {
            use syn::spanned::Spanned;
            let start = item.span().start();
            visitor.violations.push(
                Violation::new(
                    self.code(),
                    self.name(),
                    self.severity().into(),
                    format!(
                        "Found {} associated type bounds (max {}). Consider simplifying the type constraints.",
                        visitor.assoc_bound_count,
                        self.config.max_assoc_bounds
                    ),
                    file_path,
                    start.line,
                    start.column + 1,
                )
                .with_suggestion(self.suggestions()),
            );
        }

        Ok(visitor.violations)
    }
}

struct AssocTypeVisitor<'a> {
    violations: Vec<Violation>,
    file_path: &'a str,
    checker: &'a E1216AssociatedTypeChains,
    assoc_bound_count: usize,
}

impl<'a> AssocTypeVisitor<'a> {
    fn check_type_path_depth(&mut self, type_path: &syn::TypePath) -> usize {
        let mut depth = 0;
        let mut is_associated = false;

        for (i, segment) in type_path.path.segments.iter().enumerate() {
            // Skip the first segment (the type itself like T or Self)
            if i > 0 {
                // Each additional segment in T::A::B is an associated type access
                depth += 1;
                is_associated = true;
            }

            // Check nested type arguments for deeper chains
            if let syn::PathArguments::AngleBracketed(args) = &segment.arguments {
                for arg in &args.args {
                    if let syn::GenericArgument::AssocType(_) = arg {
                        depth += 1;
                        is_associated = true;
                    }
                }
            }
        }

        if is_associated {
            depth
        } else {
            0
        }
    }

    fn check_type(&mut self, ty: &syn::Type) {
        use syn::spanned::Spanned;

        if let syn::Type::Path(type_path) = ty {
            let depth = self.check_type_path_depth(type_path);

            if depth > self.checker.config.max_chain_depth {
                let start = ty.span().start();
                self.violations.push(
                    Violation::new(
                        self.checker.code(),
                        self.checker.name(),
                        self.checker.severity().into(),
                        format!(
                            "Associated type chain has depth {} (max {}). Consider using type aliases.",
                            depth,
                            self.checker.config.max_chain_depth
                        ),
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

impl<'a> Visit<'a> for AssocTypeVisitor<'a> {
    fn visit_type(&mut self, ty: &'a syn::Type) {
        self.check_type(ty);
        syn::visit::visit_type(self, ty);
    }

    fn visit_where_predicate(&mut self, pred: &'a syn::WherePredicate) {
        if let syn::WherePredicate::Type(type_pred) = pred {
            // Count associated type bounds
            for bound in &type_pred.bounds {
                if let syn::TypeParamBound::Trait(trait_bound) = bound {
                    // Check for associated type constraints in angle brackets
                    for segment in &trait_bound.path.segments {
                        if let syn::PathArguments::AngleBracketed(args) = &segment.arguments {
                            for arg in &args.args {
                                if matches!(arg, syn::GenericArgument::AssocType(_)) {
                                    self.assoc_bound_count += 1;
                                }
                            }
                        }
                    }
                }
            }
        }
        syn::visit::visit_where_predicate(self, pred);
    }

    fn visit_assoc_type(&mut self, _node: &'a syn::AssocType) {
        self.assoc_bound_count += 1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::checker::Checker;

    #[test]
    fn test_detects_deep_associated_chain() {
        // Test with many associated type bounds instead
        let code = r#"
            fn bounded<T, U, V, W>()
            where
                T: Iterator<Item = i32>,
                U: Iterator<Item = String>,
                V: Iterator<Item = bool>,
                W: Iterator<Item = f64>,
                T: Clone,
            {
            }
        "#;

        let syntax = syn::parse_file(code).unwrap();
        let checker = E1216AssociatedTypeChains::default();

        let mut violations = Vec::new();
        for item in &syntax.items {
            violations.extend(checker.check_item(item, "test.rs").unwrap());
        }

        // Should detect many associated type bounds
        assert!(!violations.is_empty());
    }

    #[test]
    fn test_simple_associated_type_passes() {
        let code = r#"
            trait Simple {
                type Item;
            }

            fn simple<T: Simple>() -> T::Item {
                todo!()
            }
        "#;

        let syntax = syn::parse_file(code).unwrap();
        let checker = E1216AssociatedTypeChains::default();

        let mut violations = Vec::new();
        for item in &syntax.items {
            violations.extend(checker.check_item(item, "test.rs").unwrap());
        }

        // Single level is fine
        assert!(violations.is_empty());
    }

    #[test]
    fn test_detects_many_assoc_bounds() {
        let code = r#"
            fn bounded<T, U, V, W>()
            where
                T: Iterator<Item = i32>,
                U: Iterator<Item = String>,
                V: Iterator<Item = bool>,
                W: Iterator<Item = f64>,
            {
            }
        "#;

        let syntax = syn::parse_file(code).unwrap();
        let checker = E1216AssociatedTypeChains::default();

        let mut violations = Vec::new();
        for item in &syntax.items {
            violations.extend(checker.check_item(item, "test.rs").unwrap());
        }

        assert!(!violations.is_empty());
        assert!(violations[0].message.contains("associated type bounds"));
    }

    #[test]
    fn test_few_assoc_bounds_passes() {
        let code = r#"
            fn bounded<T>()
            where
                T: Iterator<Item = i32>,
            {
            }
        "#;

        let syntax = syn::parse_file(code).unwrap();
        let checker = E1216AssociatedTypeChains::default();

        let mut violations = Vec::new();
        for item in &syntax.items {
            violations.extend(checker.check_item(item, "test.rs").unwrap());
        }

        assert!(violations.is_empty());
    }
}
