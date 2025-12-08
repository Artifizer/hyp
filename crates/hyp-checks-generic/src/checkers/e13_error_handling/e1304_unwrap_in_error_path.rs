//! E1304: Using unwrap() in error paths
//!
//! Detects unwrap() calls inside error handling code (match arms for Err, catch blocks),
//! which can cause double-panics and make error handling unreliable.

use crate::{checker::Checker, define_checker, violation::Violation};

use syn::visit::Visit;

define_checker! {
    /// Checker for E1304: Using unwrap() in error paths
    E1304UnwrapInErrorPath,
    code = "E1304",
    name = "Using unwrap() in error handling code",
    suggestions = "Use ? operator or proper error handling instead of unwrap() in error paths",
    target_items = [Function],
    config_entry_name = "e1304_unwrap_in_error_path",
    config = E1304Config {
        enabled: bool = true,
        severity: crate::config::SeverityLevel = crate::config::SeverityLevel::High,
        categories: Vec<crate::config::CheckerCategory> = vec![crate::config::CheckerCategory::Operations],
    },
    check_item(self, item, file_path) {
        let mut visitor = UnwrapInErrorVisitor {
            violations: Vec::new(),
            file_path,
            checker: self,
            in_error_path: false,
        };
        visitor.visit_item(item);
        Ok(visitor.violations)
    }
}

struct UnwrapInErrorVisitor<'a> {
    violations: Vec<Violation>,
    file_path: &'a str,
    checker: &'a E1304UnwrapInErrorPath,
    in_error_path: bool,
}

impl<'a> Visit<'a> for UnwrapInErrorVisitor<'a> {
    fn visit_arm(&mut self, node: &'a syn::Arm) {
        // Check if this arm is matching an Err variant
        let is_err_arm = match &node.pat {
            syn::Pat::TupleStruct(ts) => {
                let path_str = ts
                    .path
                    .segments
                    .iter()
                    .map(|s| s.ident.to_string())
                    .collect::<Vec<_>>()
                    .join("::");
                path_str == "Err" || path_str.ends_with("::Err")
            }
            syn::Pat::Ident(ident) => {
                let name = ident.ident.to_string();
                name == "Err" || name.starts_with("err") || name.starts_with("Err")
            }
            _ => false,
        };

        let was_in_error_path = self.in_error_path;
        if is_err_arm {
            self.in_error_path = true;
        }

        syn::visit::visit_arm(self, node);

        self.in_error_path = was_in_error_path;
    }

    fn visit_expr_method_call(&mut self, node: &'a syn::ExprMethodCall) {
        if self.in_error_path {
            let method_name = node.method.to_string();
            if matches!(method_name.as_str(), "unwrap" | "expect") {
                use syn::spanned::Spanned;
                let start = node.span().start();
                self.violations.push(
                    Violation::new(
                        self.checker.code(),
                        self.checker.name(),
                        self.checker.severity().into(),
                        format!(
                            "Using {}() in error handling code can cause double-panics.",
                            method_name
                        ),
                        self.file_path,
                        start.line,
                        start.column + 1,
                    )
                    .with_suggestion(self.checker.suggestions()),
                );
            }
        }

        syn::visit::visit_expr_method_call(self, node);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::checker::Checker;

    #[test]
    fn test_detects_unwrap_in_err_arm() {
        let code = r#"
            fn example(r: Result<String, std::io::Error>) {
                match r {
                    Ok(s) => println!("{}", s),
                    Err(_) => {
                        let fallback = std::fs::read_to_string("fallback.txt").unwrap();
                        println!("{}", fallback);
                    }
                }
            }
        "#;

        let syntax = syn::parse_file(code).unwrap();
        let checker = E1304UnwrapInErrorPath::default();

        let mut violations = Vec::new();
        for item in &syntax.items {
            violations.extend(checker.check_item(item, "test.rs").unwrap());
        }

        assert_eq!(violations.len(), 1);
    }

    #[test]
    fn test_unwrap_in_ok_arm_passes() {
        let code = r#"
            fn example(r: Result<String, std::io::Error>) {
                match r {
                    Ok(s) => {
                        let parsed: i32 = s.parse().unwrap();
                        println!("{}", parsed);
                    }
                    Err(e) => eprintln!("Error: {}", e),
                }
            }
        "#;

        let syntax = syn::parse_file(code).unwrap();
        let checker = E1304UnwrapInErrorPath::default();

        let mut violations = Vec::new();
        for item in &syntax.items {
            violations.extend(checker.check_item(item, "test.rs").unwrap());
        }

        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_proper_error_handling_passes() {
        let code = r#"
            fn example(r: Result<String, std::io::Error>) -> Result<(), std::io::Error> {
                match r {
                    Ok(s) => println!("{}", s),
                    Err(e) => {
                        let fallback = std::fs::read_to_string("fallback.txt")?;
                        println!("{}", fallback);
                    }
                }
                Ok(())
            }
        "#;

        let syntax = syn::parse_file(code).unwrap();
        let checker = E1304UnwrapInErrorPath::default();

        let mut violations = Vec::new();
        for item in &syntax.items {
            violations.extend(checker.check_item(item, "test.rs").unwrap());
        }

        assert_eq!(violations.len(), 0);
    }
}
