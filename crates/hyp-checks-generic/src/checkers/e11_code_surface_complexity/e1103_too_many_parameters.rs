//! E1103: Too many function parameters
//!
//! Detects functions with too many parameters, which makes them hard to call
//! correctly and indicates the function may be doing too much.

use crate::{define_checker, violation::Violation};

define_checker! {
    /// Checker for E1103: Too many function parameters
    E1103TooManyParameters,
    code = "E1103",
    name = "Too many function parameters",
    suggestions = "Group related parameters into a struct. Consider the builder pattern for complex construction.",
    target_items = [Function],
    config_entry_name = "e1103_too_many_parameters",
    /// Configuration for E1103: Too many parameters checker
    config = E1103Config {
        /// Whether this checker is enabled
        enabled: bool = true,
        /// Severity level, defaults to Low
        severity: crate::config::SeverityLevel = crate::config::SeverityLevel::Low,
        /// Categories this checker belongs to
        categories: Vec<crate::config::CheckerCategory> = vec![crate::config::CheckerCategory::Complexity],
        /// Maximum allowed parameters
        max_parameters: usize = 5,
    },
    check_item(self, item, file_path) {
        let mut violations = Vec::new();

        if let syn::Item::Fn(func) = item {
            let param_count = func.sig.inputs.len();

            if param_count > self.config.max_parameters {
                let func_name = func.sig.ident.to_string();
                let span = func.sig.ident.span().start();

                violations.push(
                    Violation::new(
                        self.code(),
                        self.name(),
                        self.severity().into(),
                        format!(
                            "Function '{}' has {} parameters, exceeding the limit of {}. Too many parameters make functions hard to use correctly.",
                            func_name, param_count, self.config.max_parameters
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::checker::Checker;

    #[test]
    fn test_few_parameters_passes() {
        let code = r#"
            fn example(a: i32, b: i32, c: i32) -> i32 {
                a + b + c
            }
        "#;

        let syntax = syn::parse_file(code).unwrap();
        let checker = E1103TooManyParameters::default();

        let mut violations = Vec::new();
        for item in &syntax.items {
            violations.extend(checker.check_item(item, "test.rs").unwrap());
        }

        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_detects_too_many_parameters() {
        let code = r#"
            fn example(a: i32, b: i32, c: i32, d: i32, e: i32, f: i32, g: i32) -> i32 {
                a + b + c + d + e + f + g
            }
        "#;

        let syntax = syn::parse_file(code).unwrap();
        let checker = E1103TooManyParameters::default();

        let mut violations = Vec::new();
        for item in &syntax.items {
            violations.extend(checker.check_item(item, "test.rs").unwrap());
        }

        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].code, "E1103");
        assert!(violations[0].message.contains("7 parameters"));
    }

    #[test]
    fn test_method_with_self() {
        // Note: Methods inside impl blocks are not directly checked by target_items = [Function]
        // This test documents that behavior - methods need the Impl target type
        let code = r#"
            fn method(self_: i32, a: i32, b: i32, c: i32, d: i32, e: i32, f: i32) -> i32 {
                self_ + a + b + c + d + e + f
            }
        "#;

        let syntax = syn::parse_file(code).unwrap();
        let checker = E1103TooManyParameters::default();

        let mut violations = Vec::new();
        for item in &syntax.items {
            violations.extend(checker.check_item(item, "test.rs").unwrap());
        }

        // 7 params
        assert_eq!(violations.len(), 1);
    }

    #[test]
    fn test_custom_threshold() {
        let code = r#"
            fn example(a: i32, b: i32, c: i32) -> i32 {
                a + b + c
            }
        "#;

        let syntax = syn::parse_file(code).unwrap();
        let mut checker = E1103TooManyParameters::default();
        checker.config.max_parameters = 2;

        let mut violations = Vec::new();
        for item in &syntax.items {
            violations.extend(checker.check_item(item, "test.rs").unwrap());
        }

        assert_eq!(violations.len(), 1);
    }
}
