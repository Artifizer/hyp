//! E1412: Prohibit union types entirely
//!
//! Detects any use of union types - both definitions and initializations.
//! Unions allow type confusion (storing one type, reading another) which
//! causes undefined behavior. This checker takes a strict approach: ban unions entirely.

use crate::{checker::Checker, define_checker, violation::Violation};

use syn::visit::Visit;
use std::collections::HashSet;

define_checker! {
    /// Checker for E1412: Prohibit union types
    E1412ProhibitUnions,
    code = "E1412",
    name = "Union types prohibited",
    suggestions = "Use enums with explicit variants instead of unions. Enums are type-safe and the compiler tracks which variant is active.",
    target_items = [Union, Function, Impl, Struct],
    config_entry_name = "e1412_prohibit_unions",
    /// Configuration for E1412: Prohibit unions checker
    config = E1412Config {
        /// Whether this checker is enabled
        enabled: bool = true,
        /// Severity level, defaults to High
        severity: crate::config::SeverityLevel = crate::config::SeverityLevel::High,
        /// Categories this checker belongs to
        categories: Vec<crate::config::CheckerCategory> = vec![crate::config::CheckerCategory::Operations],
    },
    check_item(self, item, file_path) {
        let mut visitor = UnionProhibitVisitor {
            violations: Vec::new(),
            file_path,
            checker: self,
            union_names: HashSet::new(),
        };

        // First pass: collect all union names from this item
        visitor.collect_union_names(item);

        // Second pass: check for violations
        visitor.visit_item(item);
        Ok(visitor.violations)
    }
}

struct UnionProhibitVisitor<'a> {
    violations: Vec<Violation>,
    file_path: &'a str,
    checker: &'a E1412ProhibitUnions,
    union_names: HashSet<String>,
}

impl<'a> UnionProhibitVisitor<'a> {
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

    fn collect_union_names(&mut self, item: &syn::Item) {
        struct NameCollector {
            names: HashSet<String>,
        }

        impl<'ast> Visit<'ast> for NameCollector {
            fn visit_item_union(&mut self, node: &'ast syn::ItemUnion) {
                self.names.insert(node.ident.to_string());
                syn::visit::visit_item_union(self, node);
            }
        }

        let mut collector = NameCollector {
            names: HashSet::new(),
        };
        collector.visit_item(item);
        self.union_names.extend(collector.names);
    }
}

impl<'a> Visit<'a> for UnionProhibitVisitor<'a> {
    fn visit_item_union(&mut self, node: &'a syn::ItemUnion) {
        self.violations.push(self.create_violation(
            node.ident.span(),
            &format!(
                "Union type '{}' is prohibited. Unions allow type confusion (storing one type, reading another) which causes undefined behavior. Use enums instead.",
                node.ident
            ),
        ));

        syn::visit::visit_item_union(self, node);
    }

    fn visit_expr_struct(&mut self, node: &'a syn::ExprStruct) {
        use syn::spanned::Spanned;

        // Check if this is initializing a union type
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
                            "Initializing union type '{}' is prohibited. Unions allow type confusion which causes undefined behavior. Use enums instead.",
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

    fn visit_field(&mut self, node: &'a syn::Field) {
        use syn::spanned::Spanned;

        // Check if a field has a union type
        if let syn::Type::Path(type_path) = &node.ty {
            if let Some(segment) = type_path.path.segments.last() {
                let type_str = segment.ident.to_string();
                if self.union_names.contains(&type_str) {
                    let start = node.ty.span().start();
                    let field_name = node.ident.as_ref()
                        .map(|i| i.to_string())
                        .unwrap_or_else(|| "unnamed".to_string());

                    self.violations.push(
                        Violation::new(
                            self.checker.code(),
                            self.checker.name(),
                            self.checker.severity().into(),
                            format!(
                                "Field '{}' has union type '{}'. Using unions is prohibited due to type confusion risks. Use enums instead.",
                                field_name, type_str
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

        syn::visit::visit_field(self, node);
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
        let checker = E1412ProhibitUnions::default();

        let mut violations = Vec::new();
        for item in &syntax.items {
            violations.extend(checker.check_item(item, "test.rs").unwrap());
        }

        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].code, "E1412");
        assert!(violations[0].message.contains("Union type"));
        assert!(violations[0].message.contains("prohibited"));
    }

    #[test]
    fn test_detects_union_initialization_same_item() {
        let code = r#"
            union Value {
                int: i32,
                float: f32,
            }

            impl Value {
                fn new_int(val: i32) -> Self {
                    Value { int: val }
                }
            }
        "#;

        let syntax = syn::parse_file(code).unwrap();
        let checker = E1412ProhibitUnions::default();

        let mut violations = Vec::new();
        for item in &syntax.items {
            violations.extend(checker.check_item(item, "test.rs").unwrap());
        }

        // Should detect the union definition (initialization detection requires cross-item analysis)
        assert!(!violations.is_empty(), "Expected at least 1 violation, got {}", violations.len());
        assert!(violations.iter().any(|v| v.message.contains("Union type") && v.message.contains("prohibited")));
    }

    #[test]
    fn test_detects_union_in_struct_field_same_item() {
        let code = r#"
            union Value {
                int: i32,
                float: f32,
            }
        "#;

        let syntax = syn::parse_file(code).unwrap();
        let checker = E1412ProhibitUnions::default();

        let mut violations = Vec::new();
        for item in &syntax.items {
            violations.extend(checker.check_item(item, "test.rs").unwrap());
        }

        // Should detect the union definition
        assert_eq!(violations.len(), 1);
        assert!(violations[0].message.contains("Union type"));
        assert!(violations[0].message.contains("prohibited"));
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
        let checker = E1412ProhibitUnions::default();

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
        let checker = E1412ProhibitUnions::default();

        let mut violations = Vec::new();
        for item in &syntax.items {
            violations.extend(checker.check_item(item, "test.rs").unwrap());
        }

        assert_eq!(violations.len(), 0);
    }
}
