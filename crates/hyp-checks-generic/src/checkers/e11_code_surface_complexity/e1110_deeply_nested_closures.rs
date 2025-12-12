//! E1110: Deeply nested callbacks/closures
//!
//! Detects closures nested beyond a configurable depth threshold.
//! Deeply nested closures make code hard to read and maintain.
//!
//! Example:
//! ```text
//! // Bad: 5 levels of nesting
//! let result = items.iter()
//!     .map(|x| {           // depth 1
//!         x.iter()
//!             .filter(|y| {  // depth 2
//!                 y.map(|z| {  // depth 3
//!                     z.and_then(|a| {  // depth 4
//!                         a.or_else(|b| { b + 1 })  // depth 5
//!                     })
//!                 })
//!             })
//!     });
//! ```

use crate::{checker::Checker, define_checker, violation::Violation};
use syn::visit::Visit;

define_checker! {
    /// Checker for E1110: Deeply nested callbacks/closures
    E1110DeeplyNestedClosures,
    code = "E1110",
    name = "Deeply nested callbacks/closures",
    suggestions = "Extract nested closures into named functions or use early returns. Consider using iterators with fewer chained operations.",
    target_items = [Function, Impl],
    config_entry_name = "e1110_deeply_nested_closures",
    config = E1110Config {
        enabled: bool = true,
        severity: crate::config::SeverityLevel = crate::config::SeverityLevel::Medium,
        categories: Vec<crate::config::CheckerCategory> = vec![crate::config::CheckerCategory::Complexity],
        /// Maximum allowed closure nesting depth
        max_depth: usize = 4,
    },
    check_item(self, item, file_path) {
        let mut visitor = ClosureNestingVisitor {
            violations: Vec::new(),
            file_path,
            checker: self,
            current_depth: 0,
            max_depth_found: 0,
        };
        visitor.visit_item(item);
        Ok(visitor.violations)
    }
}

struct ClosureNestingVisitor<'a> {
    violations: Vec<Violation>,
    file_path: &'a str,
    checker: &'a E1110DeeplyNestedClosures,
    current_depth: usize,
    max_depth_found: usize,
}

impl<'a> Visit<'a> for ClosureNestingVisitor<'a> {
    fn visit_expr_closure(&mut self, node: &'a syn::ExprClosure) {
        use syn::spanned::Spanned;

        self.current_depth += 1;

        if self.current_depth > self.max_depth_found {
            self.max_depth_found = self.current_depth;
        }

        // Report when we exceed the threshold
        if self.current_depth > self.checker.config.max_depth {
            let start = node.span().start();
            self.violations.push(
                Violation::new(
                    self.checker.code(),
                    self.checker.name(),
                    self.checker.severity().into(),
                    format!(
                        "Closure nested {} levels deep (max {}). Deeply nested closures are hard to read and maintain.",
                        self.current_depth,
                        self.checker.config.max_depth
                    ),
                    self.file_path,
                    start.line,
                    start.column + 1,
                )
                .with_suggestion(self.checker.suggestions()),
            );
        }

        // Continue visiting nested closures
        syn::visit::visit_expr_closure(self, node);

        self.current_depth -= 1;
    }

    // Also track async blocks as they behave like closures
    fn visit_expr_async(&mut self, node: &'a syn::ExprAsync) {
        use syn::spanned::Spanned;

        self.current_depth += 1;

        if self.current_depth > self.max_depth_found {
            self.max_depth_found = self.current_depth;
        }

        if self.current_depth > self.checker.config.max_depth {
            let start = node.span().start();
            self.violations.push(
                Violation::new(
                    self.checker.code(),
                    self.checker.name(),
                    self.checker.severity().into(),
                    format!(
                        "Async block nested {} levels deep (max {}). Consider extracting into a named async function.",
                        self.current_depth,
                        self.checker.config.max_depth
                    ),
                    self.file_path,
                    start.line,
                    start.column + 1,
                )
                .with_suggestion(self.checker.suggestions()),
            );
        }

        syn::visit::visit_expr_async(self, node);

        self.current_depth -= 1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::checker::Checker;

    #[test]
    fn test_detects_deeply_nested_closures() {
        let code = r#"
            fn nested_closures() {
                let _ = vec![1, 2, 3]
                    .iter()
                    .map(|x| {             // depth 1
                        vec![1, 2]
                            .iter()
                            .map(|y| {     // depth 2
                                vec![1]
                                    .iter()
                                    .map(|z| {   // depth 3
                                        vec![1]
                                            .iter()
                                            .map(|a| {  // depth 4
                                                vec![1]
                                                    .iter()
                                                    .map(|b| b + 1)  // depth 5
                                            })
                                    })
                            })
                    });
            }
        "#;

        let syntax = syn::parse_file(code).unwrap();
        let checker = E1110DeeplyNestedClosures::default();

        let mut violations = Vec::new();
        for item in &syntax.items {
            violations.extend(checker.check_item(item, "test.rs").unwrap());
        }

        assert!(!violations.is_empty());
        assert!(violations[0].message.contains("5 levels"));
    }

    #[test]
    fn test_shallow_closures_pass() {
        let code = r#"
            fn shallow_closures() {
                let _ = vec![1, 2, 3]
                    .iter()
                    .map(|x| {           // depth 1
                        x.checked_add(1)
                            .map(|y| y * 2)  // depth 2
                    });
            }
        "#;

        let syntax = syn::parse_file(code).unwrap();
        let checker = E1110DeeplyNestedClosures::default();

        let mut violations = Vec::new();
        for item in &syntax.items {
            violations.extend(checker.check_item(item, "test.rs").unwrap());
        }

        assert!(violations.is_empty());
    }

    #[test]
    fn test_detects_nested_async_blocks() {
        let code = r#"
            async fn nested_async() {
                let _ = async {          // depth 1
                    let _ = async {      // depth 2
                        let _ = async {  // depth 3
                            let _ = async {  // depth 4
                                let _ = async {  // depth 5
                                    42
                                };
                            };
                        };
                    };
                };
            }
        "#;

        let syntax = syn::parse_file(code).unwrap();
        let checker = E1110DeeplyNestedClosures::default();

        let mut violations = Vec::new();
        for item in &syntax.items {
            violations.extend(checker.check_item(item, "test.rs").unwrap());
        }

        assert!(!violations.is_empty());
    }

    #[test]
    fn test_custom_threshold() {
        let code = r#"
            fn closures() {
                let _ = vec![1].iter().map(|x| {  // depth 1
                    vec![1].iter().map(|y| {      // depth 2
                        y + 1
                    })
                });
            }
        "#;

        let syntax = syn::parse_file(code).unwrap();
        let mut checker = E1110DeeplyNestedClosures::default();
        checker.config.max_depth = 1; // Set low threshold

        let mut violations = Vec::new();
        for item in &syntax.items {
            violations.extend(checker.check_item(item, "test.rs").unwrap());
        }

        assert_eq!(violations.len(), 1);
        assert!(violations[0].message.contains("2 levels"));
    }
}
