//! E1607: Forgetting to drop
//!
//! Detects use of mem::forget which prevents destructors from running,
//! potentially leaking resources.

use crate::{checker::Checker, define_checker, violation::Violation};

use syn::visit::Visit;

define_checker! {
    /// Checker for E1607: Forgetting to drop
    E1607ForgetDrop,
    code = "E1607",
    name = "Using mem::forget to leak resources",
    suggestions = "Let values drop normally, or use ManuallyDrop for explicit control over destruction",
    target_items = [Function],
    config_entry_name = "e1607_forget_drop",
    config = E1607Config {
        enabled: bool = true,
        severity: crate::config::SeverityLevel = crate::config::SeverityLevel::Medium,
        categories: Vec<crate::config::CheckerCategory> = vec![crate::config::CheckerCategory::Complexity],
    },
    check_item(self, item, file_path) {
        let mut visitor = ForgetVisitor {
            violations: Vec::new(),
            file_path,
            checker: self,
        };
        visitor.visit_item(item);
        Ok(visitor.violations)
    }
}

struct ForgetVisitor<'a> {
    violations: Vec<Violation>,
    file_path: &'a str,
    checker: &'a E1607ForgetDrop,
}

impl<'a> Visit<'a> for ForgetVisitor<'a> {
    fn visit_expr_call(&mut self, node: &'a syn::ExprCall) {
        if let syn::Expr::Path(path) = &*node.func {
            let path_str = path
                .path
                .segments
                .iter()
                .map(|s| s.ident.to_string())
                .collect::<Vec<_>>()
                .join("::");

            // Check for mem::forget or std::mem::forget
            if path_str.ends_with("forget") && (path_str.contains("mem") || path_str == "forget") {
                use syn::spanned::Spanned;
                let start = node.span().start();
                self.violations.push(
                    Violation::new(
                        self.checker.code(),
                        self.checker.name(),
                        self.checker.severity().into(),
                        "Using mem::forget prevents destructor from running, potentially leaking resources.",
                        self.file_path,
                        start.line,
                        start.column + 1,
                    )
                    .with_suggestion(self.checker.suggestions()),
                );
            }
        }

        syn::visit::visit_expr_call(self, node);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::checker::Checker;

    #[test]
    fn test_detects_mem_forget() {
        let code = r#"
            fn example() {
                let data = vec![1, 2, 3];
                std::mem::forget(data);
            }
        "#;

        let syntax = syn::parse_file(code).unwrap();
        let checker = E1607ForgetDrop::default();

        let mut violations = Vec::new();
        for item in &syntax.items {
            violations.extend(checker.check_item(item, "test.rs").unwrap());
        }

        assert_eq!(violations.len(), 1);
    }

    #[test]
    fn test_normal_drop_passes() {
        let code = r#"
            fn example() {
                let data = vec![1, 2, 3];
                // data drops naturally at end of scope
            }
        "#;

        let syntax = syn::parse_file(code).unwrap();
        let checker = E1607ForgetDrop::default();

        let mut violations = Vec::new();
        for item in &syntax.items {
            violations.extend(checker.check_item(item, "test.rs").unwrap());
        }

        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_explicit_drop_passes() {
        let code = r#"
            fn example() {
                let data = vec![1, 2, 3];
                drop(data);
            }
        "#;

        let syntax = syn::parse_file(code).unwrap();
        let checker = E1607ForgetDrop::default();

        let mut violations = Vec::new();
        for item in &syntax.items {
            violations.extend(checker.check_item(item, "test.rs").unwrap());
        }

        assert_eq!(violations.len(), 0);
    }
}
