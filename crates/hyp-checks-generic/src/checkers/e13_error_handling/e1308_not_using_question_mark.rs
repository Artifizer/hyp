//! E1308: Not using ? operator when appropriate
//!
//! Detects patterns like `match result { Ok(v) => v, Err(e) => return Err(e) }`
//! that could be simplified with the ? operator.

use crate::{checker::Checker, define_checker, violation::Violation};

use syn::visit::Visit;

define_checker! {
    /// Checker for E1308: Not using ? operator when appropriate
    E1308NotUsingQuestionMark,
    code = "E1308",
    name = "Not using ? operator",
    suggestions = "Use the ? operator for cleaner error propagation",
    target_items = [Function],
    config_entry_name = "e1308_not_using_question_mark",
    config = E1308Config {
        enabled: bool = true,
        severity: crate::config::SeverityLevel = crate::config::SeverityLevel::Low,
        categories: Vec<crate::config::CheckerCategory> = vec![crate::config::CheckerCategory::Complexity],
    },
    check_item(self, item, file_path) {
        let mut visitor = QuestionMarkVisitor {
            violations: Vec::new(),
            file_path,
            checker: self,
        };
        visitor.visit_item(item);
        Ok(visitor.violations)
    }
}

struct QuestionMarkVisitor<'a> {
    violations: Vec<Violation>,
    file_path: &'a str,
    checker: &'a E1308NotUsingQuestionMark,
}

impl<'a> Visit<'a> for QuestionMarkVisitor<'a> {
    fn visit_expr_match(&mut self, node: &'a syn::ExprMatch) {
        // Check for pattern: match x { Ok(v) => v, Err(e) => return Err(e) }
        if node.arms.len() == 2 {
            let has_ok_unwrap = node.arms.iter().any(is_ok_unwrap_arm);
            let has_err_return = node.arms.iter().any(is_err_return_arm);

            if has_ok_unwrap && has_err_return {
                let start = node.match_token.span.start();
                self.violations.push(
                    Violation::new(
                        self.checker.code(),
                        self.checker.name(),
                        self.checker.severity().into(),
                        "Match expression could be replaced with the ? operator.",
                        self.file_path,
                        start.line,
                        start.column + 1,
                    )
                    .with_suggestion(self.checker.suggestions()),
                );
            }
        }

        syn::visit::visit_expr_match(self, node);
    }
}

fn is_ok_unwrap_arm(arm: &syn::Arm) -> bool {
    if let syn::Pat::TupleStruct(ts) = &arm.pat {
        let path_str = ts.path.segments.iter()
            .map(|s| s.ident.to_string())
            .collect::<Vec<_>>()
            .join("::");
        if path_str == "Ok" || path_str == "Some" {
            // Check if body is just the unwrapped value
            if let syn::Expr::Path(_) = &*arm.body {
                return true;
            }
        }
    }
    false
}

fn is_err_return_arm(arm: &syn::Arm) -> bool {
    if let syn::Pat::TupleStruct(ts) = &arm.pat {
        let path_str = ts.path.segments.iter()
            .map(|s| s.ident.to_string())
            .collect::<Vec<_>>()
            .join("::");
        if path_str == "Err" || path_str == "None" {
            // Check if body is return Err(...)
            if let syn::Expr::Return(_) = &*arm.body {
                return true;
            }
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detects_manual_error_propagation() {
        let code = r#"
            fn example(r: Result<i32, String>) -> Result<i32, String> {
                let v = match r {
                    Ok(v) => v,
                    Err(e) => return Err(e),
                };
                Ok(v + 1)
            }
        "#;

        let syntax = syn::parse_file(code).unwrap();
        let checker = E1308NotUsingQuestionMark::default();

        let mut violations = Vec::new();
        for item in &syntax.items {
            violations.extend(checker.check_item(item, "test.rs").unwrap());
        }

        assert_eq!(violations.len(), 1);
    }

    #[test]
    fn test_question_mark_passes() {
        let code = r#"
            fn example(r: Result<i32, String>) -> Result<i32, String> {
                let v = r?;
                Ok(v + 1)
            }
        "#;

        let syntax = syn::parse_file(code).unwrap();
        let checker = E1308NotUsingQuestionMark::default();

        let mut violations = Vec::new();
        for item in &syntax.items {
            violations.extend(checker.check_item(item, "test.rs").unwrap());
        }

        assert_eq!(violations.len(), 0);
    }
}
