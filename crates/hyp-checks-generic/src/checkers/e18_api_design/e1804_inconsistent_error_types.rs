//! E1804: Inconsistent error types
//!
//! Detects modules where functions return different error types,
//! suggesting a unified error type for the module.

use crate::{define_checker, violation::Violation};

use syn::visit::Visit;

define_checker! {
    /// Checker for E1804: Inconsistent error types
    E1804InconsistentErrorTypes,
    code = "E1804",
    name = "Inconsistent error types",
    suggestions = "Define a unified error type for the module using thiserror or a custom enum",
    target_items = [Module],
    config_entry_name = "e1804_inconsistent_error_types",
    config = E1804Config {
        enabled: bool = true,
        severity: crate::config::SeverityLevel = crate::config::SeverityLevel::Medium,
        categories: Vec<crate::config::CheckerCategory> = vec![crate::config::CheckerCategory::Complexity],
    },
    check_item(self, item, file_path) {
        let mut visitor = ErrorTypeVisitor {
            error_types: Vec::new(),
        };

        if let syn::Item::Mod(module) = item {
            if let Some((_, items)) = &module.content {
                for item in items {
                    visitor.visit_item(item);
                }

                // Check if there are multiple different error types
                let unique_errors: std::collections::HashSet<_> =
                    visitor.error_types.iter().collect();

                if unique_errors.len() > 2 {
                    let start = module.ident.span().start();
                    return Ok(vec![
                        Violation::new(
                            self.code(),
                            self.name(),
                            self.severity().into(),
                            format!(
                                "Module '{}' uses {} different error types: {:?}. Consider a unified error type.",
                                module.ident,
                                unique_errors.len(),
                                unique_errors.iter().take(3).collect::<Vec<_>>()
                            ),
                            file_path,
                            start.line,
                            start.column + 1,
                        )
                        .with_suggestion(self.suggestions()),
                    ]);
                }
            }
        }

        Ok(Vec::new())
    }
}

struct ErrorTypeVisitor {
    error_types: Vec<String>,
}

impl<'ast> Visit<'ast> for ErrorTypeVisitor {
    fn visit_item_fn(&mut self, node: &'ast syn::ItemFn) {
        if let syn::ReturnType::Type(_, ty) = &node.sig.output {
            if let Some(err_type) = extract_result_error_type(ty) {
                self.error_types.push(err_type);
            }
        }
        syn::visit::visit_item_fn(self, node);
    }
}

fn extract_result_error_type(ty: &syn::Type) -> Option<String> {
    if let syn::Type::Path(type_path) = ty {
        if let Some(segment) = type_path.path.segments.last() {
            if segment.ident == "Result" {
                if let syn::PathArguments::AngleBracketed(args) = &segment.arguments {
                    if args.args.len() >= 2 {
                        if let Some(syn::GenericArgument::Type(err_ty)) = args.args.iter().nth(1) {
                            return Some(type_to_string(err_ty));
                        }
                    }
                }
            }
        }
    }
    None
}

fn type_to_string(ty: &syn::Type) -> String {
    if let syn::Type::Path(type_path) = ty {
        type_path
            .path
            .segments
            .iter()
            .map(|s| s.ident.to_string())
            .collect::<Vec<_>>()
            .join("::")
    } else {
        "Unknown".to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::checker::Checker;

    #[test]
    fn test_detects_inconsistent_errors() {
        let code = r#"
            mod example {
                fn a() -> Result<(), std::io::Error> { Ok(()) }
                fn b() -> Result<(), String> { Ok(()) }
                fn c() -> Result<(), MyError> { Ok(()) }
            }
        "#;

        let syntax = syn::parse_file(code).unwrap();
        let checker = E1804InconsistentErrorTypes::default();

        let mut violations = Vec::new();
        for item in &syntax.items {
            violations.extend(checker.check_item(item, "test.rs").unwrap());
        }

        assert_eq!(violations.len(), 1);
    }

    #[test]
    fn test_consistent_errors_pass() {
        let code = r#"
            mod example {
                fn a() -> Result<(), MyError> { Ok(()) }
                fn b() -> Result<(), MyError> { Ok(()) }
            }
        "#;

        let syntax = syn::parse_file(code).unwrap();
        let checker = E1804InconsistentErrorTypes::default();

        let mut violations = Vec::new();
        for item in &syntax.items {
            violations.extend(checker.check_item(item, "test.rs").unwrap());
        }

        assert_eq!(violations.len(), 0);
    }
}
