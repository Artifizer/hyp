//! E1701: Oversized struct passed by value
//!
//! Detects function parameters that take large structs by value
//! instead of by reference.

use crate::{define_checker, violation::Violation};

use syn::spanned::Spanned;

define_checker! {
    /// Checker for E1701: Oversized struct passed by value
    E1701OversizedStructByValue,
    code = "E1701",
    name = "Oversized struct passed by value",
    suggestions = "Pass large types by reference (&T or &mut T) instead of by value",
    target_items = [Function],
    config_entry_name = "e1701_oversized_struct_by_value",
    config = E1701Config {
        enabled: bool = true,
        severity: crate::config::SeverityLevel = crate::config::SeverityLevel::Low,
        categories: Vec<crate::config::CheckerCategory> = vec![crate::config::CheckerCategory::Complexity],
        /// Threshold for warning - types with this many fields are considered large
        min_fields_threshold: usize = 8,
    },
    check_item(self, item, file_path) {
        let mut violations = Vec::new();

        if let syn::Item::Fn(func) = item {
            for input in &func.sig.inputs {
                if let syn::FnArg::Typed(pat_type) = input {
                    // Check for array types passed by value
                    if let syn::Type::Array(arr) = &*pat_type.ty {
                        if let syn::Expr::Lit(lit) = &arr.len {
                            if let syn::Lit::Int(int_lit) = &lit.lit {
                                if let Ok(len) = int_lit.base10_parse::<usize>() {
                                    if len > self.config.min_fields_threshold {
                                        let start = pat_type.span().start();
                                        violations.push(
                                            Violation::new(
                                                self.code(),
                                                self.name(),
                                                self.severity().into(),
                                                format!(
                                                    "Array of {} elements passed by value. Consider passing by reference.",
                                                    len
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
                    }

                    // Check for tuple types with many elements
                    if let syn::Type::Tuple(tuple) = &*pat_type.ty {
                        if tuple.elems.len() > self.config.min_fields_threshold {
                            let start = pat_type.span().start();
                            violations.push(
                                Violation::new(
                                    self.code(),
                                    self.name(),
                                    self.severity().into(),
                                    format!(
                                        "Tuple with {} elements passed by value. Consider passing by reference or using a struct.",
                                        tuple.elems.len()
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
        }

        Ok(violations)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::checker::Checker;

    #[test]
    fn test_detects_large_array_by_value() {
        let code = r#"
            fn example(arr: [i32; 100]) {
                println!("{:?}", arr);
            }
        "#;

        let syntax = syn::parse_file(code).unwrap();
        let checker = E1701OversizedStructByValue::default();

        let mut violations = Vec::new();
        for item in &syntax.items {
            violations.extend(checker.check_item(item, "test.rs").unwrap());
        }

        assert_eq!(violations.len(), 1);
    }

    #[test]
    fn test_reference_passes() {
        let code = r#"
            fn example(arr: &[i32; 100]) {
                println!("{:?}", arr);
            }
        "#;

        let syntax = syn::parse_file(code).unwrap();
        let checker = E1701OversizedStructByValue::default();

        let mut violations = Vec::new();
        for item in &syntax.items {
            violations.extend(checker.check_item(item, "test.rs").unwrap());
        }

        assert_eq!(violations.len(), 0);
    }
}
