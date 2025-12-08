//! E1109: Excessive method chaining
//!
//! Detects long chains of method calls that become hard to read and debug.
//! While method chaining is idiomatic in Rust, excessive chaining hurts readability.

use crate::{checker::Checker, define_checker, violation::Violation};

use syn::visit::Visit;

define_checker! {
    /// Checker for E1109: Excessive method chaining
    E1109ExcessiveChaining,
    code = "E1109",
    name = "Excessive method chaining",
    suggestions = "Break long chains into intermediate variables with meaningful names",
    target_items = [Function],
    config_entry_name = "e1109_excessive_chaining",
    /// Configuration for E1109: Excessive chaining checker
    config = E1109Config {
        /// Whether this checker is enabled
        enabled: bool = true,
        /// Severity level, defaults to Low
        severity: crate::config::SeverityLevel = crate::config::SeverityLevel::Low,
        /// Categories this checker belongs to
        categories: Vec<crate::config::CheckerCategory> = vec![crate::config::CheckerCategory::Complexity],
        /// Maximum allowed chain length
        max_chain_length: usize = 5,
    },
    check_item(self, item, file_path) {
        let mut visitor = ChainVisitor {
            violations: Vec::new(),
            file_path,
            checker: self,
        };
        visitor.visit_item(item);
        Ok(visitor.violations)
    }
}

struct ChainVisitor<'a> {
    violations: Vec<Violation>,
    file_path: &'a str,
    checker: &'a E1109ExcessiveChaining,
}

impl<'a> ChainVisitor<'a> {
    fn count_chain_length(&self, expr: &syn::Expr) -> usize {
        match expr {
            syn::Expr::MethodCall(call) => 1 + self.count_chain_length(&call.receiver),
            syn::Expr::Field(field) => 1 + self.count_chain_length(&field.base),
            syn::Expr::Try(try_expr) => self.count_chain_length(&try_expr.expr),
            syn::Expr::Await(await_expr) => 1 + self.count_chain_length(&await_expr.base),
            _ => 0,
        }
    }
}

impl<'a> Visit<'a> for ChainVisitor<'a> {
    fn visit_expr_method_call(&mut self, node: &'a syn::ExprMethodCall) {
        let chain_length = self.count_chain_length(&syn::Expr::MethodCall(node.clone()));

        if chain_length > self.checker.config.max_chain_length {
            let start = node.method.span().start();
            self.violations.push(
                Violation::new(
                    self.checker.code(),
                    self.checker.name(),
                    self.checker.severity().into(),
                    format!(
                        "Method chain of length {} exceeds maximum of {}. Long chains are hard to read and debug.",
                        chain_length, self.checker.config.max_chain_length
                    ),
                    self.file_path,
                    start.line,
                    start.column + 1,
                )
                .with_suggestion(self.checker.suggestions()),
            );
        }

        // Don't recurse into the chain - we've already counted it
        // But do visit the arguments
        for arg in &node.args {
            self.visit_expr(arg);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::checker::Checker;

    #[test]
    fn test_short_chain_passes() {
        let code = r#"
            fn example() {
                let result = vec![1, 2, 3]
                    .iter()
                    .map(|x| x * 2)
                    .collect::<Vec<_>>();
            }
        "#;

        let syntax = syn::parse_file(code).unwrap();
        let checker = E1109ExcessiveChaining::default();

        let mut violations = Vec::new();
        for item in &syntax.items {
            violations.extend(checker.check_item(item, "test.rs").unwrap());
        }

        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_detects_long_chain() {
        let code = r#"
            fn example() {
                let result = vec![1, 2, 3]
                    .iter()
                    .filter(|x| **x > 0)
                    .map(|x| x * 2)
                    .filter(|x| *x > 2)
                    .map(|x| x + 1)
                    .take(10)
                    .collect::<Vec<_>>();
            }
        "#;

        let syntax = syn::parse_file(code).unwrap();
        let checker = E1109ExcessiveChaining::default();

        let mut violations = Vec::new();
        for item in &syntax.items {
            violations.extend(checker.check_item(item, "test.rs").unwrap());
        }

        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].code, "E1109");
    }

    #[test]
    fn test_no_chain_passes() {
        let code = r#"
            fn example() {
                let x = 42;
                let y = x + 1;
                println!("{}", y);
            }
        "#;

        let syntax = syn::parse_file(code).unwrap();
        let checker = E1109ExcessiveChaining::default();

        let mut violations = Vec::new();
        for item in &syntax.items {
            violations.extend(checker.check_item(item, "test.rs").unwrap());
        }

        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_custom_threshold() {
        let code = r#"
            fn example() {
                let result = vec![1, 2, 3].iter().map(|x| x * 2).collect::<Vec<_>>();
            }
        "#;

        let syntax = syn::parse_file(code).unwrap();
        let mut checker = E1109ExcessiveChaining::default();
        checker.config.max_chain_length = 2;

        let mut violations = Vec::new();
        for item in &syntax.items {
            violations.extend(checker.check_item(item, "test.rs").unwrap());
        }

        assert_eq!(violations.len(), 1);
    }
}
