//! E1806: Exposing internal details
//!
//! Detects public functions or types that expose internal implementation details
//! in their API, such as returning private types or using implementation-specific types.

use crate::{define_checker, violation::Violation};

define_checker! {
    /// Checker for E1806: Exposing internal details
    E1806ExposingInternalDetails,
    code = "E1806",
    name = "Exposing internal details",
    suggestions = "Use abstraction layers or newtype wrappers to hide implementation details",
    target_items = [Function],
    config_entry_name = "e1806_exposing_internal_details",
    config = E1806Config {
        enabled: bool = true,
        severity: crate::config::SeverityLevel = crate::config::SeverityLevel::Medium,
        categories: Vec<crate::config::CheckerCategory> = vec![crate::config::CheckerCategory::Complexity],
    },
    check_item(self, item, file_path) {
        let mut violations = Vec::new();

        if let syn::Item::Fn(func) = item {
            // Only check public functions
            if !matches!(func.vis, syn::Visibility::Public(_)) {
                return Ok(violations);
            }

            // Check return type for internal details
            if let syn::ReturnType::Type(_, ty) = &func.sig.output {
                if exposes_internal_details(ty) {
                    let start = func.sig.ident.span().start();
                    violations.push(
                        Violation::new(
                            self.code(),
                            self.name(),
                            self.severity().into(),
                            format!(
                                "Public function '{}' exposes internal implementation details in its return type.",
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

            // Check parameters for internal details
            for param in &func.sig.inputs {
                if let syn::FnArg::Typed(pat_type) = param {
                    if exposes_internal_details(&pat_type.ty) {
                        let start = func.sig.ident.span().start();
                        violations.push(
                            Violation::new(
                                self.code(),
                                self.name(),
                                self.severity().into(),
                                format!(
                                    "Public function '{}' exposes internal implementation details in its parameters.",
                                    func.sig.ident
                                ),
                                file_path,
                                start.line,
                                start.column + 1,
                            )
                            .with_suggestion(self.suggestions()),
                        );
                        break; // One warning per function is enough
                    }
                }
            }
        }

        Ok(violations)
    }
}

fn exposes_internal_details(ty: &syn::Type) -> bool {
    if let syn::Type::Path(type_path) = ty {
        let path_str = type_path
            .path
            .segments
            .iter()
            .map(|s| s.ident.to_string())
            .collect::<Vec<_>>()
            .join("::");

        // Check for common internal implementation types
        if path_str.contains("Impl")
            || path_str.contains("Internal")
            || path_str.contains("Private")
            || path_str.contains("Raw")
        {
            return true;
        }

        // Check for raw pointers in public API
        if path_str.contains("*const") || path_str.contains("*mut") {
            return true;
        }
    }

    // Check for raw pointer types
    if matches!(ty, syn::Type::Ptr(_)) {
        return true;
    }

    false
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::checker::Checker;

    #[test]
    fn test_detects_internal_type() {
        let code = r#"
            pub fn get_internal() -> InternalState {
                InternalState {}
            }
        "#;

        let syntax = syn::parse_file(code).unwrap();
        let checker = E1806ExposingInternalDetails::default();

        let mut violations = Vec::new();
        for item in &syntax.items {
            violations.extend(checker.check_item(item, "test.rs").unwrap());
        }

        assert_eq!(violations.len(), 1);
    }

    #[test]
    fn test_detects_raw_pointer() {
        let code = r#"
            pub fn get_ptr() -> *const u8 {
                std::ptr::null()
            }
        "#;

        let syntax = syn::parse_file(code).unwrap();
        let checker = E1806ExposingInternalDetails::default();

        let mut violations = Vec::new();
        for item in &syntax.items {
            violations.extend(checker.check_item(item, "test.rs").unwrap());
        }

        assert_eq!(violations.len(), 1);
    }

    #[test]
    fn test_public_api_passes() {
        let code = r#"
            pub fn get_value() -> String {
                String::new()
            }
        "#;

        let syntax = syn::parse_file(code).unwrap();
        let checker = E1806ExposingInternalDetails::default();

        let mut violations = Vec::new();
        for item in &syntax.items {
            violations.extend(checker.check_item(item, "test.rs").unwrap());
        }

        assert_eq!(violations.len(), 0);
    }
}
