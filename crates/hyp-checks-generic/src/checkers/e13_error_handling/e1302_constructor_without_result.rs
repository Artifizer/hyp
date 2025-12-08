//! E1302: Constructors returning bare values instead of Result
//!
//! Detects new() constructors that can fail (contain unwrap/expect/panic) but don't
//! return Result, forcing them to panic on invalid input.

use crate::{define_checker, violation::Violation};

use syn::visit::Visit;

define_checker! {
    /// Checker for E1302: Constructor without Result
    E1302ConstructorWithoutResult,
    code = "E1302",
    name = "Constructor can fail but doesn't return Result",
    suggestions = "Return Result<Self, Error> from constructors that can fail, or use try_new() naming convention",
    target_items = [Impl],
    config_entry_name = "e1302_constructor_without_result",
    config = E1302Config {
        enabled: bool = true,
        severity: crate::config::SeverityLevel = crate::config::SeverityLevel::Medium,
        categories: Vec<crate::config::CheckerCategory> = vec![crate::config::CheckerCategory::Operations],
    },
    check_item(self, item, file_path) {
        let mut violations = Vec::new();

        if let syn::Item::Impl(impl_item) = item {
            for item in &impl_item.items {
                if let syn::ImplItem::Fn(method) = item {
                    let method_name = method.sig.ident.to_string();

                    // Only check new() methods
                    if method_name != "new" {
                        continue;
                    }

                    // Check if return type is Result
                    let returns_result = match &method.sig.output {
                        syn::ReturnType::Type(_, ty) => is_result_type(ty),
                        _ => false,
                    };

                    if !returns_result {
                        // Check if method body contains fallible operations
                        let mut visitor = FallibleOpVisitor { has_fallible: false };
                        visitor.visit_block(&method.block);

                        if visitor.has_fallible {
                            let start = method.sig.ident.span().start();
                            violations.push(
                                Violation::new(
                                    self.code(),
                                    self.name(),
                                    self.severity().into(),
                                    "Constructor new() can fail but doesn't return Result. Consider returning Result<Self, Error>.",
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

struct FallibleOpVisitor {
    has_fallible: bool,
}

impl<'ast> Visit<'ast> for FallibleOpVisitor {
    fn visit_expr_method_call(&mut self, node: &'ast syn::ExprMethodCall) {
        let method_name = node.method.to_string();
        if matches!(method_name.as_str(), "unwrap" | "expect") {
            self.has_fallible = true;
        }
        syn::visit::visit_expr_method_call(self, node);
    }

    fn visit_macro(&mut self, node: &'ast syn::Macro) {
        if let Some(ident) = node.path.get_ident() {
            let name = ident.to_string();
            if matches!(name.as_str(), "panic" | "unreachable" | "unimplemented") {
                self.has_fallible = true;
            }
        }
        syn::visit::visit_macro(self, node);
    }
}

fn is_result_type(ty: &syn::Type) -> bool {
    if let syn::Type::Path(type_path) = ty {
        if let Some(segment) = type_path.path.segments.last() {
            return segment.ident == "Result";
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::checker::Checker;

    #[test]
    fn test_detects_fallible_constructor() {
        let code = r#"
            struct Config {
                value: i32,
            }

            impl Config {
                pub fn new(s: &str) -> Self {
                    let value = s.parse().unwrap();
                    Self { value }
                }
            }
        "#;

        let syntax = syn::parse_file(code).unwrap();
        let checker = E1302ConstructorWithoutResult::default();

        let mut violations = Vec::new();
        for item in &syntax.items {
            violations.extend(checker.check_item(item, "test.rs").unwrap());
        }

        assert_eq!(violations.len(), 1);
    }

    #[test]
    fn test_result_constructor_passes() {
        let code = r#"
            struct Config {
                value: i32,
            }

            impl Config {
                pub fn new(s: &str) -> Result<Self, std::num::ParseIntError> {
                    let value = s.parse()?;
                    Ok(Self { value })
                }
            }
        "#;

        let syntax = syn::parse_file(code).unwrap();
        let checker = E1302ConstructorWithoutResult::default();

        let mut violations = Vec::new();
        for item in &syntax.items {
            violations.extend(checker.check_item(item, "test.rs").unwrap());
        }

        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_infallible_constructor_passes() {
        let code = r#"
            struct Point {
                x: i32,
                y: i32,
            }

            impl Point {
                pub fn new(x: i32, y: i32) -> Self {
                    Self { x, y }
                }
            }
        "#;

        let syntax = syn::parse_file(code).unwrap();
        let checker = E1302ConstructorWithoutResult::default();

        let mut violations = Vec::new();
        for item in &syntax.items {
            violations.extend(checker.check_item(item, "test.rs").unwrap());
        }

        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_panic_in_constructor() {
        let code = r#"
            struct Config {
                value: i32,
            }

            impl Config {
                pub fn new(value: i32) -> Self {
                    if value < 0 {
                        panic!("value must be non-negative");
                    }
                    Self { value }
                }
            }
        "#;

        let syntax = syn::parse_file(code).unwrap();
        let checker = E1302ConstructorWithoutResult::default();

        let mut violations = Vec::new();
        for item in &syntax.items {
            violations.extend(checker.check_item(item, "test.rs").unwrap());
        }

        assert_eq!(violations.len(), 1);
    }
}
