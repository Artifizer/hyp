//! E1807: Non-idiomatic builder
//!
//! Detects builder patterns that don't follow Rust idioms,
//! such as builders that don't return Self or use &mut self incorrectly.

use crate::{checker::Checker, define_checker, violation::Violation};

use syn::visit::Visit;

define_checker! {
    /// Checker for E1807: Non-idiomatic builder
    E1807NonIdiomaticBuilder,
    code = "E1807",
    name = "Non-idiomatic builder",
    suggestions = "Builder methods should take self by value and return Self for chaining",
    target_items = [Impl],
    config_entry_name = "e1807_non_idiomatic_builder",
    config = E1807Config {
        enabled: bool = true,
        severity: crate::config::SeverityLevel = crate::config::SeverityLevel::Low,
        categories: Vec<crate::config::CheckerCategory> = vec![crate::config::CheckerCategory::Complexity],
    },
    check_item(self, item, file_path) {
        let mut visitor = BuilderVisitor {
            violations: Vec::new(),
            file_path,
            checker: self,
        };
        visitor.visit_item(item);
        Ok(visitor.violations)
    }
}

struct BuilderVisitor<'a> {
    violations: Vec<Violation>,
    file_path: &'a str,
    checker: &'a E1807NonIdiomaticBuilder,
}

impl<'a> Visit<'a> for BuilderVisitor<'a> {
    fn visit_item_impl(&mut self, node: &'a syn::ItemImpl) {
        // Check if this looks like a builder (type name ends with "Builder")
        let type_name = if let syn::Type::Path(path) = &*node.self_ty {
            path.path
                .segments
                .last()
                .map(|s| s.ident.to_string())
                .unwrap_or_default()
        } else {
            return;
        };

        if !type_name.ends_with("Builder") {
            return;
        }

        // Check builder methods
        for item in &node.items {
            if let syn::ImplItem::Fn(method) = item {
                let method_name = method.sig.ident.to_string();

                // Skip build() and new() methods
                if method_name == "build" || method_name == "new" {
                    continue;
                }

                // Check if method takes &mut self instead of self
                if let Some(syn::FnArg::Receiver(recv)) = method.sig.inputs.first() {
                    if recv.mutability.is_some() && recv.reference.is_some() {
                        let start = method.sig.ident.span().start();
                        self.violations.push(
                            Violation::new(
                                self.checker.code(),
                                self.checker.name(),
                                self.checker.severity().into(),
                                format!(
                                    "Builder method '{}' takes &mut self. Consider taking self by value for better ergonomics.",
                                    method_name
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

        syn::visit::visit_item_impl(self, node);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detects_mut_self_builder() {
        let code = r#"
            struct ConfigBuilder {
                value: i32,
            }

            impl ConfigBuilder {
                fn with_value(&mut self, value: i32) -> &mut Self {
                    self.value = value;
                    self
                }
            }
        "#;

        let syntax = syn::parse_file(code).unwrap();
        let checker = E1807NonIdiomaticBuilder::default();

        let mut violations = Vec::new();
        for item in &syntax.items {
            violations.extend(checker.check_item(item, "test.rs").unwrap());
        }

        assert_eq!(violations.len(), 1);
    }

    #[test]
    fn test_idiomatic_builder_passes() {
        let code = r#"
            struct ConfigBuilder {
                value: i32,
            }

            impl ConfigBuilder {
                fn with_value(mut self, value: i32) -> Self {
                    self.value = value;
                    self
                }
            }
        "#;

        let syntax = syn::parse_file(code).unwrap();
        let checker = E1807NonIdiomaticBuilder::default();

        let mut violations = Vec::new();
        for item in &syntax.items {
            violations.extend(checker.check_item(item, "test.rs").unwrap());
        }

        assert_eq!(violations.len(), 0);
    }
}
