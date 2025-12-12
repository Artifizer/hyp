//! E1708: Inefficient data structure
//!
//! Detects potentially inefficient data structure choices,
//! such as using Vec for membership tests instead of HashSet.

use crate::{checker::Checker, define_checker, violation::Violation};

use syn::visit::Visit;

define_checker! {
    /// Checker for E1708: Inefficient data structure
    E1708InefficientDataStructure,
    code = "E1708",
    name = "Inefficient data structure",
    suggestions = "Consider using HashSet for membership tests or HashMap for key-value lookups",
    target_items = [Function],
    config_entry_name = "e1708_inefficient_data_structure",
    config = E1708Config {
        enabled: bool = true,
        severity: crate::config::SeverityLevel = crate::config::SeverityLevel::Low,
        categories: Vec<crate::config::CheckerCategory> = vec![crate::config::CheckerCategory::Complexity],
    },
    check_item(self, item, file_path) {
        let mut visitor = DataStructureVisitor {
            violations: Vec::new(),
            file_path,
            checker: self,
        };
        visitor.visit_item(item);
        Ok(visitor.violations)
    }
}

struct DataStructureVisitor<'a> {
    violations: Vec<Violation>,
    file_path: &'a str,
    checker: &'a E1708InefficientDataStructure,
}

impl<'a> Visit<'a> for DataStructureVisitor<'a> {
    fn visit_expr_method_call(&mut self, node: &'a syn::ExprMethodCall) {
        let method_name = node.method.to_string();

        // Check for .contains() on Vec-like types inside loops or frequent paths
        // This is a heuristic - we flag Vec::contains as potentially inefficient
        if method_name == "contains" {
            // Check if receiver looks like a Vec (ends with .iter() or is a variable)
            if let syn::Expr::MethodCall(inner) = &*node.receiver {
                if inner.method == "iter" {
                    use syn::spanned::Spanned;
                    let start = node.span().start();
                    self.violations.push(
                        Violation::new(
                            self.checker.code(),
                            self.checker.name(),
                            self.checker.severity().into(),
                            ".iter().contains() on a collection is O(n). Consider using HashSet for O(1) lookups.",
                            self.file_path,
                            start.line,
                            start.column + 1,
                        )
                        .with_suggestion(self.checker.suggestions()),
                    );
                }
            }
        }

        // Check for .find() patterns that might benefit from HashMap
        if method_name == "find" {
            if let syn::Expr::MethodCall(inner) = &*node.receiver {
                if inner.method == "iter" {
                    use syn::spanned::Spanned;
                    let start = node.span().start();
                    self.violations.push(
                        Violation::new(
                            self.checker.code(),
                            self.checker.name(),
                            self.checker.severity().into(),
                            ".iter().find() is O(n). Consider using HashMap for O(1) key lookups.",
                            self.file_path,
                            start.line,
                            start.column + 1,
                        )
                        .with_suggestion(self.checker.suggestions()),
                    );
                }
            }
        }

        syn::visit::visit_expr_method_call(self, node);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detects_iter_contains() {
        let code = r#"
            fn example(v: Vec<i32>, x: i32) -> bool {
                v.iter().contains(&x)
            }
        "#;

        let syntax = syn::parse_file(code).unwrap();
        let checker = E1708InefficientDataStructure::default();

        let mut violations = Vec::new();
        for item in &syntax.items {
            violations.extend(checker.check_item(item, "test.rs").unwrap());
        }

        assert_eq!(violations.len(), 1);
    }

    #[test]
    fn test_hashset_contains_passes() {
        let code = r#"
            use std::collections::HashSet;

            fn example(s: HashSet<i32>, x: i32) -> bool {
                s.contains(&x)
            }
        "#;

        let syntax = syn::parse_file(code).unwrap();
        let checker = E1708InefficientDataStructure::default();

        let mut violations = Vec::new();
        for item in &syntax.items {
            violations.extend(checker.check_item(item, "test.rs").unwrap());
        }

        assert_eq!(violations.len(), 0);
    }
}
