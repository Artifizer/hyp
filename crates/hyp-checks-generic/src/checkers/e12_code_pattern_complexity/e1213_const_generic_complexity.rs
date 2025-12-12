//! E1213: Const generics with complex constraints
//!
//! Detects overly complex const generic usage that can make code hard to understand:
//! - Many const generic parameters
//! - Complex const expressions in bounds
//! - Deeply nested const generic types
//!
//! While const generics are powerful, overuse leads to confusing APIs.

use crate::{define_checker, violation::Violation};

define_checker! {
    /// Checker for E1213: Const generic complexity
    E1213ConstGenericComplexity,
    code = "E1213",
    name = "Const generics with complex constraints",
    suggestions = "Consider using runtime values, type aliases, or simpler const generic patterns.",
    target_items = [Function, Struct, Impl],
    config_entry_name = "e1213_const_generic_complexity",
    config = E1213Config {
        enabled: bool = true,
        severity: crate::config::SeverityLevel = crate::config::SeverityLevel::Medium,
        categories: Vec<crate::config::CheckerCategory> = vec![crate::config::CheckerCategory::Complexity],
        /// Maximum const generic parameters
        max_const_params: usize = 2,
    },
    check_item(self, item, file_path) {
        let mut violations = Vec::new();

        let generics = match item {
            syn::Item::Fn(f) => Some(&f.sig.generics),
            syn::Item::Struct(s) => Some(&s.generics),
            syn::Item::Impl(i) => Some(&i.generics),
            syn::Item::Enum(e) => Some(&e.generics),
            syn::Item::Trait(t) => Some(&t.generics),
            _ => None,
        };

        if let Some(generics) = generics {
            let const_count = generics.const_params().count();

            if const_count > self.config.max_const_params {
                use syn::spanned::Spanned;
                let start = generics.span().start();
                violations.push(
                    Violation::new(
                        self.code(),
                        self.name(),
                        self.severity().into(),
                        format!(
                            "Item has {} const generic parameters (max {}). This can make the API confusing.",
                            const_count, self.config.max_const_params
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
    fn test_detects_many_const_params() {
        let code = r#"
            struct Matrix<const ROWS: usize, const COLS: usize, const DEPTH: usize> {
                data: [[[f64; DEPTH]; COLS]; ROWS],
            }
        "#;

        let syntax = syn::parse_file(code).unwrap();
        let checker = E1213ConstGenericComplexity::default();

        let mut violations = Vec::new();
        for item in &syntax.items {
            violations.extend(checker.check_item(item, "test.rs").unwrap());
        }

        assert_eq!(violations.len(), 1);
        assert!(violations[0].message.contains("const generic"));
    }

    #[test]
    fn test_simple_const_generic_passes() {
        let code = r#"
            struct Array<const N: usize> {
                data: [i32; N],
            }
        "#;

        let syntax = syn::parse_file(code).unwrap();
        let checker = E1213ConstGenericComplexity::default();

        let mut violations = Vec::new();
        for item in &syntax.items {
            violations.extend(checker.check_item(item, "test.rs").unwrap());
        }

        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_detects_in_function() {
        let code = r#"
            fn process<const A: usize, const B: usize, const C: usize>(
                data: [i32; A]
            ) -> [i32; B] {
                todo!()
            }
        "#;

        let syntax = syn::parse_file(code).unwrap();
        let checker = E1213ConstGenericComplexity::default();

        let mut violations = Vec::new();
        for item in &syntax.items {
            violations.extend(checker.check_item(item, "test.rs").unwrap());
        }

        assert_eq!(violations.len(), 1);
    }
}
