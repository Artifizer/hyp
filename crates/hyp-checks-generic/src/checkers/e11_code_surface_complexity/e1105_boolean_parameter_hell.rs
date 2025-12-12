//! E1105: Boolean parameter hell
//!
//! Detects functions with multiple boolean parameters, which makes call sites
//! confusing (e.g., `do_thing(true, false, true)` - what do these mean?).

use crate::{define_checker, violation::Violation};

define_checker! {
    /// Checker for E1105: Boolean parameter hell
    E1105BooleanParameterHell,
    code = "E1105",
    name = "Boolean parameter hell",
    suggestions = "Replace boolean parameters with enums or a config struct for clarity",
    target_items = [Function],
    config_entry_name = "e1105_boolean_parameter_hell",
    /// Configuration for E1105: Boolean parameter hell checker
    config = E1105Config {
        /// Whether this checker is enabled
        enabled: bool = true,
        /// Severity level, defaults to Low
        severity: crate::config::SeverityLevel = crate::config::SeverityLevel::Low,
        /// Categories this checker belongs to
        categories: Vec<crate::config::CheckerCategory> = vec![crate::config::CheckerCategory::Complexity],
        /// Maximum allowed boolean parameters
        max_bool_params: usize = 1,
    },
    check_item(self, item, file_path) {
        let mut violations = Vec::new();

        if let syn::Item::Fn(func) = item {
            let bool_count = func.sig.inputs.iter().filter(|arg| {
                if let syn::FnArg::Typed(pat_type) = arg {
                    is_bool_type(&pat_type.ty)
                } else {
                    false
                }
            }).count();

            if bool_count > self.config.max_bool_params {
                let func_name = func.sig.ident.to_string();
                let span = func.sig.ident.span().start();

                violations.push(
                    Violation::new(
                        self.code(),
                        self.name(),
                        self.severity().into(),
                        format!(
                            "Function '{}' has {} boolean parameters. Multiple booleans make call sites confusing.",
                            func_name, bool_count
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

/// Check if a type is a boolean
fn is_bool_type(ty: &syn::Type) -> bool {
    if let syn::Type::Path(type_path) = ty {
        if type_path.qself.is_none() {
            if let Some(segment) = type_path.path.segments.last() {
                return segment.ident == "bool";
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
    fn test_single_bool_passes() {
        let code = r#"
            fn example(enabled: bool) {
                if enabled {
                    println!("enabled");
                }
            }
        "#;

        let syntax = syn::parse_file(code).unwrap();
        let checker = E1105BooleanParameterHell::default();

        let mut violations = Vec::new();
        for item in &syntax.items {
            violations.extend(checker.check_item(item, "test.rs").unwrap());
        }

        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_detects_multiple_bools() {
        let code = r#"
            fn confusing(enable_a: bool, enable_b: bool, enable_c: bool) {
                // What does confusing(true, false, true) mean?
            }
        "#;

        let syntax = syn::parse_file(code).unwrap();
        let checker = E1105BooleanParameterHell::default();

        let mut violations = Vec::new();
        for item in &syntax.items {
            violations.extend(checker.check_item(item, "test.rs").unwrap());
        }

        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].code, "E1105");
        assert!(violations[0].message.contains("3 boolean"));
    }

    #[test]
    fn test_no_bool_passes() {
        let code = r#"
            fn example(x: i32, y: i32) -> i32 {
                x + y
            }
        "#;

        let syntax = syn::parse_file(code).unwrap();
        let checker = E1105BooleanParameterHell::default();

        let mut violations = Vec::new();
        for item in &syntax.items {
            violations.extend(checker.check_item(item, "test.rs").unwrap());
        }

        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_custom_threshold() {
        let code = r#"
            fn example(a: bool, b: bool) {
            }
        "#;

        let syntax = syn::parse_file(code).unwrap();
        let mut checker = E1105BooleanParameterHell::default();
        checker.config.max_bool_params = 2;

        let mut violations = Vec::new();
        for item in &syntax.items {
            violations.extend(checker.check_item(item, "test.rs").unwrap());
        }

        assert_eq!(violations.len(), 0);
    }
}
