//! E1013: Union with unsafe field access
//!
//! Detects union type definitions and field accesses. Unions are inherently unsafe
//! because reading from a union field reinterprets the bits as that type.

use crate::{checker::Checker, define_checker, violation::Violation};

use syn::visit::Visit;
use std::collections::HashSet;

define_checker! {
    /// Checker for E1013: Union with unsafe field access
    E1013UnionFieldAccess,
    code = "E1013",
    name = "Union with unsafe field access",
    suggestions = "Consider using enums with explicit variants, or ensure all union access is carefully validated",
    target_items = [Union, Function, Impl],
    config_entry_name = "e1013_union_field_access",
    /// Configuration for E1013: Union field access checker
    config = E1013Config {
        /// Whether this checker is enabled
        enabled: bool = true,
        /// Severity level, defaults to High
        severity: crate::config::SeverityLevel = crate::config::SeverityLevel::High,
        /// Categories this checker belongs to
        categories: Vec<crate::config::CheckerCategory> = vec![crate::config::CheckerCategory::Operations],
    },
    check_item(self, item, file_path) {
        let mut visitor = UnionVisitor {
            violations: Vec::new(),
            file_path,
            checker: self,
            union_names: HashSet::new(),
            in_unsafe_block: false,
            current_union_name: None,
            in_method_call: false,
            in_match_arm: false,
        };
        visitor.visit_item(item);
        Ok(visitor.violations)
    }
}

struct UnionVisitor<'a> {
    violations: Vec<Violation>,
    file_path: &'a str,
    checker: &'a E1013UnionFieldAccess,
    union_names: HashSet<String>,
    in_unsafe_block: bool,
    current_union_name: Option<String>,
    in_method_call: bool,
    in_match_arm: bool,
}

impl<'a> UnionVisitor<'a> {
    fn create_violation(&self, span: proc_macro2::Span, message: &str) -> Violation {
        let start = span.start();
        Violation::new(
            self.checker.code(),
            self.checker.name(),
            self.checker.severity().into(),
            message,
            self.file_path,
            start.line,
            start.column + 1,
        )
        .with_suggestion(self.checker.suggestions())
    }
}

