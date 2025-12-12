//! E1901: Allowed names and paths control
//!
//! Enforces project-specific naming and location rules for AST items.
//! Prevents unwanted patterns like DTOs outside API layers or wildcard imports in specific modules.

use crate::{checker::Checker, define_checker, violation::Violation};
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::path::Path;
use syn::{spanned::Spanned, visit::Visit};

/// AST item types that can be controlled
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum AstItemType {
    /// Struct definition
    Struct,
    /// Enum definition
    Enum,
    /// Trait definition
    Trait,
    /// Function definition
    Function,
    /// Const item
    Const,
    /// Static item
    Static,
    /// Type alias
    Type,
    /// Use statement
    Use,
    /// Module definition
    Mod,
    /// Impl block
    Impl,
}

/// Reference type - whether we're defining or using an item
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ReferenceType {
    /// Item definition (where it's declared)
    Define,
    /// Item reference (where it's used)
    Refer,
}

/// A single naming rule
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NamingRule {
    /// Types of AST items this rule applies to
    pub item_types: Vec<AstItemType>,
    /// Whether this rule applies to definitions or references
    pub reference_type: ReferenceType,
    /// Regex patterns for item names (e.g., ".*DTO$" to match DTOs)
    pub name_patterns: Vec<String>,
    /// Regex patterns for allowed file paths
    pub allowed_paths: Vec<String>,
    /// Custom message template with placeholders: {type}, {name}, {path}, {allowed_paths}
    #[serde(default = "default_message")]
    pub message: String,
}

fn default_message() -> String {
    "{type} '{name}' in {path} is only allowed in: {allowed_paths}".to_string()
}

impl NamingRule {
    fn compile_patterns(&self) -> Result<(Vec<Regex>, Vec<Regex>), String> {
        let name_regexes: Result<Vec<_>, _> = self
            .name_patterns
            .iter()
            .map(|p| Regex::new(p).map_err(|e| format!("Invalid name pattern '{}': {}", p, e)))
            .collect();

        let path_regexes: Result<Vec<_>, _> = self
            .allowed_paths
            .iter()
            .map(|p| Regex::new(p).map_err(|e| format!("Invalid path pattern '{}': {}", p, e)))
            .collect();

        Ok((name_regexes?, path_regexes?))
    }
}

define_checker! {
    /// Checker for E1901: Allowed names and paths control
    E1901AllowedNames,
    code = "E1901",
    name = "Item name/location violates project rules",
    suggestions = "Move the item to an allowed location or rename it according to project conventions",
    target_items = [Struct, Enum, Trait, Function, Const, Static, Type, Use, Module, Impl],
    config_entry_name = "e1901_allowed_names",
    config = E1901Config {
        enabled: bool = true,
        severity: crate::config::SeverityLevel = crate::config::SeverityLevel::High,
        categories: Vec<crate::config::CheckerCategory> = vec![crate::config::CheckerCategory::Compliance],
        /// Project-specific naming rules (empty by default - configure in Hyp.toml)
        rules: Vec<NamingRule> = vec![],
    },
    check_item(self, item, file_path) {
        let mut visitor = NamingVisitor {
            violations: Vec::new(),
            file_path,
            checker: self,
        };
        visitor.visit_item(item);
        Ok(visitor.violations)
    }
}

struct NamingVisitor<'a> {
    violations: Vec<Violation>,
    file_path: &'a str,
    checker: &'a E1901AllowedNames,
}

impl<'a> NamingVisitor<'a> {
    fn check_rules(&mut self, item_type: AstItemType, name: &str, span: proc_macro2::Span) {
        let normalized_path = Path::new(self.file_path)
            .to_str()
            .unwrap_or(self.file_path)
            .replace('\\', "/");

        for rule in &self.checker.config.rules {
            // Check if this rule applies to this item type
            if !rule.item_types.contains(&item_type) {
                continue;
            }

            // Only check definitions for now (references would need more complex analysis)
            if rule.reference_type != ReferenceType::Define {
                continue;
            }

            // Compile patterns
            let Ok((name_regexes, path_regexes)) = rule.compile_patterns() else {
                continue;
            };

            // Check if name matches any pattern
            let name_matches = name_regexes.iter().any(|re| re.is_match(name));
            if !name_matches {
                continue;
            }

            // Check if path is allowed
            let path_allowed = path_regexes.iter().any(|re| re.is_match(&normalized_path));
            if path_allowed {
                continue;
            }

            // Violation found - format message
            let message = rule
                .message
                .replace("{type}", &format!("{:?}", item_type))
                .replace("{name}", name)
                .replace("{path}", &normalized_path)
                .replace("{allowed_paths}", &rule.allowed_paths.join(", "));

            self.violations.push(
                Violation::new(
                    self.checker.code(),
                    self.checker.name(),
                    self.checker.severity().into(),
                    &message,
                    self.file_path,
                    span.start().line,
                    span.start().column + 1,
                )
                .with_suggestion(self.checker.suggestions()),
            );
        }
    }
}

