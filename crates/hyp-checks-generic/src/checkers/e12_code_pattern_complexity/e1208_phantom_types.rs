//! E1208: Phantom types and zero-sized markers
//!
//! Detects `PhantomData` usage patterns and warns about potential misuse
//! or lack of documentation.
//!
//! Example:
//! ```text
//! // Should document why PhantomData is needed
//! struct Wrapper<T> {
//!     _marker: PhantomData<T>,
//! }
//! ```

use crate::{checker::Checker, define_checker, violation::Violation};

define_checker! {
    /// Checker for E1208: Phantom types and zero-sized markers
    E1208PhantomTypes,
    code = "E1208",
    name = "Phantom types and zero-sized markers",
    suggestions = "Document why PhantomData is needed. If using for variance, consider PhantomData<fn() -> T> for covariance or PhantomData<*const T> for invariance.",
    target_items = [Struct, Enum],
    config_entry_name = "e1208_phantom_types",
    config = E1208Config {
        enabled: bool = true,
        severity: crate::config::SeverityLevel = crate::config::SeverityLevel::Medium,
        categories: Vec<crate::config::CheckerCategory> = vec![crate::config::CheckerCategory::Complexity],
        /// Maximum number of PhantomData fields before warning
        max_phantom_fields: usize = 2,
        /// Require doc comment on PhantomData fields
        require_docs: bool = true,
    },
    check_item(self, item, file_path) {
        let mut visitor = PhantomVisitor {
            violations: Vec::new(),
            file_path,
            checker: self,
            phantom_count: 0,
        };

        match item {
            syn::Item::Struct(s) => {
                visitor.check_struct(s);
            }
            syn::Item::Enum(e) => {
                visitor.check_enum(e);
            }
            _ => {}
        }

        Ok(visitor.violations)
    }
}

struct PhantomVisitor<'a> {
    violations: Vec<Violation>,
    file_path: &'a str,
    checker: &'a E1208PhantomTypes,
    phantom_count: usize,
}

impl<'a> PhantomVisitor<'a> {
    fn is_phantom_data(ty: &syn::Type) -> bool {
        if let syn::Type::Path(type_path) = ty {
            let path_str = type_path
                .path
                .segments
                .iter()
                .map(|s| s.ident.to_string())
                .collect::<Vec<_>>()
                .join("::");

            return path_str == "PhantomData"
                || path_str == "std::marker::PhantomData"
                || path_str == "core::marker::PhantomData"
                || path_str.ends_with("::PhantomData");
        }
        false
    }

    fn check_field(&mut self, field: &syn::Field, has_doc: bool) {
        use syn::spanned::Spanned;

        if Self::is_phantom_data(&field.ty) {
            self.phantom_count += 1;

            if self.checker.config.require_docs && !has_doc {
                let start = field.ty.span().start();
                self.violations.push(
                    Violation::new(
                        self.checker.code(),
                        self.checker.name(),
                        self.checker.severity().into(),
                        "PhantomData field without documentation. Explain why this marker is needed.",
                        self.file_path,
                        start.line,
                        start.column + 1,
                    )
                    .with_suggestion(self.checker.suggestions()),
                );
            }
        }
    }

    fn field_has_doc(field: &syn::Field) -> bool {
        field.attrs.iter().any(|attr| attr.path().is_ident("doc"))
    }

    fn check_struct(&mut self, s: &syn::ItemStruct) {
        match &s.fields {
            syn::Fields::Named(fields) => {
                for field in &fields.named {
                    let has_doc = Self::field_has_doc(field);
                    self.check_field(field, has_doc);
                }
            }
            syn::Fields::Unnamed(fields) => {
                for field in &fields.unnamed {
                    let has_doc = Self::field_has_doc(field);
                    self.check_field(field, has_doc);
                }
            }
            syn::Fields::Unit => {}
        }

        // Check for too many PhantomData fields
        if self.phantom_count > self.checker.config.max_phantom_fields {
            let start = s.ident.span().start();
            self.violations.push(
                Violation::new(
                    self.checker.code(),
                    self.checker.name(),
                    self.checker.severity().into(),
                    format!(
                        "Struct has {} PhantomData fields (max {}). This may indicate overly complex type-level programming.",
                        self.phantom_count,
                        self.checker.config.max_phantom_fields
                    ),
                    self.file_path,
                    start.line,
                    start.column + 1,
                )
                .with_suggestion(self.checker.suggestions()),
            );
        }
    }

    fn check_enum(&mut self, e: &syn::ItemEnum) {
        for variant in &e.variants {
            match &variant.fields {
                syn::Fields::Named(fields) => {
                    for field in &fields.named {
                        let has_doc = Self::field_has_doc(field);
                        self.check_field(field, has_doc);
                    }
                }
                syn::Fields::Unnamed(fields) => {
                    for field in &fields.unnamed {
                        let has_doc = Self::field_has_doc(field);
                        self.check_field(field, has_doc);
                    }
                }
                syn::Fields::Unit => {}
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::checker::Checker;

    #[test]
    fn test_detects_undocumented_phantom() {
        let code = r#"
            use std::marker::PhantomData;

            struct Wrapper<T> {
                _marker: PhantomData<T>,
            }
        "#;

        let syntax = syn::parse_file(code).unwrap();
        let checker = E1208PhantomTypes::default();

        let mut violations = Vec::new();
        for item in &syntax.items {
            violations.extend(checker.check_item(item, "test.rs").unwrap());
        }

        assert_eq!(violations.len(), 1);
        assert!(violations[0].message.contains("without documentation"));
    }

    #[test]
    fn test_documented_phantom_passes() {
        let code = r#"
            use std::marker::PhantomData;

            struct Wrapper<T> {
                /// Marker for type T ownership
                _marker: PhantomData<T>,
            }
        "#;

        let syntax = syn::parse_file(code).unwrap();
        let checker = E1208PhantomTypes::default();

        let mut violations = Vec::new();
        for item in &syntax.items {
            violations.extend(checker.check_item(item, "test.rs").unwrap());
        }

        assert!(violations.is_empty());
    }

    #[test]
    fn test_detects_too_many_phantoms() {
        let code = r#"
            use std::marker::PhantomData;

            struct Complex<T, U, V> {
                /// Marker T
                _t: PhantomData<T>,
                /// Marker U
                _u: PhantomData<U>,
                /// Marker V
                _v: PhantomData<V>,
            }
        "#;

        let syntax = syn::parse_file(code).unwrap();
        let checker = E1208PhantomTypes::default();

        let mut violations = Vec::new();
        for item in &syntax.items {
            violations.extend(checker.check_item(item, "test.rs").unwrap());
        }

        assert_eq!(violations.len(), 1);
        assert!(violations[0].message.contains("3 PhantomData fields"));
    }

    #[test]
    fn test_no_phantom_passes() {
        let code = r#"
            struct Simple {
                value: i32,
            }
        "#;

        let syntax = syn::parse_file(code).unwrap();
        let checker = E1208PhantomTypes::default();

        let mut violations = Vec::new();
        for item in &syntax.items {
            violations.extend(checker.check_item(item, "test.rs").unwrap());
        }

        assert!(violations.is_empty());
    }
}