impl<'a> Visit<'a> for UnionVisitor<'a> {
    fn visit_item_union(&mut self, node: &'a syn::ItemUnion) {
        // Track union name for later field access detection
        let union_name = node.ident.to_string();
        self.union_names.insert(union_name.clone());

        self.violations.push(self.create_violation(
            node.ident.span(),
            &format!(
                "Union type '{}' defined. Reading union fields is unsafe and can cause undefined behavior if the wrong variant is accessed.",
                node.ident
            ),
        ));

        // Store current union name for nested visits
        let old_union_name = self.current_union_name.clone();
        self.current_union_name = Some(union_name);
        syn::visit::visit_item_union(self, node);
        self.current_union_name = old_union_name;
    }

    fn visit_expr_unsafe(&mut self, node: &'a syn::ExprUnsafe) {
        let was_in_unsafe = self.in_unsafe_block;
        self.in_unsafe_block = true;
        syn::visit::visit_expr_unsafe(self, node);
        self.in_unsafe_block = was_in_unsafe;
    }

    fn visit_item_fn(&mut self, node: &'a syn::ItemFn) {
        // Check if function is marked unsafe
        let is_unsafe_fn = node.sig.unsafety.is_some();

        if is_unsafe_fn {
            let was_in_unsafe = self.in_unsafe_block;
            self.in_unsafe_block = true;
            syn::visit::visit_item_fn(self, node);
            self.in_unsafe_block = was_in_unsafe;
        } else {
            syn::visit::visit_item_fn(self, node);
        }
    }

    fn visit_impl_item_fn(&mut self, node: &'a syn::ImplItemFn) {
        // Check if method is marked unsafe
        let is_unsafe_fn = node.sig.unsafety.is_some();

        if is_unsafe_fn {
            let was_in_unsafe = self.in_unsafe_block;
            self.in_unsafe_block = true;
            syn::visit::visit_impl_item_fn(self, node);
            self.in_unsafe_block = was_in_unsafe;
        } else {
            syn::visit::visit_impl_item_fn(self, node);
        }
    }

    fn visit_expr_method_call(&mut self, node: &'a syn::ExprMethodCall) {
        let was_in_method_call = self.in_method_call;
        self.in_method_call = true;
        syn::visit::visit_expr_method_call(self, node);
        self.in_method_call = was_in_method_call;
    }

    fn visit_arm(&mut self, node: &'a syn::Arm) {
        let was_in_match_arm = self.in_match_arm;
        self.in_match_arm = true;
        syn::visit::visit_arm(self, node);
        self.in_match_arm = was_in_match_arm;
    }

    fn visit_expr_field(&mut self, node: &'a syn::ExprField) {
        use syn::spanned::Spanned;

        // Detect field access in unsafe context
        // Skip if we're inside a method call (e.g., self.value.get())
        // Skip if we're inside a match arm (indicates conditional validation)
        if self.in_unsafe_block && !self.in_method_call && !self.in_match_arm {
            // Check if the base expression could be a union type
            // This is a heuristic: we flag field accesses in unsafe contexts
            // as potential union field accesses
            if let syn::Member::Named(field_name) = &node.member {
                let start = node.span().start();
                self.violations.push(
                    Violation::new(
                        self.checker.code(),
                        self.checker.name(),
                        self.checker.severity().into(),
                        format!(
                            "Accessing field '{}' in unsafe context. This may be a union field access which can cause undefined behavior if the wrong variant is accessed.",
                            field_name
                        ),
                        self.file_path,
                        start.line,
                        start.column + 1,
                    )
                    .with_suggestion(self.checker.suggestions()),
                );
            }
        }

        syn::visit::visit_expr_field(self, node);
    }

    fn visit_expr_struct(&mut self, node: &'a syn::ExprStruct) {
        use syn::spanned::Spanned;

        // Check if this is a union initialization
        // Extract the type name from the path
        if let Some(type_name) = node.path.segments.last() {
            let type_str = type_name.ident.to_string();

            // If this is initializing a known union type, flag it
            if self.union_names.contains(&type_str) {
                let start = node.span().start();
                self.violations.push(
                    Violation::new(
                        self.checker.code(),
                        self.checker.name(),
                        self.checker.severity().into(),
                        format!(
                            "Initializing union type '{}'. Unions require manual tracking of the active variant and can cause undefined behavior if accessed incorrectly.",
                            type_str
                        ),
                        self.file_path,
                        start.line,
                        start.column + 1,
                    )
                    .with_suggestion(self.checker.suggestions()),
                );
            }
        }

        syn::visit::visit_expr_struct(self, node);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detects_union_definition() {
        let code = r#"
            union MyUnion {
                i: i32,
                f: f32,
            }
        "#;

        let syntax = syn::parse_file(code).unwrap();
        let checker = E1013UnionFieldAccess::default();

        let mut violations = Vec::new();
        for item in &syntax.items {
            violations.extend(checker.check_item(item, "test.rs").unwrap());
        }

        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].code, "E1013");
        assert!(violations[0].message.contains("MyUnion"));
    }

    #[test]
    fn test_detects_union_field_access_in_unsafe_fn() {
        let code = r#"
            union Value {
                int: i32,
                float: f32,
            }

            pub unsafe fn get_int(val: &Value) -> i32 {
                val.int
            }
        "#;

        let syntax = syn::parse_file(code).unwrap();
        let checker = E1013UnionFieldAccess::default();

        let mut violations = Vec::new();
        for item in &syntax.items {
            violations.extend(checker.check_item(item, "test.rs").unwrap());
        }

        // Should detect both the union definition and the field access
        assert!(violations.len() >= 2, "Expected at least 2 violations, got {}", violations.len());
        assert!(violations.iter().any(|v| v.message.contains("Union type")));
        assert!(violations.iter().any(|v| v.message.contains("Accessing field")));
    }

    #[test]
    fn test_detects_union_field_access_in_unsafe_block() {
        let code = r#"
            union Value {
                int: i32,
                float: f32,
            }

            pub fn get_int(val: &Value) -> i32 {
                unsafe { val.int }
            }
        "#;

        let syntax = syn::parse_file(code).unwrap();
        let checker = E1013UnionFieldAccess::default();

        let mut violations = Vec::new();
        for item in &syntax.items {
            violations.extend(checker.check_item(item, "test.rs").unwrap());
        }

        // Should detect both the union definition and the field access
        assert!(violations.len() >= 2, "Expected at least 2 violations, got {}", violations.len());
        assert!(violations.iter().any(|v| v.message.contains("Union type")));
        assert!(violations.iter().any(|v| v.message.contains("Accessing field")));
    }

    #[test]
    fn test_struct_passes() {
        let code = r#"
            struct MyStruct {
                i: i32,
                f: f32,
            }
        "#;

        let syntax = syn::parse_file(code).unwrap();
        let checker = E1013UnionFieldAccess::default();

        let mut violations = Vec::new();
        for item in &syntax.items {
            violations.extend(checker.check_item(item, "test.rs").unwrap());
        }

        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_enum_passes() {
        let code = r#"
            enum MyEnum {
                Int(i32),
                Float(f32),
            }
        "#;

        let syntax = syn::parse_file(code).unwrap();
        let checker = E1013UnionFieldAccess::default();

        let mut violations = Vec::new();
        for item in &syntax.items {
            violations.extend(checker.check_item(item, "test.rs").unwrap());
        }

        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_safe_field_access_passes() {
        let code = r#"
            struct MyStruct {
                field: i32,
            }

            fn get_field(s: &MyStruct) -> i32 {
                s.field
            }
        "#;

        let syntax = syn::parse_file(code).unwrap();
        let checker = E1013UnionFieldAccess::default();

        let mut violations = Vec::new();
        for item in &syntax.items {
            violations.extend(checker.check_item(item, "test.rs").unwrap());
        }

        // Should not detect any violations for safe struct field access
        assert_eq!(violations.len(), 0);
    }
}
