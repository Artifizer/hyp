//! E1802: Public fields without validation
//!
//! Detects structs with public fields that might benefit from
//! getter/setter methods with validation.

use crate::{define_checker, violation::Violation};

define_checker! {
    /// Checker for E1802: Public fields without validation
    E1802PublicFields,
    code = "E1802",
    name = "Public fields without validation",
    suggestions = "Consider using private fields with getter/setter methods for validation",
    target_items = [Struct],
    config_entry_name = "e1802_public_fields",
    config = E1802Config {
        enabled: bool = true,
        severity: crate::config::SeverityLevel = crate::config::SeverityLevel::Medium,
        categories: Vec<crate::config::CheckerCategory> = vec![crate::config::CheckerCategory::Complexity],
        /// Minimum number of public fields to trigger warning
        min_public_fields: usize = 3,
    },
    check_item(self, item, file_path) {
        let mut violations = Vec::new();

        if let syn::Item::Struct(struct_item) = item {
            // Only check pub structs
            if !matches!(struct_item.vis, syn::Visibility::Public(_)) {
                return Ok(violations);
            }

            let public_fields: Vec<_> = match &struct_item.fields {
                syn::Fields::Named(fields) => fields
                    .named
                    .iter()
                    .filter(|f| matches!(f.vis, syn::Visibility::Public(_)))
                    .collect(),
                _ => return Ok(violations),
            };

            if public_fields.len() >= self.config.min_public_fields {
                let start = struct_item.ident.span().start();
                violations.push(
                    Violation::new(
                        self.code(),
                        self.name(),
                        self.severity().into(),
                        format!(
                            "Struct '{}' has {} public fields. Consider using private fields with getters/setters.",
                            struct_item.ident,
                            public_fields.len()
                        ),
                        file_path,
                        start.line,
                        start.column + 1,
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
    fn test_detects_many_public_fields() {
        let code = r#"
            pub struct Config {
                pub host: String,
                pub port: u16,
                pub timeout: u64,
            }
        "#;

        let syntax = syn::parse_file(code).unwrap();
        let checker = E1802PublicFields::default();

        let mut violations = Vec::new();
        for item in &syntax.items {
            violations.extend(checker.check_item(item, "test.rs").unwrap());
        }

        assert_eq!(violations.len(), 1);
    }

    #[test]
    fn test_private_fields_pass() {
        let code = r#"
            pub struct Config {
                host: String,
                port: u16,
                timeout: u64,
            }
        "#;

        let syntax = syn::parse_file(code).unwrap();
        let checker = E1802PublicFields::default();

        let mut violations = Vec::new();
        for item in &syntax.items {
            violations.extend(checker.check_item(item, "test.rs").unwrap());
        }

        assert_eq!(violations.len(), 0);
    }
}
