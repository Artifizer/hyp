//! E1210: Recursive type definitions
//!
//! Detects recursive type definitions (enums, structs, traits) which can be
//! confusing and require careful handling with Box or other indirection.

use crate::{define_checker, violation::Violation};

use syn::visit::Visit;

define_checker! {
    /// Checker for E1210: Recursive type definitions
    E1210RecursiveTypes,
    code = "E1210",
    name = "Recursive type definitions",
    suggestions = "Document recursive structures clearly, use Box/Rc for indirection, consider if non-recursive design would work",
    target_items = [Struct, Enum, Trait],
    config_entry_name = "e1210_recursive_types",
    config = E1210Config {
        enabled: bool = true,
        severity: crate::config::SeverityLevel = crate::config::SeverityLevel::High,
        categories: Vec<crate::config::CheckerCategory> = vec![crate::config::CheckerCategory::Complexity],
    },
    check_item(self, item, file_path) {
        let mut violations = Vec::new();

        match item {
            syn::Item::Enum(enum_item) => {
                let type_name = enum_item.ident.to_string();
                let mut visitor = RecursiveTypeVisitor {
                    type_name: &type_name,
                    is_recursive: false,
                };

                for variant in &enum_item.variants {
                    visitor.visit_variant(variant);
                }

                if visitor.is_recursive {
                    let start = enum_item.ident.span().start();
                    violations.push(
                        Violation::new(
                            self.code(),
                            self.name(),
                            self.severity().into(),
                            format!(
                                "Enum '{}' is recursive (references itself). Ensure proper indirection with Box/Rc.",
                                type_name
                            ),
                            file_path,
                            start.line,
                            start.column + 1,
                        )
                        .with_suggestion(self.suggestions()),
                    );
                }
            }
            syn::Item::Struct(struct_item) => {
                let type_name = struct_item.ident.to_string();
                let mut visitor = RecursiveTypeVisitor {
                    type_name: &type_name,
                    is_recursive: false,
                };

                visitor.visit_fields(&struct_item.fields);

                if visitor.is_recursive {
                    let start = struct_item.ident.span().start();
                    violations.push(
                        Violation::new(
                            self.code(),
                            self.name(),
                            self.severity().into(),
                            format!(
                                "Struct '{}' is recursive (references itself). Ensure proper indirection with Box/Rc.",
                                type_name
                            ),
                            file_path,
                            start.line,
                            start.column + 1,
                        )
                        .with_suggestion(self.suggestions()),
                    );
                }
            }
            syn::Item::Trait(trait_item) => {
                let type_name = trait_item.ident.to_string();

                // Check for recursive associated types
                for item in &trait_item.items {
                    if let syn::TraitItem::Type(assoc_type) = item {
                        for bound in &assoc_type.bounds {
                            if let syn::TypeParamBound::Trait(trait_bound) = bound {
                                let bound_name = trait_bound.path.segments.iter()
                                    .map(|s| s.ident.to_string())
                                    .collect::<Vec<_>>()
                                    .join("::");
                                if bound_name == type_name {
                                    let start = trait_item.ident.span().start();
                                    violations.push(
                                        Violation::new(
                                            self.code(),
                                            self.name(),
                                            self.severity().into(),
                                            format!(
                                                "Trait '{}' has recursive associated type constraint. This can be hard to reason about.",
                                                type_name
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
            }
            _ => {}
        }

        Ok(violations)
    }
}

struct RecursiveTypeVisitor<'a> {
    type_name: &'a str,
    is_recursive: bool,
}

impl<'ast> Visit<'ast> for RecursiveTypeVisitor<'_> {
    fn visit_type(&mut self, ty: &'ast syn::Type) {
        if let syn::Type::Path(type_path) = ty {
            // Check if this type references our type name
            if let Some(segment) = type_path.path.segments.last() {
                if segment.ident == self.type_name {
                    self.is_recursive = true;
                }
                // Also check generic arguments
                if let syn::PathArguments::AngleBracketed(args) = &segment.arguments {
                    for arg in &args.args {
                        if let syn::GenericArgument::Type(inner_ty) = arg {
                            self.visit_type(inner_ty);
                        }
                    }
                }
            }
        }
        syn::visit::visit_type(self, ty);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::checker::Checker;

    #[test]
    fn test_detects_recursive_enum() {
        let code = r#"
            enum List<T> {
                Cons(T, Box<List<T>>),
                Nil,
            }
        "#;

        let syntax = syn::parse_file(code).unwrap();
        let checker = E1210RecursiveTypes::default();

        let mut violations = Vec::new();
        for item in &syntax.items {
            violations.extend(checker.check_item(item, "test.rs").unwrap());
        }

        assert_eq!(violations.len(), 1);
        assert!(violations[0].message.contains("List"));
        assert!(violations[0].message.contains("recursive"));
    }

    #[test]
    fn test_detects_recursive_struct() {
        let code = r#"
            struct Tree<T> {
                value: T,
                left: Option<Box<Tree<T>>>,
                right: Option<Box<Tree<T>>>,
            }
        "#;

        let syntax = syn::parse_file(code).unwrap();
        let checker = E1210RecursiveTypes::default();

        let mut violations = Vec::new();
        for item in &syntax.items {
            violations.extend(checker.check_item(item, "test.rs").unwrap());
        }

        assert_eq!(violations.len(), 1);
        assert!(violations[0].message.contains("Tree"));
    }

    #[test]
    fn test_detects_recursive_trait() {
        let code = r#"
            trait Recursive {
                type Next: Recursive;
                fn next(&self) -> Self::Next;
            }
        "#;

        let syntax = syn::parse_file(code).unwrap();
        let checker = E1210RecursiveTypes::default();

        let mut violations = Vec::new();
        for item in &syntax.items {
            violations.extend(checker.check_item(item, "test.rs").unwrap());
        }

        assert_eq!(violations.len(), 1);
        assert!(violations[0].message.contains("Recursive"));
    }

    #[test]
    fn test_non_recursive_types_pass() {
        let code = r#"
            struct Point {
                x: i32,
                y: i32,
            }

            enum Color {
                Red,
                Green,
                Blue,
            }
        "#;

        let syntax = syn::parse_file(code).unwrap();
        let checker = E1210RecursiveTypes::default();

        let mut violations = Vec::new();
        for item in &syntax.items {
            violations.extend(checker.check_item(item, "test.rs").unwrap());
        }

        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_nested_list_recursive() {
        let code = r#"
            enum NestedList<T> {
                Value(T),
                List(Vec<NestedList<T>>),
                Tree(Box<NestedList<T>>, Box<NestedList<T>>),
            }
        "#;

        let syntax = syn::parse_file(code).unwrap();
        let checker = E1210RecursiveTypes::default();

        let mut violations = Vec::new();
        for item in &syntax.items {
            violations.extend(checker.check_item(item, "test.rs").unwrap());
        }

        assert_eq!(violations.len(), 1);
    }
}
