//! E1809: Fallible new()
//!
//! Detects new() functions that can fail but don't return Result or Option,
//! instead panicking on invalid input.

use crate::{define_checker, violation::Violation};

use syn::visit::Visit;

define_checker! {
    /// Checker for E1809: Fallible new()
    E1809FallibleNew,
    code = "E1809",
    name = "Fallible new()",
    suggestions = "Use Result<Self, Error> for new() that can fail, or provide try_new() alternative",
    target_items = [Impl],
    config_entry_name = "e1809_fallible_new",
    config = E1809Config {
        enabled: bool = true,
        severity: crate::config::SeverityLevel = crate::config::SeverityLevel::Medium,
        categories: Vec<crate::config::CheckerCategory> = vec![crate::config::CheckerCategory::Complexity],
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

                    // Check if return type is NOT Result or Option
                    let returns_fallible = match &method.sig.output {
                        syn::ReturnType::Type(_, ty) => is_result_or_option(ty),
                        _ => false,
                    };

                    if !returns_fallible {
                        // Check if body contains panic, unwrap, or expect
                        let mut visitor = PanicVisitor { has_panic: false };
                        visitor.visit_block(&method.block);

                        if visitor.has_panic {
                            let start = method.sig.ident.span().start();
                            violations.push(
                                Violation::new(
                                    self.code(),
                                    self.name(),
                                    self.severity().into(),
                                    "new() can panic but doesn't return Result. Consider returning Result<Self, Error>.",
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

struct PanicVisitor {
    has_panic: bool,
}

impl<'ast> Visit<'ast> for PanicVisitor {
    fn visit_expr_method_call(&mut self, node: &'ast syn::ExprMethodCall) {
        let method_name = node.method.to_string();
        if method_name == "unwrap" || method_name == "expect" {
            self.has_panic = true;
        }
        syn::visit::visit_expr_method_call(self, node);
    }

    fn visit_macro(&mut self, node: &'ast syn::Macro) {
        if let Some(ident) = node.path.get_ident() {
            let name = ident.to_string();
            if name == "panic" || name == "unreachable" || name == "unimplemented" {
                self.has_panic = true;
            }
        }
        syn::visit::visit_macro(self, node);
    }
}

fn is_result_or_option(ty: &syn::Type) -> bool {
    if let syn::Type::Path(type_path) = ty {
        if let Some(segment) = type_path.path.segments.last() {
            let name = segment.ident.to_string();
            return name == "Result" || name == "Option";
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::checker::Checker;

    #[test]
    fn test_detects_fallible_new() {
        let code = r#"
            struct Config {
                value: i32,
            }

            impl Config {
                fn new(value: i32) -> Self {
                    if value < 0 {
                        panic!("value must be non-negative");
                    }
                    Self { value }
                }
            }
        "#;

        let syntax = syn::parse_file(code).unwrap();
        let checker = E1809FallibleNew::default();

        let mut violations = Vec::new();
        for item in &syntax.items {
            violations.extend(checker.check_item(item, "test.rs").unwrap());
        }

        assert_eq!(violations.len(), 1);
    }

    #[test]
    fn test_result_new_passes() {
        let code = r#"
            struct Config {
                value: i32,
            }

            impl Config {
                fn new(value: i32) -> Result<Self, String> {
                    if value < 0 {
                        return Err("value must be non-negative".to_string());
                    }
                    Ok(Self { value })
                }
            }
        "#;

        let syntax = syn::parse_file(code).unwrap();
        let checker = E1809FallibleNew::default();

        let mut violations = Vec::new();
        for item in &syntax.items {
            violations.extend(checker.check_item(item, "test.rs").unwrap());
        }

        assert_eq!(violations.len(), 0);
    }
}
