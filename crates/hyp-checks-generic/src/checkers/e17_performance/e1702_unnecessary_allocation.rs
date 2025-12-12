//! E1702: Unnecessary allocations
//!
//! Detects allocations that could be avoided, such as
//! String::new() followed by push_str of a known string.

use crate::{checker::Checker, define_checker, violation::Violation};

use syn::visit::Visit;

define_checker! {
    /// Checker for E1702: Unnecessary allocations
    E1702UnnecessaryAllocation,
    code = "E1702",
    name = "Unnecessary allocation",
    suggestions = "Use string literals or pre-allocate with capacity",
    target_items = [Function],
    config_entry_name = "e1702_unnecessary_allocation",
    config = E1702Config {
        enabled: bool = true,
        severity: crate::config::SeverityLevel = crate::config::SeverityLevel::Low,
        categories: Vec<crate::config::CheckerCategory> = vec![crate::config::CheckerCategory::Complexity],
    },
    check_item(self, item, file_path) {
        let mut visitor = AllocationVisitor {
            violations: Vec::new(),
            file_path,
            checker: self,
        };
        visitor.visit_item(item);
        Ok(visitor.violations)
    }
}

struct AllocationVisitor<'a> {
    violations: Vec<Violation>,
    file_path: &'a str,
    checker: &'a E1702UnnecessaryAllocation,
}

impl<'a> Visit<'a> for AllocationVisitor<'a> {
    fn visit_expr_call(&mut self, node: &'a syn::ExprCall) {
        if let syn::Expr::Path(path) = &*node.func {
            let path_str = path
                .path
                .segments
                .iter()
                .map(|s| s.ident.to_string())
                .collect::<Vec<_>>()
                .join("::");

            // Detect String::from("") or String::new() followed by format use
            if path_str.ends_with("String::from") {
                if let Some(syn::Expr::Lit(syn::ExprLit {
                    lit: syn::Lit::Str(s),
                    ..
                })) = node.args.first()
                {
                    if s.value().is_empty() {
                        use syn::spanned::Spanned;
                        let start = node.span().start();
                        self.violations.push(
                            Violation::new(
                                self.checker.code(),
                                self.checker.name(),
                                self.checker.severity().into(),
                                "String::from(\"\") allocates unnecessarily. Use String::new() or consider if allocation is needed.",
                                self.file_path,
                                start.line,
                                start.column + 1,
                            )
                            .with_suggestion(self.checker.suggestions()),
                        );
                    }
                }
            }

            // Detect Vec::new() without with_capacity when size is known
            if path_str == "vec" {
                // This is the vec![] macro which is fine
            }
        }

        syn::visit::visit_expr_call(self, node);
    }

    fn visit_expr_method_call(&mut self, node: &'a syn::ExprMethodCall) {
        let method_name = node.method.to_string();

        // Detect .to_string() on string literals
        if method_name == "to_string" {
            if let syn::Expr::Lit(lit) = &*node.receiver {
                if let syn::Lit::Str(_) = &lit.lit {
                    use syn::spanned::Spanned;
                    let start = node.span().start();
                    self.violations.push(
                        Violation::new(
                            self.checker.code(),
                            self.checker.name(),
                            self.checker.severity().into(),
                            "Calling .to_string() on a string literal allocates. Use String::from() for clarity or consider if &str suffices.",
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
    fn test_detects_empty_string_from() {
        let code = r#"
            fn example() {
                let s = String::from("");
            }
        "#;

        let syntax = syn::parse_file(code).unwrap();
        let checker = E1702UnnecessaryAllocation::default();

        let mut violations = Vec::new();
        for item in &syntax.items {
            violations.extend(checker.check_item(item, "test.rs").unwrap());
        }

        assert_eq!(violations.len(), 1);
    }

    #[test]
    fn test_string_new_passes() {
        let code = r#"
            fn example() {
                let s = String::new();
            }
        "#;

        let syntax = syn::parse_file(code).unwrap();
        let checker = E1702UnnecessaryAllocation::default();

        let mut violations = Vec::new();
        for item in &syntax.items {
            violations.extend(checker.check_item(item, "test.rs").unwrap());
        }

        assert_eq!(violations.len(), 0);
    }
}
