//! E1704: Unnecessary collect()
//!
//! Detects .collect() calls that are immediately iterated over again,
//! which could be avoided by using iterator adaptors directly.

use crate::{checker::Checker, define_checker, violation::Violation};

use syn::visit::Visit;

define_checker! {
    /// Checker for E1704: Unnecessary collect()
    E1704UnnecessaryCollect,
    code = "E1704",
    name = "Unnecessary collect()",
    suggestions = "Chain iterator methods directly instead of collecting to a Vec first",
    target_items = [Function],
    config_entry_name = "e1704_unnecessary_collect",
    config = E1704Config {
        enabled: bool = true,
        severity: crate::config::SeverityLevel = crate::config::SeverityLevel::Low,
        categories: Vec<crate::config::CheckerCategory> = vec![crate::config::CheckerCategory::Complexity],
    },
    check_item(self, item, file_path) {
        let mut visitor = CollectVisitor {
            violations: Vec::new(),
            file_path,
            checker: self,
        };
        visitor.visit_item(item);
        Ok(visitor.violations)
    }
}

struct CollectVisitor<'a> {
    violations: Vec<Violation>,
    file_path: &'a str,
    checker: &'a E1704UnnecessaryCollect,
}

impl<'a> Visit<'a> for CollectVisitor<'a> {
    fn visit_expr_method_call(&mut self, node: &'a syn::ExprMethodCall) {
        let method_name = node.method.to_string();

        // Check for .collect().iter() or .collect().into_iter()
        if method_name == "iter" || method_name == "into_iter" {
            if let syn::Expr::MethodCall(inner) = &*node.receiver {
                if inner.method == "collect" {
                    use syn::spanned::Spanned;
                    let start = node.span().start();
                    self.violations.push(
                        Violation::new(
                            self.checker.code(),
                            self.checker.name(),
                            self.checker.severity().into(),
                            "Collecting and immediately iterating is wasteful. Use iterator adaptors directly.",
                            self.file_path,
                            start.line,
                            start.column + 1,
                        )
                        .with_suggestion(self.checker.suggestions()),
                    );
                }
            }
        }

        // Check for .collect::<Vec<_>>().len()
        if method_name == "len" || method_name == "is_empty" {
            if let syn::Expr::MethodCall(inner) = &*node.receiver {
                if inner.method == "collect" {
                    use syn::spanned::Spanned;
                    let start = node.span().start();
                    self.violations.push(
                        Violation::new(
                            self.checker.code(),
                            self.checker.name(),
                            self.checker.severity().into(),
                            format!("Collecting just to call .{}() is wasteful. Use .count() directly on the iterator.", method_name),
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
    fn test_detects_collect_iter() {
        let code = r#"
            fn example() {
                let v: Vec<i32> = (0..10).map(|x| x * 2).collect::<Vec<_>>().into_iter().filter(|x| *x > 5).collect();
            }
        "#;

        let syntax = syn::parse_file(code).unwrap();
        let checker = E1704UnnecessaryCollect::default();

        let mut violations = Vec::new();
        for item in &syntax.items {
            violations.extend(checker.check_item(item, "test.rs").unwrap());
        }

        assert_eq!(violations.len(), 1);
    }

    #[test]
    fn test_collect_len() {
        let code = r#"
            fn example() {
                let count = (0..10).filter(|x| *x > 5).collect::<Vec<_>>().len();
            }
        "#;

        let syntax = syn::parse_file(code).unwrap();
        let checker = E1704UnnecessaryCollect::default();

        let mut violations = Vec::new();
        for item in &syntax.items {
            violations.extend(checker.check_item(item, "test.rs").unwrap());
        }

        assert_eq!(violations.len(), 1);
    }

    #[test]
    fn test_proper_collect_passes() {
        let code = r#"
            fn example() -> Vec<i32> {
                (0..10).filter(|x| *x > 5).collect()
            }
        "#;

        let syntax = syn::parse_file(code).unwrap();
        let checker = E1704UnnecessaryCollect::default();

        let mut violations = Vec::new();
        for item in &syntax.items {
            violations.extend(checker.check_item(item, "test.rs").unwrap());
        }

        assert_eq!(violations.len(), 0);
    }
}