impl<'a> Visit<'a> for NamingVisitor<'a> {
    fn visit_item_struct(&mut self, node: &'a syn::ItemStruct) {
        let name = node.ident.to_string();
        self.check_rules(AstItemType::Struct, &name, node.ident.span());
        syn::visit::visit_item_struct(self, node);
    }

    fn visit_item_enum(&mut self, node: &'a syn::ItemEnum) {
        let name = node.ident.to_string();
        self.check_rules(AstItemType::Enum, &name, node.ident.span());
        syn::visit::visit_item_enum(self, node);
    }

    fn visit_item_trait(&mut self, node: &'a syn::ItemTrait) {
        let name = node.ident.to_string();
        self.check_rules(AstItemType::Trait, &name, node.ident.span());
        syn::visit::visit_item_trait(self, node);
    }

    fn visit_item_fn(&mut self, node: &'a syn::ItemFn) {
        let name = node.sig.ident.to_string();
        self.check_rules(AstItemType::Function, &name, node.sig.ident.span());
        syn::visit::visit_item_fn(self, node);
    }

    fn visit_item_type(&mut self, node: &'a syn::ItemType) {
        let name = node.ident.to_string();
        self.check_rules(AstItemType::Type, &name, node.ident.span());
        syn::visit::visit_item_type(self, node);
    }

    fn visit_item_use(&mut self, node: &'a syn::ItemUse) {
        // Extract use path as string
        let use_path = quote::quote!(#node).to_string();
        self.check_rules(AstItemType::Use, &use_path, node.span());
        syn::visit::visit_item_use(self, node);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn check_code_with_config(code: &str, rules: Vec<NamingRule>, file_path: &str) -> Vec<Violation> {
        let mut config = E1901Config::default();
        config.rules = rules;
        let checker = E1901AllowedNames { config };

        let file = syn::parse_file(code).expect("Failed to parse");
        let mut violations = Vec::new();
        for item in &file.items {
            violations.extend(checker.check_item(item, file_path).unwrap());
        }
        violations
    }

    #[test]
    fn test_dto_in_wrong_location() {
        let code = r#"
            pub struct UserDTO {
                id: i32,
                name: String,
            }
        "#;

        let rules = vec![NamingRule {
            item_types: vec![AstItemType::Struct],
            reference_type: ReferenceType::Define,
            name_patterns: vec![".*DTO$".to_string()],
            allowed_paths: vec!["^.*/api/.*\\.rs$".to_string()],
            message: "DTO in wrong location".to_string(),
        }];

        let violations = check_code_with_config(code, rules, "src/models/user.rs");
        assert_eq!(violations.len(), 1);
        assert!(violations[0].message.contains("DTO in wrong location"));
    }

    #[test]
    fn test_dto_in_correct_location() {
        let code = r#"
            pub struct UserDTO {
                id: i32,
                name: String,
            }
        "#;

        let rules = vec![NamingRule {
            item_types: vec![AstItemType::Struct],
            reference_type: ReferenceType::Define,
            name_patterns: vec![".*DTO$".to_string()],
            allowed_paths: vec!["^.*/api/.*\\.rs$".to_string()],
            message: "DTO in wrong location".to_string(),
        }];

        let violations = check_code_with_config(code, rules, "src/api/user.rs");
        assert!(violations.is_empty());
    }

    #[test]
    fn test_non_dto_struct_not_flagged() {
        let code = r#"
            pub struct User {
                id: i32,
                name: String,
            }
        "#;

        let rules = vec![NamingRule {
            item_types: vec![AstItemType::Struct],
            reference_type: ReferenceType::Define,
            name_patterns: vec![".*DTO$".to_string()],
            allowed_paths: vec!["^.*/api/.*\\.rs$".to_string()],
            message: "DTO in wrong location".to_string(),
        }];

        let violations = check_code_with_config(code, rules, "src/models/user.rs");
        assert!(violations.is_empty());
    }

    #[test]
    fn test_multiple_patterns() {
        let code = r#"
            pub struct UserDTO { id: i32 }
            pub struct OrderRequest { id: i32 }
        "#;

        let rules = vec![NamingRule {
            item_types: vec![AstItemType::Struct],
            reference_type: ReferenceType::Define,
            name_patterns: vec![".*DTO$".to_string(), ".*Request$".to_string()],
            allowed_paths: vec!["^.*/api/.*\\.rs$".to_string()],
            message: "API type in wrong location".to_string(),
        }];

        let violations = check_code_with_config(code, rules, "src/models/user.rs");
        assert_eq!(violations.len(), 2);
    }
}
