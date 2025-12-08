//! E1108: Deeply nested match expressions
//!
//! Detects match expressions that are nested too deeply, making pattern
//! matching logic hard to follow.

use crate::{checker::Checker, define_checker, violation::Violation};

use syn::visit::Visit;

define_checker! {
    /// Checker for E1108: Deeply nested match expressions
    E1108DeeplyNestedMatch,
    code = "E1108",
    name = "Deeply nested match expressions",
    suggestions = "Extract nested matches into separate functions. Consider using combinators or early returns.",
    target_items = [Function],
    config_entry_name = "e1108_deeply_nested_match",
    /// Configuration for E1108: Deeply nested match checker
    config = E1108Config {
        /// Whether this checker is enabled
        enabled: bool = true,
        /// Severity level, defaults to Medium
        severity: crate::config::SeverityLevel = crate::config::SeverityLevel::Medium,
        /// Categories this checker belongs to
        categories: Vec<crate::config::CheckerCategory> = vec![crate::config::CheckerCategory::Complexity],
        /// Maximum allowed match nesting depth
        max_match_depth: usize = 2,
    },
    check_item(self, item, file_path) {
        let mut visitor = MatchNestingVisitor {
            violations: Vec::new(),
            file_path,
            checker: self,
            current_depth: 0,
        };
        visitor.visit_item(item);
        Ok(visitor.violations)
    }
}

struct MatchNestingVisitor<'a> {
    violations: Vec<Violation>,
    file_path: &'a str,
    checker: &'a E1108DeeplyNestedMatch,
    current_depth: usize,
}

impl<'a> Visit<'a> for MatchNestingVisitor<'a> {
    fn visit_expr_match(&mut self, node: &'a syn::ExprMatch) {
        self.current_depth += 1;

        if self.current_depth > self.checker.config.max_match_depth {
            let start = node.match_token.span.start();
            self.violations.push(
                Violation::new(
                    self.checker.code(),
                    self.checker.name(),
                    self.checker.severity().into(),
                    format!(
                        "Match expression at depth {} exceeds maximum of {}. Nested matches are hard to reason about.",
                        self.current_depth, self.checker.config.max_match_depth
                    ),
                    self.file_path,
                    start.line,
                    start.column + 1,
                )
                .with_suggestion(self.checker.suggestions()),
            );
        }

        syn::visit::visit_expr_match(self, node);
        self.current_depth -= 1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::checker::Checker;

    #[test]
    fn test_single_match_passes() {
        let code = r#"
            fn example(x: Option<i32>) {
                match x {
                    Some(v) => println!("{}", v),
                    None => println!("none"),
                }
            }
        "#;

        let syntax = syn::parse_file(code).unwrap();
        let checker = E1108DeeplyNestedMatch::default();

        let mut violations = Vec::new();
        for item in &syntax.items {
            violations.extend(checker.check_item(item, "test.rs").unwrap());
        }

        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_two_nested_matches_passes() {
        let code = r#"
            fn example(x: Option<Option<i32>>) {
                match x {
                    Some(inner) => match inner {
                        Some(v) => println!("{}", v),
                        None => println!("inner none"),
                    },
                    None => println!("outer none"),
                }
            }
        "#;

        let syntax = syn::parse_file(code).unwrap();
        let checker = E1108DeeplyNestedMatch::default();

        let mut violations = Vec::new();
        for item in &syntax.items {
            violations.extend(checker.check_item(item, "test.rs").unwrap());
        }

        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_detects_deeply_nested_match() {
        let code = r#"
            fn example(x: Option<Option<Option<i32>>>) {
                match x {
                    Some(a) => match a {
                        Some(b) => match b {
                            Some(v) => println!("{}", v),
                            None => {}
                        },
                        None => {}
                    },
                    None => {}
                }
            }
        "#;

        let syntax = syn::parse_file(code).unwrap();
        let checker = E1108DeeplyNestedMatch::default();

        let mut violations = Vec::new();
        for item in &syntax.items {
            violations.extend(checker.check_item(item, "test.rs").unwrap());
        }

        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].code, "E1108");
    }

    #[test]
    fn test_sequential_matches_pass() {
        let code = r#"
            fn example(x: Option<i32>, y: Option<i32>) {
                match x {
                    Some(v) => println!("{}", v),
                    None => {}
                }
                match y {
                    Some(v) => println!("{}", v),
                    None => {}
                }
            }
        "#;

        let syntax = syn::parse_file(code).unwrap();
        let checker = E1108DeeplyNestedMatch::default();

        let mut violations = Vec::new();
        for item in &syntax.items {
            violations.extend(checker.check_item(item, "test.rs").unwrap());
        }

        assert_eq!(violations.len(), 0);
    }
}
