//! E1605: Rc cycle memory leak
//!
//! Detects patterns that may create reference cycles with Rc/RefCell combinations,
//! which can cause memory leaks because the reference count never reaches zero.

use crate::{define_checker, violation::Violation};

use syn::visit::Visit;

define_checker! {
    /// Checker for E1605: Rc cycle memory leak
    E1605RcCycle,
    code = "E1605",
    name = "Potential Rc cycle memory leak",
    suggestions = "Use Weak references to break cycles, or use arena allocators for graph structures",
    target_items = [Struct],
    config_entry_name = "e1605_rc_cycle",
    config = E1605Config {
        enabled: bool = true,
        severity: crate::config::SeverityLevel = crate::config::SeverityLevel::Medium,
        categories: Vec<crate::config::CheckerCategory> = vec![crate::config::CheckerCategory::Complexity],
    },
    check_item(self, item, file_path) {
        let mut violations = Vec::new();

        if let syn::Item::Struct(struct_item) = item {
            let struct_name = struct_item.ident.to_string();
            let mut visitor = RcFieldVisitor {
                struct_name: &struct_name,
                has_rc_self_reference: false,
            };

            visitor.visit_item_struct(struct_item);

            if visitor.has_rc_self_reference {
                let start = struct_item.ident.span().start();
                violations.push(
                    Violation::new(
                        self.code(),
                        self.name(),
                        self.severity().into(),
                        format!(
                            "Struct '{}' has Rc/RefCell field that references itself. This can create reference cycles and memory leaks.",
                            struct_name
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

struct RcFieldVisitor<'a> {
    struct_name: &'a str,
    has_rc_self_reference: bool,
}

impl<'ast> Visit<'ast> for RcFieldVisitor<'_> {
    fn visit_type(&mut self, ty: &'ast syn::Type) {
        if let syn::Type::Path(type_path) = ty {
            // Check for Rc<...> or RefCell<...> containing the struct itself
            for segment in &type_path.path.segments {
                let name = segment.ident.to_string();
                if name == "Rc" || name == "Arc" || name == "RefCell" {
                    // Check if the inner type references our struct
                    if let syn::PathArguments::AngleBracketed(args) = &segment.arguments {
                        for arg in &args.args {
                            if let syn::GenericArgument::Type(inner_ty) = arg {
                                if type_contains_name(inner_ty, self.struct_name) {
                                    self.has_rc_self_reference = true;
                                }
                            }
                        }
                    }
                }
            }
        }
        syn::visit::visit_type(self, ty);
    }
}

fn type_contains_name(ty: &syn::Type, name: &str) -> bool {
    if let syn::Type::Path(type_path) = ty {
        for segment in &type_path.path.segments {
            if segment.ident == name {
                return true;
            }
            if let syn::PathArguments::AngleBracketed(args) = &segment.arguments {
                for arg in &args.args {
                    if let syn::GenericArgument::Type(inner_ty) = arg {
                        if type_contains_name(inner_ty, name) {
                            return true;
                        }
                    }
                }
            }
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::checker::Checker;

    #[test]
    fn test_detects_rc_self_reference() {
        let code = r#"
            use std::rc::Rc;
            use std::cell::RefCell;

            struct Node {
                next: Option<Rc<RefCell<Node>>>,
            }
        "#;

        let syntax = syn::parse_file(code).unwrap();
        let checker = E1605RcCycle::default();

        let mut violations = Vec::new();
        for item in &syntax.items {
            violations.extend(checker.check_item(item, "test.rs").unwrap());
        }

        assert_eq!(violations.len(), 1);
    }

    #[test]
    fn test_no_self_reference_passes() {
        let code = r#"
            use std::rc::Rc;

            struct Node {
                value: i32,
                data: Rc<String>,
            }
        "#;

        let syntax = syn::parse_file(code).unwrap();
        let checker = E1605RcCycle::default();

        let mut violations = Vec::new();
        for item in &syntax.items {
            violations.extend(checker.check_item(item, "test.rs").unwrap());
        }

        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_weak_reference_would_be_better() {
        let code = r#"
            use std::rc::Rc;
            use std::cell::RefCell;

            struct TreeNode {
                children: Vec<Rc<RefCell<TreeNode>>>,
            }
        "#;

        let syntax = syn::parse_file(code).unwrap();
        let checker = E1605RcCycle::default();

        let mut violations = Vec::new();
        for item in &syntax.items {
            violations.extend(checker.check_item(item, "test.rs").unwrap());
        }

        assert_eq!(violations.len(), 1);
    }
}
