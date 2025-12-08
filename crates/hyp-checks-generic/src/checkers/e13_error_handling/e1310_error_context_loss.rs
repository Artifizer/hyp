//! E1310: Error context loss
//!
//! Detects when map_err uses `|_|` to discard the original error, losing
//! valuable debugging information about what actually went wrong.

use crate::{checker::Checker, define_checker, violation::Violation};

use syn::visit::Visit;

define_checker! {
    /// Checker for E1310: Error context loss
    E1310ErrorContextLoss,
    code = "E1310",
    name = "Error context loss",
    suggestions = "Preserve the original error by wrapping it, not discarding with |_|. Use anyhow/thiserror for error chains",
    target_items = [Function],
    config_entry_name = "e1310_error_context_loss",
    config = E1310Config {
        enabled: bool = true,
        severity: crate::config::SeverityLevel = crate::config::SeverityLevel::Medium,
        categories: Vec<crate::config::CheckerCategory> = vec![crate::config::CheckerCategory::Operations],
    },
    check_item(self, item, file_path) {
        let mut visitor = ContextLossVisitor {
            violations: Vec::new(),
            file_path,
            checker: self,
        };
        visitor.visit_item(item);
        Ok(visitor.violations)
    }
}

struct ContextLossVisitor<'a> {
    violations: Vec<Violation>,
    file_path: &'a str,
    checker: &'a E1310ErrorContextLoss,
}

impl<'a> Visit<'a> for ContextLossVisitor<'a> {
    fn visit_expr_method_call(&mut self, node: &'a syn::ExprMethodCall) {
        let method_name = node.method.to_string();

        // Check for map_err with discarding closure |_| ...
        if method_name == "map_err" {
            if let Some(arg) = node.args.first() {
                if is_discarding_closure(arg) {
                    use syn::spanned::Spanned;
                    let start = node.span().start();
                    self.violations.push(
                        Violation::new(
                            self.checker.code(),
                            self.checker.name(),
                            self.checker.severity().into(),
                            "map_err with |_| discards original error context. Preserve error information for debugging.",
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

fn is_discarding_closure(expr: &syn::Expr) -> bool {
    if let syn::Expr::Closure(closure) = expr {
        // Check if the closure parameter is _ (wildcard)
        if closure.inputs.len() == 1 {
            if let syn::Pat::Wild(_) = &closure.inputs[0] {
                return true;
            }
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::checker::Checker;

    #[test]
    fn test_detects_discarded_error() {
        let code = r#"
            fn example() -> Result<i32, &'static str> {
                std::fs::read_to_string("config.txt")
                    .map_err(|_| "Failed to read file")?;
                Ok(42)
            }
        "#;

        let syntax = syn::parse_file(code).unwrap();
        let checker = E1310ErrorContextLoss::default();

        let mut violations = Vec::new();
        for item in &syntax.items {
            violations.extend(checker.check_item(item, "test.rs").unwrap());
        }

        assert_eq!(violations.len(), 1);
    }

    #[test]
    fn test_preserved_error_passes() {
        let code = r#"
            fn example() -> Result<i32, String> {
                std::fs::read_to_string("config.txt")
                    .map_err(|e| format!("Failed to read file: {}", e))?;
                Ok(42)
            }
        "#;

        let syntax = syn::parse_file(code).unwrap();
        let checker = E1310ErrorContextLoss::default();

        let mut violations = Vec::new();
        for item in &syntax.items {
            violations.extend(checker.check_item(item, "test.rs").unwrap());
        }

        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_question_mark_passes() {
        let code = r#"
            fn example() -> std::io::Result<i32> {
                let _ = std::fs::read_to_string("config.txt")?;
                Ok(42)
            }
        "#;

        let syntax = syn::parse_file(code).unwrap();
        let checker = E1310ErrorContextLoss::default();

        let mut violations = Vec::new();
        for item in &syntax.items {
            violations.extend(checker.check_item(item, "test.rs").unwrap());
        }

        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_map_err_with_named_param_passes() {
        let code = r#"
            fn example() -> Result<i32, String> {
                std::fs::read_to_string("config.txt")
                    .map_err(|e| e.to_string())?;
                Ok(42)
            }
        "#;

        let syntax = syn::parse_file(code).unwrap();
        let checker = E1310ErrorContextLoss::default();

        let mut violations = Vec::new();
        for item in &syntax.items {
            violations.extend(checker.check_item(item, "test.rs").unwrap());
        }

        assert_eq!(violations.len(), 0);
    }
}
