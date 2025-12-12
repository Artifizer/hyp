//! E1102: Deeply nested logic in loops and conditions
//!
//! Detects code with excessive nesting depth, which makes it hard to follow
//! the control flow and understand the logic.

use crate::{checker::Checker, define_checker, violation::Violation};

use syn::visit::Visit;

define_checker! {
    /// Checker for E1102: Deeply nested logic
    E1102DeeplyNestedLogic,
    code = "E1102",
    name = "Deeply nested logic",
    suggestions = "Extract nested logic into separate functions. Use early returns to reduce nesting.",
    target_items = [Function],
    config_entry_name = "e1102_deeply_nested_logic",
    /// Configuration for E1102: Deeply nested logic checker
    config = E1102Config {
        /// Whether this checker is enabled
        enabled: bool = true,
        /// Severity level, defaults to Medium
        severity: crate::config::SeverityLevel = crate::config::SeverityLevel::Medium,
        /// Categories this checker belongs to
        categories: Vec<crate::config::CheckerCategory> = vec![crate::config::CheckerCategory::Complexity],
        /// Maximum allowed nesting depth
        max_depth: usize = 5,
    },
    check_item(self, item, file_path) {
        let mut visitor = NestingVisitor {
            violations: Vec::new(),
            file_path,
            checker: self,
            current_depth: 0,
            max_depth_seen: 0,
        };
        visitor.visit_item(item);
        Ok(visitor.violations)
    }
}

struct NestingVisitor<'a> {
    violations: Vec<Violation>,
    file_path: &'a str,
    checker: &'a E1102DeeplyNestedLogic,
    current_depth: usize,
    max_depth_seen: usize,
}

impl<'a> NestingVisitor<'a> {
    fn check_and_report(&mut self, span: proc_macro2::Span, kind: &str) {
        if self.current_depth > self.checker.config.max_depth && self.current_depth > self.max_depth_seen {
            self.max_depth_seen = self.current_depth;
            let start = span.start();
            self.violations.push(
                Violation::new(
                    self.checker.code(),
                    self.checker.name(),
                    self.checker.severity().into(),
                    format!(
                        "{} at nesting depth {} exceeds maximum of {}. Deep nesting makes code hard to follow.",
                        kind, self.current_depth, self.checker.config.max_depth
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

impl<'a> Visit<'a> for NestingVisitor<'a> {
    fn visit_expr_if(&mut self, node: &'a syn::ExprIf) {
        self.current_depth += 1;
        self.check_and_report(node.if_token.span, "If statement");
        syn::visit::visit_expr_if(self, node);
        self.current_depth -= 1;
    }

    fn visit_expr_match(&mut self, node: &'a syn::ExprMatch) {
        self.current_depth += 1;
        self.check_and_report(node.match_token.span, "Match expression");
        syn::visit::visit_expr_match(self, node);
        self.current_depth -= 1;
    }

    fn visit_expr_while(&mut self, node: &'a syn::ExprWhile) {
        self.current_depth += 1;
        self.check_and_report(node.while_token.span, "While loop");
        syn::visit::visit_expr_while(self, node);
        self.current_depth -= 1;
    }

    fn visit_expr_for_loop(&mut self, node: &'a syn::ExprForLoop) {
        self.current_depth += 1;
        self.check_and_report(node.for_token.span, "For loop");
        syn::visit::visit_expr_for_loop(self, node);
        self.current_depth -= 1;
    }

    fn visit_expr_loop(&mut self, node: &'a syn::ExprLoop) {
        self.current_depth += 1;
        self.check_and_report(node.loop_token.span, "Loop");
        syn::visit::visit_expr_loop(self, node);
        self.current_depth -= 1;
    }

    fn visit_expr_block(&mut self, node: &'a syn::ExprBlock) {
        // Only count labeled blocks or blocks that aren't part of other constructs
        if node.label.is_some() {
            self.current_depth += 1;
            self.check_and_report(node.block.brace_token.span.open(), "Block");
            syn::visit::visit_expr_block(self, node);
            self.current_depth -= 1;
        } else {
            syn::visit::visit_expr_block(self, node);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::checker::Checker;

    #[test]
    fn test_shallow_nesting_passes() {
        let code = r#"
            fn example() {
                if true {
                    if true {
                        println!("ok");
                    }
                }
            }
        "#;

        let syntax = syn::parse_file(code).unwrap();
        let checker = E1102DeeplyNestedLogic::default();

        let mut violations = Vec::new();
        for item in &syntax.items {
            violations.extend(checker.check_item(item, "test.rs").unwrap());
        }

        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_detects_deep_nesting() {
        let code = r#"
            fn example() {
                if true {
                    if true {
                        if true {
                            if true {
                                if true {
                                    if true {
                                        println!("too deep");
                                    }
                                }
                            }
                        }
                    }
                }
            }
        "#;

        let syntax = syn::parse_file(code).unwrap();
        let checker = E1102DeeplyNestedLogic::default();

        let mut violations = Vec::new();
        for item in &syntax.items {
            violations.extend(checker.check_item(item, "test.rs").unwrap());
        }

        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].code, "E1102");
    }

    #[test]
    fn test_detects_mixed_nesting() {
        let code = r#"
            fn example() {
                for i in 0..10 {
                    while true {
                        if true {
                            match i {
                                0 => {
                                    if true {
                                        if true {
                                            println!("deep");
                                        }
                                    }
                                }
                                _ => {}
                            }
                        }
                    }
                }
            }
        "#;

        let syntax = syn::parse_file(code).unwrap();
        let checker = E1102DeeplyNestedLogic::default();

        let mut violations = Vec::new();
        for item in &syntax.items {
            violations.extend(checker.check_item(item, "test.rs").unwrap());
        }

        assert!(!violations.is_empty());
    }
}
