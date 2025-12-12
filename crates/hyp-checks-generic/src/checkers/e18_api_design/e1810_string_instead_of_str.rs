//! E1810: String instead of &str
//!
//! Detects function parameters that take String where &str would suffice,
//! forcing unnecessary allocations by callers.

use crate::{define_checker, violation::Violation};

use syn::spanned::Spanned;

define_checker! {
    /// Checker for E1810: String instead of &str
    E1810StringInsteadOfStr,
    code = "E1810",
    name = "String instead of &str",
    suggestions = "Use &str for parameters that don't need ownership, or impl AsRef<str> for flexibility",
    target_items = [Function],
    config_entry_name = "e1810_string_instead_of_str",
    config = E1810Config {
        enabled: bool = true,
        severity: crate::config::SeverityLevel = crate::config::SeverityLevel::Low,
        categories: Vec<crate::config::CheckerCategory> = vec![crate::config::CheckerCategory::Complexity],
    },
    check_item(self, item, file_path) {
        let mut violations = Vec::new();

        if let syn::Item::Fn(func) = item {
            // Only check public functions
            if !matches!(func.vis, syn::Visibility::Public(_)) {
                return Ok(violations);
            }

            for input in &func.sig.inputs {
                if let syn::FnArg::Typed(pat_type) = input {
                    if is_owned_string(&pat_type.ty) {
                        let param_name = if let syn::Pat::Ident(ident) = &*pat_type.pat {
                            ident.ident.to_string()
                        } else {
                            "parameter".to_string()
                        };

                        let start = pat_type.span().start();
                        violations.push(
                            Violation::new(
                                self.code(),
                                self.name(),
                                self.severity().into(),
                                format!(
                                    "Parameter '{}' takes String. Consider &str if ownership isn't needed.",
                                    param_name
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
        }

        Ok(violations)
    }
}

fn is_owned_string(ty: &syn::Type) -> bool {
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
    fn test_detects_string_parameter() {
        let code = r#"
            pub fn greet(name: String) {
                println!("Hello, {}!", name);
            }
        "#;

        let syntax = syn::parse_file(code).unwrap();
        let checker = E1810StringInsteadOfStr::default();

        let mut violations = Vec::new();
        for item in &syntax.items {
            violations.extend(checker.check_item(item, "test.rs").unwrap());
        }

        assert_eq!(violations.len(), 1);
    }

    #[test]
    fn test_str_reference_passes() {
        let code = r#"
            pub fn greet(name: &str) {
                println!("Hello, {}!", name);
            }
        "#;

        let syntax = syn::parse_file(code).unwrap();
        let checker = E1810StringInsteadOfStr::default();

        let mut violations = Vec::new();
        for item in &syntax.items {
            violations.extend(checker.check_item(item, "test.rs").unwrap());
        }

        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_private_function_passes() {
        let code = r#"
            fn greet(name: String) {
                println!("Hello, {}!", name);
            }
        "#;

        let syntax = syn::parse_file(code).unwrap();
        let checker = E1810StringInsteadOfStr::default();

        let mut violations = Vec::new();
        for item in &syntax.items {
            violations.extend(checker.check_item(item, "test.rs").unwrap());
        }

        assert_eq!(violations.len(), 0);
    }
}
