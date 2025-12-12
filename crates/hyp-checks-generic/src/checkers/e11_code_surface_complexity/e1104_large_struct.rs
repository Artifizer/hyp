//! E1104: Overly large struct (too many fields)
//!
//! Detects structs with too many fields, which indicates the struct may be
//! trying to do too much and should be split into smaller, focused types.

use crate::{define_checker, violation::Violation};

define_checker! {
    /// Checker for E1104: Overly large struct
    E1104LargeStruct,
    code = "E1104",
    name = "Overly large struct",
    suggestions = "Split the struct into smaller, focused types. Group related fields into nested structs.",
    target_items = [Struct],
    config_entry_name = "e1104_large_struct",
    /// Configuration for E1104: Large struct checker
    config = E1104Config {
        /// Whether this checker is enabled
        enabled: bool = true,
        /// Severity level, defaults to Medium
        severity: crate::config::SeverityLevel = crate::config::SeverityLevel::Medium,
        /// Categories this checker belongs to
        categories: Vec<crate::config::CheckerCategory> = vec![crate::config::CheckerCategory::Complexity],
        /// Maximum allowed fields
        max_fields: usize = 12,
    },
    check_item(self, item, file_path) {
        let mut violations = Vec::new();

        if let syn::Item::Struct(s) = item {
            let field_count = match &s.fields {
                syn::Fields::Named(fields) => fields.named.len(),
                syn::Fields::Unnamed(fields) => fields.unnamed.len(),
                syn::Fields::Unit => 0,
            };

            if field_count > self.config.max_fields {
                let struct_name = s.ident.to_string();
                let span = s.ident.span().start();

                violations.push(
                    Violation::new(
                        self.code(),
                        self.name(),
                        self.severity().into(),
                        format!(
                            "Struct '{}' has {} fields, exceeding the limit of {}. Large structs are hard to construct and maintain.",
                            struct_name, field_count, self.config.max_fields
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
    fn test_small_struct_passes() {
        let code = r#"
            struct Point {
                x: f64,
                y: f64,
            }
        "#;

        let syntax = syn::parse_file(code).unwrap();
        let checker = E1104LargeStruct::default();

        let mut violations = Vec::new();
        for item in &syntax.items {
            violations.extend(checker.check_item(item, "test.rs").unwrap());
        }

        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_detects_large_struct() {
        let code = r#"
            struct GodObject {
                field1: i32,
                field2: i32,
                field3: i32,
                field4: i32,
                field5: i32,
                field6: i32,
                field7: i32,
                field8: i32,
                field9: i32,
                field10: i32,
                field11: i32,
                field12: i32,
                field13: i32,
                field14: i32,
                field15: i32,
            }
        "#;

        let syntax = syn::parse_file(code).unwrap();
        let checker = E1104LargeStruct::default();

        let mut violations = Vec::new();
        for item in &syntax.items {
            violations.extend(checker.check_item(item, "test.rs").unwrap());
        }

        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].code, "E1104");
        assert!(violations[0].message.contains("15 fields"));
    }

    #[test]
    fn test_tuple_struct() {
        let code = r#"
            struct TooManyTuples(i32, i32, i32, i32, i32, i32, i32, i32, i32, i32, i32, i32, i32, i32, i32);
        "#;

        let syntax = syn::parse_file(code).unwrap();
        let checker = E1104LargeStruct::default();

        let mut violations = Vec::new();
        for item in &syntax.items {
            violations.extend(checker.check_item(item, "test.rs").unwrap());
        }

        assert_eq!(violations.len(), 1);
    }

    #[test]
    fn test_unit_struct_passes() {
        let code = r#"
            struct Unit;
        "#;

        let syntax = syn::parse_file(code).unwrap();
        let checker = E1104LargeStruct::default();

        let mut violations = Vec::new();
        for item in &syntax.items {
            violations.extend(checker.check_item(item, "test.rs").unwrap());
        }

        assert_eq!(violations.len(), 0);
    }
}
