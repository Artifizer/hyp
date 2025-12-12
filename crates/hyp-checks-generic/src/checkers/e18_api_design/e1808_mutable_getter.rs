//! E1808: Mutable getter
//!
//! Detects getter methods that return mutable references, which can
//! break encapsulation and make invariants hard to maintain.

use crate::{define_checker, violation::Violation};

define_checker! {
    /// Checker for E1808: Mutable getter
    E1808MutableGetter,
    code = "E1808",
    name = "Mutable getter",
    suggestions = "Consider returning an immutable reference or provide a setter method instead",
    target_items = [Impl],
    config_entry_name = "e1808_mutable_getter",
    config = E1808Config {
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

                    // Check if it looks like a getter (get_*, *_mut, or just field name)
                    let is_getter = method_name.starts_with("get_")
                        || method_name.ends_with("_mut")
                        || (method.sig.inputs.len() == 1
                            && matches!(method.sig.inputs.first(), Some(syn::FnArg::Receiver(_))));

                    if is_getter {
                        // Check if return type is &mut T
                        if let syn::ReturnType::Type(_, ty) = &method.sig.output {
                            if is_mut_reference(ty) {
                                let start = method.sig.ident.span().start();
                                violations.push(
                                    Violation::new(
                                        self.code(),
                                        self.name(),
                                        self.severity().into(),
                                        format!(
                                            "Getter '{}' returns a mutable reference, which can break encapsulation.",
                                            method_name
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

        Ok(violations)
    }
}

fn is_mut_reference(ty: &syn::Type) -> bool {
    if let syn::Type::Reference(ref_type) = ty {
        return ref_type.mutability.is_some();
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::checker::Checker;

    #[test]
    fn test_detects_mutable_getter() {
        let code = r#"
            struct MyStruct {
                data: Vec<i32>,
            }

            impl MyStruct {
                fn get_data(&mut self) -> &mut Vec<i32> {
                    &mut self.data
                }
            }
        "#;

        let syntax = syn::parse_file(code).unwrap();
        let checker = E1808MutableGetter::default();

        let mut violations = Vec::new();
        for item in &syntax.items {
            violations.extend(checker.check_item(item, "test.rs").unwrap());
        }

        assert_eq!(violations.len(), 1);
    }

    #[test]
    fn test_immutable_getter_passes() {
        let code = r#"
            struct MyStruct {
                data: Vec<i32>,
            }

            impl MyStruct {
                fn get_data(&self) -> &Vec<i32> {
                    &self.data
                }
            }
        "#;

        let syntax = syn::parse_file(code).unwrap();
        let checker = E1808MutableGetter::default();

        let mut violations = Vec::new();
        for item in &syntax.items {
            violations.extend(checker.check_item(item, "test.rs").unwrap());
        }

        assert_eq!(violations.len(), 0);
    }
}
