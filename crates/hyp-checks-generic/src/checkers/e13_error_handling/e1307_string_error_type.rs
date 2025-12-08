//! E1307: Using String for error types
//!
//! Detects functions that return Result<T, String> instead of proper error types.

use crate::{define_checker, violation::Violation};

define_checker! {
    /// Checker for E1307: Using String for error types
    E1307StringErrorType,
    code = "E1307",
    name = "Using String for error types",
    suggestions = "Define a custom error type or use thiserror/anyhow for better error handling",
    target_items = [Function],
    config_entry_name = "e1307_string_error_type",
    config = E1307Config {
        enabled: bool = true,
        severity: crate::config::SeverityLevel = crate::config::SeverityLevel::Medium,
        categories: Vec<crate::config::CheckerCategory> = vec![crate::config::CheckerCategory::Operations],
    },
    check_item(self, item, file_path) {
        let mut violations = Vec::new();

        if let syn::Item::Fn(func) = item {
            if let syn::ReturnType::Type(_, ty) = &func.sig.output {
                if is_result_with_string_error(ty) {
                    let start = func.sig.ident.span().start();
                    violations.push(
                        Violation::new(
                            self.code(),
                            self.name(),
                            self.severity().into(),
                            format!(
                                "Function '{}' returns Result<_, String>. Use a proper error type for better error handling.",
                                func.sig.ident
                            ),
                            file_path,
                            start.line,
                            start.column + 1,
                        )
                        .with_suggestion(self.suggestions()),
                    );
                }
            }
        }

        Ok(violations)
    }
}

fn is_result_with_string_error(ty: &syn::Type) -> bool {
    if let syn::Type::Path(type_path) = ty {
        if let Some(segment) = type_path.path.segments.last() {
            if segment.ident == "Result" {
                if let syn::PathArguments::AngleBracketed(args) = &segment.arguments {
                    // Check if the second type argument is String
                    if args.args.len() >= 2 {
                        if let Some(syn::GenericArgument::Type(err_ty)) = args.args.iter().nth(1) {
                            return is_string_type(err_ty);
                        }
                    }
                }
            }
        }
    }
    false
}

fn is_string_type(ty: &syn::Type) -> bool {
    if let syn::Type::Path(type_path) = ty {
        if let Some(segment) = type_path.path.segments.last() {
            return segment.ident == "String";
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::checker::Checker;

    #[test]
    fn test_detects_string_error() {
        let code = r#"
            fn example() -> Result<i32, String> {
                Ok(42)
            }
        "#;

        let syntax = syn::parse_file(code).unwrap();
        let checker = E1307StringErrorType::default();

        let mut violations = Vec::new();
        for item in &syntax.items {
            violations.extend(checker.check_item(item, "test.rs").unwrap());
        }

        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].code, "E1307");
    }

    #[test]
    fn test_proper_error_type_passes() {
        let code = r#"
            fn example() -> Result<i32, MyError> {
                Ok(42)
            }
        "#;

        let syntax = syn::parse_file(code).unwrap();
        let checker = E1307StringErrorType::default();

        let mut violations = Vec::new();
        for item in &syntax.items {
            violations.extend(checker.check_item(item, "test.rs").unwrap());
        }

        assert_eq!(violations.len(), 0);
    }
}
