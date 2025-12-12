//! E1707: Unbounded recursion
//!
//! Detects recursive functions that may not have proper base cases for all inputs,
//! potentially causing infinite recursion and stack overflow.

use crate::{define_checker, violation::Violation};

use syn::visit::Visit;

define_checker! {
    /// Checker for E1707: Unbounded recursion
    E1707UnboundedRecursion,
    code = "E1707",
    name = "Potentially unbounded recursion",
    suggestions = "Ensure all inputs have base cases, use unsigned types when negative values are invalid, add validation",
    target_items = [Function],
    config_entry_name = "e1707_unbounded_recursion",
    config = E1707Config {
        enabled: bool = true,
        severity: crate::config::SeverityLevel = crate::config::SeverityLevel::High,
        categories: Vec<crate::config::CheckerCategory> = vec![crate::config::CheckerCategory::Complexity],
    },
    check_item(self, item, file_path) {
        let mut violations = Vec::new();

        if let syn::Item::Fn(func) = item {
            let fn_name = func.sig.ident.to_string();

            // Check if function is recursive
            let mut recursion_visitor = RecursionDetector {
                fn_name: &fn_name,
                is_recursive: false,
            };
            recursion_visitor.visit_block(&func.block);

            if recursion_visitor.is_recursive {
                // Check if function takes signed integers but might not handle all cases
                let has_signed_param = func.sig.inputs.iter().any(|arg| {
                    if let syn::FnArg::Typed(pat_type) = arg {
                        is_signed_integer_type(&pat_type.ty)
                    } else {
                        false
                    }
                });

                // Check for subtraction in recursive calls (common pattern for potential infinite recursion)
                let mut subtraction_visitor = SubtractionInRecursionVisitor {
                    fn_name: &fn_name,
                    has_subtraction_in_call: false,
                };
                subtraction_visitor.visit_block(&func.block);

                if has_signed_param && subtraction_visitor.has_subtraction_in_call {
                    let start = func.sig.ident.span().start();
                    violations.push(
                        Violation::new(
                            self.code(),
                            self.name(),
                            self.severity().into(),
                            format!(
                                "Recursive function '{}' takes signed integer and uses subtraction. May cause infinite recursion for negative inputs.",
                                fn_name
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

        Ok(violations)
    }
}

struct RecursionDetector<'a> {
    fn_name: &'a str,
    is_recursive: bool,
}

impl<'ast> Visit<'ast> for RecursionDetector<'_> {
    fn visit_expr_call(&mut self, node: &'ast syn::ExprCall) {
        if let syn::Expr::Path(path) = &*node.func {
            if let Some(segment) = path.path.segments.last() {
                if segment.ident == self.fn_name {
                    self.is_recursive = true;
                }
            }
        }
        syn::visit::visit_expr_call(self, node);
    }
}

struct SubtractionInRecursionVisitor<'a> {
    fn_name: &'a str,
    has_subtraction_in_call: bool,
}

impl<'ast> Visit<'ast> for SubtractionInRecursionVisitor<'_> {
    fn visit_expr_call(&mut self, node: &'ast syn::ExprCall) {
        if let syn::Expr::Path(path) = &*node.func {
            if let Some(segment) = path.path.segments.last() {
                if segment.ident == self.fn_name {
                    // Check if any argument contains subtraction
                    for arg in &node.args {
                        if contains_subtraction(arg) {
                            self.has_subtraction_in_call = true;
                        }
                    }
                }
            }
        }
        syn::visit::visit_expr_call(self, node);
    }
}

fn is_signed_integer_type(ty: &syn::Type) -> bool {
    if let syn::Type::Path(type_path) = ty {
        if let Some(segment) = type_path.path.segments.last() {
            let name = segment.ident.to_string();
            return matches!(name.as_str(), "i8" | "i16" | "i32" | "i64" | "i128" | "isize");
        }
    }
    false
}

fn contains_subtraction(expr: &syn::Expr) -> bool {
    struct SubVisitor {
        found: bool,
    }

    impl<'ast> Visit<'ast> for SubVisitor {
        fn visit_expr_binary(&mut self, node: &'ast syn::ExprBinary) {
            if matches!(node.op, syn::BinOp::Sub(_)) {
                self.found = true;
            }
            syn::visit::visit_expr_binary(self, node);
        }
    }

    let mut visitor = SubVisitor { found: false };
    visitor.visit_expr(expr);
    visitor.found
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::checker::Checker;

    #[test]
    fn test_detects_unbounded_recursion() {
        let code = r#"
            fn count(n: i32) -> i32 {
                if n == 0 {
                    0
                } else {
                    1 + count(n - 1)
                }
            }
        "#;

        let syntax = syn::parse_file(code).unwrap();
        let checker = E1707UnboundedRecursion::default();

        let mut violations = Vec::new();
        for item in &syntax.items {
            violations.extend(checker.check_item(item, "test.rs").unwrap());
        }

        assert_eq!(violations.len(), 1);
    }

    #[test]
    fn test_unsigned_recursion_passes() {
        let code = r#"
            fn count(n: u32) -> u32 {
                if n == 0 {
                    0
                } else {
                    1 + count(n - 1)
                }
            }
        "#;

        let syntax = syn::parse_file(code).unwrap();
        let checker = E1707UnboundedRecursion::default();

        let mut violations = Vec::new();
        for item in &syntax.items {
            violations.extend(checker.check_item(item, "test.rs").unwrap());
        }

        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_non_recursive_passes() {
        let code = r#"
            fn add(a: i32, b: i32) -> i32 {
                a + b
            }
        "#;

        let syntax = syn::parse_file(code).unwrap();
        let checker = E1707UnboundedRecursion::default();

        let mut violations = Vec::new();
        for item in &syntax.items {
            violations.extend(checker.check_item(item, "test.rs").unwrap());
        }

        assert_eq!(violations.len(), 0);
    }
}
