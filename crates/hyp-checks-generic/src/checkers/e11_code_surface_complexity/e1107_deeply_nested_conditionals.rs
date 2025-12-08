//! E1107: Deeply nested conditionals
//!
//! Detects if/else chains that are nested too deeply, making the logic
//! hard to follow. Similar to E1102 but specifically targets if statements.

use crate::{checker::Checker, define_checker, violation::Violation};

use syn::visit::Visit;

define_checker! {
    /// Checker for E1107: Deeply nested conditionals
    E1107DeeplyNestedConditionals,
    code = "E1107",
    name = "Deeply nested conditionals",
    suggestions = "Use early returns, guard clauses, or extract logic into helper functions",
    target_items = [Function],
    config_entry_name = "e1107_deeply_nested_conditionals",
    /// Configuration for E1107: Deeply nested conditionals checker
    config = E1107Config {
        /// Whether this checker is enabled
        enabled: bool = true,
        /// Severity level, defaults to Medium
        severity: crate::config::SeverityLevel = crate::config::SeverityLevel::Medium,
        /// Categories this checker belongs to
        categories: Vec<crate::config::CheckerCategory> = vec![crate::config::CheckerCategory::Complexity],
        /// Maximum allowed if nesting depth
        max_if_depth: usize = 3,
    },
    check_item(self, item, file_path) {
        let mut visitor = IfNestingVisitor {
            violations: Vec::new(),
            file_path,
            checker: self,
            current_depth: 0,
        };
        visitor.visit_item(item);
        Ok(visitor.violations)
    }
}

struct IfNestingVisitor<'a> {
    violations: Vec<Violation>,
    file_path: &'a str,
    checker: &'a E1107DeeplyNestedConditionals,
    current_depth: usize,
}

impl<'a> Visit<'a> for IfNestingVisitor<'a> {
    fn visit_expr_if(&mut self, node: &'a syn::ExprIf) {
        self.current_depth += 1;

        if self.current_depth > self.checker.config.max_if_depth {
            let start = node.if_token.span.start();
            self.violations.push(
                Violation::new(
                    self.checker.code(),
                    self.checker.name(),
                    self.checker.severity().into(),
                    format!(
                        "If statement at depth {} exceeds maximum of {}. Deep nesting makes conditional logic hard to follow.",
                        self.current_depth, self.checker.config.max_if_depth
                    ),
                    self.file_path,
                    start.line,
                    start.column + 1,
                )
                .with_suggestion(self.checker.suggestions()),
            );
        }

        syn::visit::visit_expr_if(self, node);
        self.current_depth -= 1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::checker::Checker;

    #[test]
    fn test_shallow_if_passes() {
        let code = r#"
            fn example(x: i32) {
                if x > 0 {
                    if x > 10 {
                        println!("big");
                    }
                }
            }
        "#;

        let syntax = syn::parse_file(code).unwrap();
        let checker = E1107DeeplyNestedConditionals::default();

        let mut violations = Vec::new();
        for item in &syntax.items {
            violations.extend(checker.check_item(item, "test.rs").unwrap());
        }

        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_detects_deep_if_nesting() {
        let code = r#"
            fn example(x: i32) {
                if x > 0 {
                    if x > 10 {
                        if x > 100 {
                            if x > 1000 {
                                println!("very big");
                            }
                        }
                    }
                }
            }
        "#;

        let syntax = syn::parse_file(code).unwrap();
        let checker = E1107DeeplyNestedConditionals::default();

        let mut violations = Vec::new();
        for item in &syntax.items {
            violations.extend(checker.check_item(item, "test.rs").unwrap());
        }

        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].code, "E1107");
    }

    #[test]
    fn test_else_if_chain_passes() {
        // else if doesn't increase nesting depth
        let code = r#"
            fn example(x: i32) {
                if x > 100 {
                    println!("big");
                } else if x > 50 {
                    println!("medium");
                } else if x > 10 {
                    println!("small");
                } else {
                    println!("tiny");
                }
            }
        "#;

        let syntax = syn::parse_file(code).unwrap();
        let checker = E1107DeeplyNestedConditionals::default();

        let mut violations = Vec::new();
        for item in &syntax.items {
            violations.extend(checker.check_item(item, "test.rs").unwrap());
        }

        // else if chains don't nest, they're sequential
        assert_eq!(violations.len(), 0);
    }
}
