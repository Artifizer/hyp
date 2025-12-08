//! E1204: Multiple traits with same method names
//!
//! Detects when multiple traits define methods with the same name, which requires
//! fully qualified syntax to disambiguate and reduces code readability.

use crate::{define_checker, violation::Violation};

use std::collections::HashMap;
use syn::visit::Visit;

define_checker! {
    /// Checker for E1204: Multiple traits with same method names
    E1204TraitMethodAmbiguity,
    code = "E1204",
    name = "Multiple traits with same method names",
    suggestions = "Use different method names in different traits, or accept that users need fully qualified syntax",
    target_items = [Module],
    config_entry_name = "e1204_trait_method_ambiguity",
    config = E1204Config {
        enabled: bool = true,
        severity: crate::config::SeverityLevel = crate::config::SeverityLevel::Medium,
        categories: Vec<crate::config::CheckerCategory> = vec![crate::config::CheckerCategory::Complexity],
    },
    check_item(self, item, file_path) {
        let mut visitor = TraitMethodVisitor {
            trait_methods: HashMap::new(),
            file_path,
        };

        // Visit the item to collect trait methods
        visitor.visit_item(item);

        let mut violations = Vec::new();

        // Find method names that appear in multiple traits
        let mut method_to_traits: HashMap<String, Vec<String>> = HashMap::new();
        for (trait_name, methods) in &visitor.trait_methods {
            for method in methods {
                method_to_traits
                    .entry(method.clone())
                    .or_default()
                    .push(trait_name.clone());
            }
        }

        for (method_name, traits) in method_to_traits {
            if traits.len() > 1 {
                violations.push(
                    Violation::new(
                        self.code(),
                        self.name(),
                        self.severity().into(),
                        format!(
                            "Method '{}' is defined in multiple traits: {}. This requires fully qualified syntax.",
                            method_name,
                            traits.join(", ")
                        ),
                        file_path,
                        1, // Line 1 since this is a module-level issue
                        1,
                    )
                    .with_suggestion(self.suggestions()),
                );
            }
        }

        Ok(violations)
    }
}

struct TraitMethodVisitor<'a> {
    trait_methods: HashMap<String, Vec<String>>,
    #[allow(dead_code)]
    file_path: &'a str,
}

impl<'ast> Visit<'ast> for TraitMethodVisitor<'_> {
    fn visit_item_trait(&mut self, node: &'ast syn::ItemTrait) {
        let trait_name = node.ident.to_string();
        let mut methods = Vec::new();

        for item in &node.items {
            if let syn::TraitItem::Fn(method) = item {
                methods.push(method.sig.ident.to_string());
            }
        }

        if !methods.is_empty() {
            self.trait_methods.insert(trait_name, methods);
        }

        syn::visit::visit_item_trait(self, node);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::checker::Checker;

    #[test]
    fn test_detects_ambiguous_method_names() {
        let code = r#"
            mod example {
                trait Display {
                    fn format(&self) -> String;
                }

                trait Serialize {
                    fn format(&self) -> String;
                }
            }
        "#;

        let syntax = syn::parse_file(code).unwrap();
        let checker = E1204TraitMethodAmbiguity::default();

        let mut violations = Vec::new();
        for item in &syntax.items {
            violations.extend(checker.check_item(item, "test.rs").unwrap());
        }

        assert_eq!(violations.len(), 1);
        assert!(violations[0].message.contains("format"));
        assert!(violations[0].message.contains("multiple traits"));
    }

    #[test]
    fn test_unique_method_names_pass() {
        let code = r#"
            mod example {
                trait Display {
                    fn display(&self) -> String;
                }

                trait Serialize {
                    fn serialize(&self) -> String;
                }
            }
        "#;

        let syntax = syn::parse_file(code).unwrap();
        let checker = E1204TraitMethodAmbiguity::default();

        let mut violations = Vec::new();
        for item in &syntax.items {
            violations.extend(checker.check_item(item, "test.rs").unwrap());
        }

        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_multiple_traits_with_same_method() {
        let code = r#"
            mod example {
                trait A {
                    fn process(&self);
                    fn unique_a(&self);
                }

                trait B {
                    fn process(&self);
                    fn unique_b(&self);
                }

                trait C {
                    fn process(&self);
                    fn unique_c(&self);
                }
            }
        "#;

        let syntax = syn::parse_file(code).unwrap();
        let checker = E1204TraitMethodAmbiguity::default();

        let mut violations = Vec::new();
        for item in &syntax.items {
            violations.extend(checker.check_item(item, "test.rs").unwrap());
        }

        assert_eq!(violations.len(), 1);
        assert!(violations[0].message.contains("process"));
    }
}
