//! E1214: Macro-generated trait implementations
//!
//! Detects macro invocations that generate trait implementations, which can
//! hide complexity and make code harder to understand.
//!
//! Example:
//! ```text
//! // Derive macros on complex types
//! #[derive(Clone, Debug, Serialize, Deserialize)]
//! struct VeryLargeStruct {
//!     // 20+ fields...
//! }
//!
//! // Procedural macros in impl blocks
//! impl MyTrait for Foo {
//!     some_macro!();  // Hidden complexity
//! }
//! ```

use crate::{checker::Checker, define_checker, violation::Violation};

define_checker! {
    /// Checker for E1214: Macro-generated trait implementations
    E1214MacroImpl,
    code = "E1214",
    name = "Macro-generated trait implementations",
    suggestions = "Review generated code with cargo expand. Consider if manual implementation would be clearer for complex types.",
    target_items = [Struct, Enum, Impl],
    config_entry_name = "e1214_macro_impl",
    config = E1214Config {
        enabled: bool = true,
        severity: crate::config::SeverityLevel = crate::config::SeverityLevel::Low,
        categories: Vec<crate::config::CheckerCategory> = vec![crate::config::CheckerCategory::Complexity],
        /// Minimum fields to warn about derive on large structs
        large_struct_threshold: usize = 10,
        /// Warn about macros inside impl blocks
        warn_impl_macros: bool = true,
    },
    check_item(self, item, file_path) {
        let mut violations = Vec::new();

        match item {
            syn::Item::Struct(s) => {
                self.check_derive_on_struct(s, file_path, &mut violations);
            }
            syn::Item::Enum(e) => {
                self.check_derive_on_enum(e, file_path, &mut violations);
            }
            syn::Item::Impl(i) => {
                self.check_impl_macros(i, file_path, &mut violations);
            }
            _ => {}
        }

        Ok(violations)
    }
}

impl E1214MacroImpl {
    fn count_fields(fields: &syn::Fields) -> usize {
        match fields {
            syn::Fields::Named(f) => f.named.len(),
            syn::Fields::Unnamed(f) => f.unnamed.len(),
            syn::Fields::Unit => 0,
        }
    }

    fn get_derive_traits(attrs: &[syn::Attribute]) -> Vec<String> {
        let mut traits = Vec::new();

        for attr in attrs {
            if attr.path().is_ident("derive") {
                if let Ok(meta) = attr.meta.require_list() {
                    let tokens = meta.tokens.to_string();
                    // Parse the derive traits
                    for part in tokens.split(',') {
                        let trait_name = part.trim().to_string();
                        if !trait_name.is_empty() {
                            traits.push(trait_name);
                        }
                    }
                }
            }
        }

        traits
    }

    fn check_derive_on_struct(
        &self,
        s: &syn::ItemStruct,
        file_path: &str,
        violations: &mut Vec<Violation>,
    ) {
        let field_count = Self::count_fields(&s.fields);
        let derives = Self::get_derive_traits(&s.attrs);

        if field_count >= self.config.large_struct_threshold && !derives.is_empty() {
            let start = s.ident.span().start();
            violations.push(
                Violation::new(
                    self.code(),
                    self.name(),
                    self.severity().into(),
                    format!(
                        "Derive macros ({}) on struct with {} fields. Generated code may be large; review with cargo expand.",
                        derives.join(", "),
                        field_count
                    ),
                    file_path,
                    start.line,
                    start.column + 1,
                )
                .with_suggestion(self.suggestions()),
            );
        }
    }

    fn check_derive_on_enum(
        &self,
        e: &syn::ItemEnum,
        file_path: &str,
        violations: &mut Vec<Violation>,
    ) {
        // Count total fields across all variants
        let total_fields: usize = e.variants.iter().map(|v| Self::count_fields(&v.fields)).sum();
        let derives = Self::get_derive_traits(&e.attrs);

        if total_fields >= self.config.large_struct_threshold && !derives.is_empty() {
            let start = e.ident.span().start();
            violations.push(
                Violation::new(
                    self.code(),
                    self.name(),
                    self.severity().into(),
                    format!(
                        "Derive macros ({}) on enum with {} total fields. Generated code may be large.",
                        derives.join(", "),
                        total_fields
                    ),
                    file_path,
                    start.line,
                    start.column + 1,
                )
                .with_suggestion(self.suggestions()),
            );
        }
    }

    fn check_impl_macros(
        &self,
        impl_block: &syn::ItemImpl,
        file_path: &str,
        violations: &mut Vec<Violation>,
    ) {
        use syn::spanned::Spanned;

        if !self.config.warn_impl_macros {
            return;
        }

        for item in &impl_block.items {
            if let syn::ImplItem::Macro(mac) = item {
                let start = mac.span().start();
                let macro_name = mac
                    .mac
                    .path
                    .segments
                    .last()
                    .map(|s| s.ident.to_string())
                    .unwrap_or_else(|| "unknown".to_string());

                violations.push(
                    Violation::new(
                        self.code(),
                        self.name(),
                        self.severity().into(),
                        format!(
                            "Macro '{}!' invocation inside impl block. This may generate hidden trait implementations.",
                            macro_name
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::checker::Checker;

    #[test]
    fn test_detects_derive_on_large_struct() {
        let code = r#"
            #[derive(Clone, Debug)]
            struct Large {
                f1: i32, f2: i32, f3: i32, f4: i32, f5: i32,
                f6: i32, f7: i32, f8: i32, f9: i32, f10: i32,
                f11: i32,
            }
        "#;

        let syntax = syn::parse_file(code).unwrap();
        let checker = E1214MacroImpl::default();

        let mut violations = Vec::new();
        for item in &syntax.items {
            violations.extend(checker.check_item(item, "test.rs").unwrap());
        }

        assert_eq!(violations.len(), 1);
        assert!(violations[0].message.contains("11 fields"));
    }

    #[test]
    fn test_small_struct_passes() {
        let code = r#"
            #[derive(Clone, Debug)]
            struct Small {
                a: i32,
                b: i32,
            }
        "#;

        let syntax = syn::parse_file(code).unwrap();
        let checker = E1214MacroImpl::default();

        let mut violations = Vec::new();
        for item in &syntax.items {
            violations.extend(checker.check_item(item, "test.rs").unwrap());
        }

        assert!(violations.is_empty());
    }

    #[test]
    fn test_detects_macro_in_impl() {
        let code = r#"
            struct Foo;

            impl Foo {
                generate_methods!();
            }
        "#;

        let syntax = syn::parse_file(code).unwrap();
        let checker = E1214MacroImpl::default();

        let mut violations = Vec::new();
        for item in &syntax.items {
            violations.extend(checker.check_item(item, "test.rs").unwrap());
        }

        assert_eq!(violations.len(), 1);
        assert!(violations[0].message.contains("generate_methods"));
    }

    #[test]
    fn test_impl_without_macros_passes() {
        let code = r#"
            struct Foo;

            impl Foo {
                fn new() -> Self {
                    Foo
                }
            }
        "#;

        let syntax = syn::parse_file(code).unwrap();
        let checker = E1214MacroImpl::default();

        let mut violations = Vec::new();
        for item in &syntax.items {
            violations.extend(checker.check_item(item, "test.rs").unwrap());
        }

        assert!(violations.is_empty());
    }
}
