//! E1206: Deeply nested generic types
//!
//! Detects generic types with excessive nesting depth that make code
//! hard to read and understand.
//!
//! Example:
//! ```text
//! // Bad: Too deeply nested
//! let x: Vec<Option<Result<Box<Arc<Mutex<T>>>, E>>> = ...;
//! ```

use crate::{checker::Checker, define_checker, violation::Violation};
use syn::visit::Visit;

define_checker! {
    /// Checker for E1206: Deeply nested generic types
    E1206NestedGenerics,
    code = "E1206",
    name = "Deeply nested generic types",
    suggestions = "Create a type alias for the nested type, or restructure to reduce nesting depth.",
    target_items = [Function, Struct, Enum, Impl],
    config_entry_name = "e1206_nested_generics",
    config = E1206Config {
        enabled: bool = true,
        severity: crate::config::SeverityLevel = crate::config::SeverityLevel::High,
        categories: Vec<crate::config::CheckerCategory> = vec![crate::config::CheckerCategory::Complexity],
        /// Maximum allowed generic nesting depth
        max_depth: usize = 4,
    },
    check_item(self, item, file_path) {
        let mut visitor = GenericNestingVisitor {
            violations: Vec::new(),
            file_path,
            checker: self,
        };
        visitor.visit_item(item);
        Ok(visitor.violations)
    }
}

struct GenericNestingVisitor<'a> {
    violations: Vec<Violation>,
    file_path: &'a str,
    checker: &'a E1206NestedGenerics,
}

impl<'a> GenericNestingVisitor<'a> {
    fn check_type_depth(&mut self, ty: &syn::Type) -> usize {
        match ty {
            syn::Type::Path(type_path) => {
                let mut max_depth = 0;

                for segment in &type_path.path.segments {
                    if let syn::PathArguments::AngleBracketed(args) = &segment.arguments {
                        for arg in &args.args {
                            if let syn::GenericArgument::Type(inner_ty) = arg {
                                let inner_depth = self.check_type_depth(inner_ty);
                                max_depth = max_depth.max(inner_depth);
                            }
                        }
                        // This level counts as 1 + the max depth inside
                        max_depth += 1;
                    }
                }

                max_depth
            }
            syn::Type::Reference(type_ref) => self.check_type_depth(&type_ref.elem),
            syn::Type::Ptr(type_ptr) => self.check_type_depth(&type_ptr.elem),
            syn::Type::Slice(type_slice) => 1 + self.check_type_depth(&type_slice.elem),
            syn::Type::Array(type_array) => 1 + self.check_type_depth(&type_array.elem),
            syn::Type::Tuple(type_tuple) => {
                let mut max_depth = 0;
                for elem in &type_tuple.elems {
                    max_depth = max_depth.max(self.check_type_depth(elem));
                }
                if !type_tuple.elems.is_empty() {
                    max_depth + 1
                } else {
                    0
                }
            }
            syn::Type::Paren(type_paren) => self.check_type_depth(&type_paren.elem),
            syn::Type::Group(type_group) => self.check_type_depth(&type_group.elem),
            _ => 0,
        }
    }

    fn report_if_too_deep(&mut self, ty: &syn::Type) {
        use syn::spanned::Spanned;

        let depth = self.check_type_depth(ty);
        if depth > self.checker.config.max_depth {
            let start = ty.span().start();
            self.violations.push(
                Violation::new(
                    self.checker.code(),
                    self.checker.name(),
                    self.checker.severity().into(),
                    format!(
                        "Generic type nesting depth is {} (max {}). Consider using a type alias.",
                        depth,
                        self.checker.config.max_depth
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

impl<'a> Visit<'a> for GenericNestingVisitor<'a> {
    fn visit_type(&mut self, ty: &'a syn::Type) {
        self.report_if_too_deep(ty);
        // Don't visit children to avoid duplicate reports
    }

    fn visit_field(&mut self, field: &'a syn::Field) {
        self.report_if_too_deep(&field.ty);
    }

    fn visit_fn_arg(&mut self, arg: &'a syn::FnArg) {
        if let syn::FnArg::Typed(pat_type) = arg {
            self.report_if_too_deep(&pat_type.ty);
        }
    }

    fn visit_return_type(&mut self, ret: &'a syn::ReturnType) {
        if let syn::ReturnType::Type(_, ty) = ret {
            self.report_if_too_deep(ty);
        }
    }

    fn visit_local(&mut self, local: &'a syn::Local) {
        if let Some(local_init) = &local.init {
            syn::visit::visit_expr(self, &local_init.expr);
        }

        // Check explicit type annotation
        if let syn::Pat::Type(pat_type) = &local.pat {
            self.report_if_too_deep(&pat_type.ty);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::checker::Checker;

    #[test]
    fn test_detects_deeply_nested_generics() {
        let code = r#"
            fn nested() -> Vec<Option<Result<Box<Arc<i32>>, String>>> {
                todo!()
            }
        "#;

        let syntax = syn::parse_file(code).unwrap();
        let checker = E1206NestedGenerics::default();

        let mut violations = Vec::new();
        for item in &syntax.items {
            violations.extend(checker.check_item(item, "test.rs").unwrap());
        }

        assert!(!violations.is_empty());
        assert!(violations[0].message.contains("nesting depth"));
    }

    #[test]
    fn test_detects_nested_in_struct_field() {
        let code = r#"
            struct Complex {
                field: Vec<Option<Result<Box<Arc<String>>, Error>>>,
            }
        "#;

        let syntax = syn::parse_file(code).unwrap();
        let checker = E1206NestedGenerics::default();

        let mut violations = Vec::new();
        for item in &syntax.items {
            violations.extend(checker.check_item(item, "test.rs").unwrap());
        }

        assert!(!violations.is_empty());
    }

    #[test]
    fn test_shallow_generics_pass() {
        let code = r#"
            fn simple() -> Option<Vec<String>> {
                None
            }
        "#;

        let syntax = syn::parse_file(code).unwrap();
        let checker = E1206NestedGenerics::default();

        let mut violations = Vec::new();
        for item in &syntax.items {
            violations.extend(checker.check_item(item, "test.rs").unwrap());
        }

        assert!(violations.is_empty());
    }

    #[test]
    fn test_result_option_combo_passes() {
        let code = r#"
            fn typical() -> Result<Option<String>, Error> {
                Ok(None)
            }
        "#;

        let syntax = syn::parse_file(code).unwrap();
        let checker = E1206NestedGenerics::default();

        let mut violations = Vec::new();
        for item in &syntax.items {
            violations.extend(checker.check_item(item, "test.rs").unwrap());
        }

        assert!(violations.is_empty());
    }
}
