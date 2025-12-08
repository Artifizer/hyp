//! E1101: High cyclomatic complexity
//!
//! Detects functions with high cyclomatic complexity, which indicates too many
//! decision paths making the code hard to understand and test.

use crate::{define_checker, violation::Violation};

use syn::visit::Visit;

define_checker! {
    /// Checker for E1101: High cyclomatic complexity
    E1101HighCyclomaticComplexity,
    code = "E1101",
    name = "High cyclomatic complexity",
    suggestions = "Break down the function into smaller, focused functions. Extract conditional logic into helper methods.",
    target_items = [Function],
    config_entry_name = "e1101_high_cyclomatic_complexity",
    /// Configuration for E1101: High cyclomatic complexity checker
    config = E1101Config {
        /// Whether this checker is enabled
        enabled: bool = true,
        /// Severity level, defaults to Medium
        severity: crate::config::SeverityLevel = crate::config::SeverityLevel::Medium,
        /// Categories this checker belongs to
        categories: Vec<crate::config::CheckerCategory> = vec![crate::config::CheckerCategory::Complexity],
        /// Maximum allowed cyclomatic complexity
        max_complexity: usize = 25,
    },
    check_item(self, item, file_path) {
        let mut violations = Vec::new();

        if let syn::Item::Fn(func) = item {
            let mut counter = ComplexityCounter { complexity: 1 }; // Start at 1
            counter.visit_item_fn(func);

            if counter.complexity > self.config.max_complexity {
                let func_name = func.sig.ident.to_string();
                let span = func.sig.ident.span().start();

                violations.push(
                    Violation::new(
                        self.code(),
                        self.name(),
                        self.severity().into(),
                        format!(
                            "Function '{}' has cyclomatic complexity of {}, exceeding the limit of {}. High complexity makes code hard to understand and test.",
                            func_name, counter.complexity, self.config.max_complexity
                        ),
                        file_path,
                        span.line,
                        span.column + 1,
                    )
                    .with_suggestion(self.suggestions()),
                );
            }
        }

        Ok(violations)
    }
}

/// Counts cyclomatic complexity by counting decision points
struct ComplexityCounter {
    complexity: usize,
}

impl<'ast> Visit<'ast> for ComplexityCounter {
    fn visit_expr_if(&mut self, node: &'ast syn::ExprIf) {
        self.complexity += 1;
        syn::visit::visit_expr_if(self, node);
    }

    fn visit_expr_match(&mut self, node: &'ast syn::ExprMatch) {
        // Each arm (except the first) adds to complexity
        if node.arms.len() > 1 {
            self.complexity += node.arms.len() - 1;
        }
        syn::visit::visit_expr_match(self, node);
    }

    fn visit_expr_while(&mut self, node: &'ast syn::ExprWhile) {
        self.complexity += 1;
        syn::visit::visit_expr_while(self, node);
    }

    fn visit_expr_for_loop(&mut self, node: &'ast syn::ExprForLoop) {
        self.complexity += 1;
        syn::visit::visit_expr_for_loop(self, node);
    }

    fn visit_expr_loop(&mut self, node: &'ast syn::ExprLoop) {
        self.complexity += 1;
        syn::visit::visit_expr_loop(self, node);
    }

    fn visit_expr_binary(&mut self, node: &'ast syn::ExprBinary) {
        // Count && and || as they represent short-circuit decision points
        match node.op {
            syn::BinOp::And(_) | syn::BinOp::Or(_) => {
                self.complexity += 1;
            }
            _ => {}
        }
        syn::visit::visit_expr_binary(self, node);
    }

    fn visit_expr_try(&mut self, node: &'ast syn::ExprTry) {
        // ? operator is a decision point (Ok vs Err)
        self.complexity += 1;
        syn::visit::visit_expr_try(self, node);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::checker::Checker;

    #[test]
    fn test_simple_function_passes() {
        let code = r#"
            fn simple() {
                let x = 1;
                let y = 2;
                println!("{}", x + y);
            }
        "#;

        let syntax = syn::parse_file(code).unwrap();
        let checker = E1101HighCyclomaticComplexity::default();

        let mut violations = Vec::new();
        for item in &syntax.items {
            violations.extend(checker.check_item(item, "test.rs").unwrap());
        }

        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_detects_high_complexity() {
        let code = r#"
            fn complex(x: i32) -> i32 {
                if x > 0 {
                    if x > 10 {
                        if x > 100 {
                            return 3;
                        }
                        return 2;
                    }
                    return 1;
                } else if x < 0 {
                    if x < -10 {
                        if x < -100 {
                            return -3;
                        }
                        return -2;
                    }
                    return -1;
                }

                match x {
                    0 => 0,
                    1 => 1,
                    2 => 2,
                    _ => 99,
                }
            }
        "#;

        let syntax = syn::parse_file(code).unwrap();
        let mut checker = E1101HighCyclomaticComplexity::default();
        checker.config.max_complexity = 5; // Lower threshold for test

        let mut violations = Vec::new();
        for item in &syntax.items {
            violations.extend(checker.check_item(item, "test.rs").unwrap());
        }

        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].code, "E1101");
    }

    #[test]
    fn test_counts_loops() {
        let code = r#"
            fn loopy() {
                for i in 0..10 {
                    while i > 5 {
                        loop {
                            break;
                        }
                    }
                }
            }
        "#;

        let syntax = syn::parse_file(code).unwrap();
        let mut checker = E1101HighCyclomaticComplexity::default();
        checker.config.max_complexity = 2;

        let mut violations = Vec::new();
        for item in &syntax.items {
            violations.extend(checker.check_item(item, "test.rs").unwrap());
        }

        assert_eq!(violations.len(), 1);
    }
}
