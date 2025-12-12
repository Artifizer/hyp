//! E1709: Unnecessary boxing
//!
//! Detects Box<T> usage where the boxed type is small enough to be stack-allocated,
//! or where boxing provides no benefit.

use crate::{checker::Checker, define_checker, violation::Violation};

use syn::visit::Visit;

define_checker! {
    /// Checker for E1709: Unnecessary boxing
    E1709UnnecessaryBoxing,
    code = "E1709",
    name = "Unnecessary boxing",
    suggestions = "Consider using the type directly instead of Box<T> for small types",
    target_items = [Function, Struct],
    config_entry_name = "e1709_unnecessary_boxing",
    config = E1709Config {
        enabled: bool = true,
        severity: crate::config::SeverityLevel = crate::config::SeverityLevel::Low,
        categories: Vec<crate::config::CheckerCategory> = vec![crate::config::CheckerCategory::Complexity],
    },
    check_item(self, item, file_path) {
        let mut visitor = BoxVisitor {
            violations: Vec::new(),
            file_path,
            checker: self,
        };
        visitor.visit_item(item);
        Ok(visitor.violations)
    }
}

struct BoxVisitor<'a> {
    violations: Vec<Violation>,
    file_path: &'a str,
    checker: &'a E1709UnnecessaryBoxing,
}

impl<'a> Visit<'a> for BoxVisitor<'a> {
    fn visit_type(&mut self, node: &'a syn::Type) {
        if let syn::Type::Path(type_path) = node {
            if let Some(segment) = type_path.path.segments.last() {
                if segment.ident == "Box" {
                    if let syn::PathArguments::AngleBracketed(args) = &segment.arguments {
                        if let Some(syn::GenericArgument::Type(inner_ty)) = args.args.first() {
                            // Check for Box<primitive> or Box<small type>
                            if is_small_type(inner_ty) {
                                use syn::spanned::Spanned;
                                let start = node.span().start();
                                self.violations.push(
                                    Violation::new(
                                        self.checker.code(),
                                        self.checker.name(),
                                        self.checker.severity().into(),
                                        format!(
                                            "Box<{}> may be unnecessary. Consider using the type directly.",
                                            type_to_string(inner_ty)
                                        ),
                                        self.file_path,
                                        start.line,
                                        start.column + 1,
                                    )
                                    .with_suggestion(self.checker.suggestions()),
                                );
                            }
                        }
                    }
                }
            }
        }

        syn::visit::visit_type(self, node);
    }
}

fn is_small_type(ty: &syn::Type) -> bool {
    if let syn::Type::Path(type_path) = ty {
        if let Some(segment) = type_path.path.segments.last() {
            let name = segment.ident.to_string();
            // Primitive types and other small types
            return matches!(
                name.as_str(),
                "i8" | "i16" | "i32" | "i64" | "i128" | "isize"
                    | "u8" | "u16" | "u32" | "u64" | "u128" | "usize"
                    | "f32" | "f64"
                    | "bool" | "char"
                    | "Option" // Option<primitive> is still small
            );
        }
    }
    false
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
        "T".to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detects_boxed_primitive() {
        let code = r#"
            fn example() {
                let x: Box<i32> = Box::new(42);
            }
        "#;

        let syntax = syn::parse_file(code).unwrap();
        let checker = E1709UnnecessaryBoxing::default();

        let mut violations = Vec::new();
        for item in &syntax.items {
            violations.extend(checker.check_item(item, "test.rs").unwrap());
        }

        assert_eq!(violations.len(), 1);
    }

    #[test]
    fn test_boxed_trait_object_passes() {
        let code = r#"
            fn example() -> Box<dyn std::error::Error> {
                Box::new(std::io::Error::new(std::io::ErrorKind::Other, "error"))
            }
        "#;

        let syntax = syn::parse_file(code).unwrap();
        let checker = E1709UnnecessaryBoxing::default();

        let mut violations = Vec::new();
        for item in &syntax.items {
            violations.extend(checker.check_item(item, "test.rs").unwrap());
        }

        assert_eq!(violations.len(), 0);
    }
}
