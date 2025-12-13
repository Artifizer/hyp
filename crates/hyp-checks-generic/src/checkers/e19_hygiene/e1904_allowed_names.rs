//! E1904: Allowed names and paths control
//!
//! Enforces project-specific naming and location rules for AST items.
//! Prevents unwanted patterns like DTOs outside API layers or wildcard imports in specific modules.

use crate::{checker::Checker, define_checker, violation::Violation};
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::path::Path;
use syn::{spanned::Spanned, visit::Visit};

/// AST item types that can be controlled
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub enum AstItemType {
    /// Wildcard - matches all item types
    All,
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

impl AstItemType {
    /// All valid item type names
    pub const VALID_VALUES: &'static [&'static str] = &[
        "*", "struct", "enum", "trait", "function", "const", "static", "type", "use", "mod", "impl",
    ];

    /// Check if this item type matches a specific type (handles wildcard)
    pub fn matches(&self, other: &AstItemType) -> bool {
        match self {
            AstItemType::All => true,
            _ => self == other,
        }
    }

    /// Parse from string
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "*" => Some(AstItemType::All),
            "struct" => Some(AstItemType::Struct),
            "enum" => Some(AstItemType::Enum),
            "trait" => Some(AstItemType::Trait),
            "function" | "fn" => Some(AstItemType::Function),
            "const" => Some(AstItemType::Const),
            "static" => Some(AstItemType::Static),
            "type" => Some(AstItemType::Type),
            "use" => Some(AstItemType::Use),
            "mod" | "module" => Some(AstItemType::Mod),
            "impl" => Some(AstItemType::Impl),
            _ => None,
        }
    }
}

impl<'de> Deserialize<'de> for AstItemType {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        AstItemType::from_str(&s).ok_or_else(|| {
            serde::de::Error::custom(format!(
                "Unknown item_type '{}'. Valid values are: {}.\n\
                 \n\
                 Examples:\n\
                 - item_types = [\"struct\"]        # Match struct definitions\n\
                 - item_types = [\"enum\", \"struct\"] # Match enums and structs\n\
                 - item_types = [\"*\"]             # Match ALL item types\n\
                 - item_types = [\"function\"]      # Match function definitions\n\
                 - item_types = [\"use\"]           # Match use statements",
                s,
                Self::VALID_VALUES.join(", ")
            ))
        })
    }
}

/// Reference type - whether we're defining, using, or referencing via use statement
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub enum ReferenceType {
    /// Item definition (where it's declared)
    Define,
    /// Item reference (where it's used in code)
    Refer,
    /// Use statement (import/re-export)
    Use,
}

impl ReferenceType {
    /// All valid reference type names
    pub const VALID_VALUES: &'static [&'static str] = &["define", "refer", "use"];

    /// Parse from string
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "define" | "definition" => Some(ReferenceType::Define),
            "refer" | "reference" => Some(ReferenceType::Refer),
            "use" | "import" => Some(ReferenceType::Use),
            _ => None,
        }
    }
}

impl<'de> Deserialize<'de> for ReferenceType {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        ReferenceType::from_str(&s).ok_or_else(|| {
            serde::de::Error::custom(format!(
                "Unknown reference_type '{}'. Valid values are: {}.\n\
                 \n\
                 Reference types explained:\n\
                 - \"define\" : Match where items are DEFINED (struct Foo {{}}, fn bar() {{}}, etc.)\n\
                 - \"refer\"  : Match where items are REFERENCED in code (not yet implemented)\n\
                 - \"use\"    : Match USE statements (use foo::bar, use std::*)\n\
                 \n\
                 Examples:\n\
                 - reference_type = \"define\"  # Match struct/enum/fn definitions\n\
                 - reference_type = \"use\"     # Match import statements",
                s,
                Self::VALID_VALUES.join(", ")
            ))
        })
    }
}

/// A single naming rule
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NamingRule {
    /// Whether this rule is enabled (default: true)
    #[serde(default = "default_enabled")]
    pub enabled: bool,
    /// Types of AST items this rule applies to (use "*" for all types)
    pub item_types: Vec<AstItemType>,
    /// Whether this rule applies to definitions, references, or use statements
    pub reference_type: ReferenceType,
    /// Regex patterns for item names (e.g., ".*DTO$" to match DTOs)
    pub name_patterns: Vec<String>,
    /// Regex patterns for allowed file paths (empty = allowed everywhere)
    #[serde(default)]
    pub allowed_paths: Vec<String>,
    /// Custom message template with placeholders: {type}, {name}, {path}, {allowed_paths}
    #[serde(default = "default_message")]
    pub message: String,
}

fn default_enabled() -> bool {
    true
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
    /// Checker for E1904: Allowed names and paths control
    E1904AllowedNames,
    code = "E1904",
    name = "Item name/location violates project rules",
    suggestions = "Move the item to an allowed location or rename it according to project conventions",
    target_items = [Struct, Enum, Trait, Function, Const, Static, Type, Use, Module, Impl],
    config_entry_name = "e1904_allowed_names",
    config = E1904Config {
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
    checker: &'a E1904AllowedNames,
}

impl<'a> NamingVisitor<'a> {
    fn check_rules(
        &mut self,
        item_type: AstItemType,
        ref_type: ReferenceType,
        name: &str,
        span: proc_macro2::Span,
    ) {
        let normalized_path = Path::new(self.file_path)
            .to_str()
            .unwrap_or(self.file_path)
            .replace('\\', "/");

        for rule in &self.checker.config.rules {
            // Skip disabled rules
            if !rule.enabled {
                continue;
            }

            // Check if this rule applies to this item type (supports wildcard "*")
            let type_matches = rule.item_types.iter().any(|t| t.matches(&item_type));
            if !type_matches {
                continue;
            }

            // Check if reference type matches
            if rule.reference_type != ref_type {
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

            // If allowed_paths is empty, the item is allowed everywhere
            if path_regexes.is_empty() {
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
        self.check_rules(
            AstItemType::Struct,
            ReferenceType::Define,
            &name,
            node.ident.span(),
        );
        syn::visit::visit_item_struct(self, node);
    }

    fn visit_item_enum(&mut self, node: &'a syn::ItemEnum) {
        let name = node.ident.to_string();
        self.check_rules(
            AstItemType::Enum,
            ReferenceType::Define,
            &name,
            node.ident.span(),
        );
        syn::visit::visit_item_enum(self, node);
    }

    fn visit_item_trait(&mut self, node: &'a syn::ItemTrait) {
        let name = node.ident.to_string();
        self.check_rules(
            AstItemType::Trait,
            ReferenceType::Define,
            &name,
            node.ident.span(),
        );
        syn::visit::visit_item_trait(self, node);
    }

    fn visit_item_fn(&mut self, node: &'a syn::ItemFn) {
        let name = node.sig.ident.to_string();
        self.check_rules(
            AstItemType::Function,
            ReferenceType::Define,
            &name,
            node.sig.ident.span(),
        );
        syn::visit::visit_item_fn(self, node);
    }

    fn visit_item_type(&mut self, node: &'a syn::ItemType) {
        let name = node.ident.to_string();
        self.check_rules(
            AstItemType::Type,
            ReferenceType::Define,
            &name,
            node.ident.span(),
        );
        syn::visit::visit_item_type(self, node);
    }

    fn visit_item_const(&mut self, node: &'a syn::ItemConst) {
        let name = node.ident.to_string();
        self.check_rules(
            AstItemType::Const,
            ReferenceType::Define,
            &name,
            node.ident.span(),
        );
        syn::visit::visit_item_const(self, node);
    }

    fn visit_item_static(&mut self, node: &'a syn::ItemStatic) {
        let name = node.ident.to_string();
        self.check_rules(
            AstItemType::Static,
            ReferenceType::Define,
            &name,
            node.ident.span(),
        );
        syn::visit::visit_item_static(self, node);
    }

    fn visit_item_mod(&mut self, node: &'a syn::ItemMod) {
        let name = node.ident.to_string();
        self.check_rules(
            AstItemType::Mod,
            ReferenceType::Define,
            &name,
            node.ident.span(),
        );
        syn::visit::visit_item_mod(self, node);
    }

    fn visit_item_use(&mut self, node: &'a syn::ItemUse) {
        // Extract use path as string (for pattern matching)
        let use_path = quote::quote!(#node).to_string();
        // Use statements use the special "use" reference type
        self.check_rules(AstItemType::Use, ReferenceType::Use, &use_path, node.span());
        syn::visit::visit_item_use(self, node);
    }

    fn visit_item_impl(&mut self, node: &'a syn::ItemImpl) {
        // For impl blocks, use the type name if available
        let name = quote::quote!(#node).to_string();
        self.check_rules(AstItemType::Impl, ReferenceType::Define, &name, node.span());
        syn::visit::visit_item_impl(self, node);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::AnalyzerConfig;

    /// Helper to create a rule with all required fields
    fn make_rule(
        item_types: Vec<AstItemType>,
        reference_type: ReferenceType,
        name_patterns: Vec<&str>,
        allowed_paths: Vec<&str>,
        message: &str,
    ) -> NamingRule {
        NamingRule {
            enabled: true,
            item_types,
            reference_type,
            name_patterns: name_patterns.into_iter().map(String::from).collect(),
            allowed_paths: allowed_paths.into_iter().map(String::from).collect(),
            message: message.to_string(),
        }
    }

    fn check_code_with_config(
        code: &str,
        rules: Vec<NamingRule>,
        file_path: &str,
    ) -> Vec<Violation> {
        let mut config = E1904Config::default();
        config.rules = rules;
        let checker = E1904AllowedNames { config };

        let file = syn::parse_file(code).expect("Failed to parse");
        let mut violations = Vec::new();
        for item in &file.items {
            violations.extend(checker.check_item(item, file_path).unwrap());
        }
        violations
    }

    // ========== Basic functionality tests ==========

    #[test]
    fn test_dto_in_wrong_location() {
        let code = r#"
            pub struct UserDTO {
                id: i32,
                name: String,
            }
        "#;

        let rules = vec![make_rule(
            vec![AstItemType::Struct],
            ReferenceType::Define,
            vec![".*DTO$"],
            vec!["^.*/api/.*\\.rs$"],
            "DTO in wrong location",
        )];

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

        let rules = vec![make_rule(
            vec![AstItemType::Struct],
            ReferenceType::Define,
            vec![".*DTO$"],
            vec!["^.*/api/.*\\.rs$"],
            "DTO in wrong location",
        )];

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

        let rules = vec![make_rule(
            vec![AstItemType::Struct],
            ReferenceType::Define,
            vec![".*DTO$"],
            vec!["^.*/api/.*\\.rs$"],
            "DTO in wrong location",
        )];

        let violations = check_code_with_config(code, rules, "src/models/user.rs");
        assert!(violations.is_empty());
    }

    #[test]
    fn test_multiple_patterns() {
        let code = r#"
            pub struct UserDTO { id: i32 }
            pub struct OrderRequest { id: i32 }
        "#;

        let rules = vec![make_rule(
            vec![AstItemType::Struct],
            ReferenceType::Define,
            vec![".*DTO$", ".*Request$"],
            vec!["^.*/api/.*\\.rs$"],
            "API type in wrong location",
        )];

        let violations = check_code_with_config(code, rules, "src/models/user.rs");
        assert_eq!(violations.len(), 2);
    }

    #[test]
    fn test_empty_allowed_paths_means_allowed_everywhere() {
        let code = r#"
            pub struct UserDTO { id: i32 }
        "#;

        let rules = vec![make_rule(
            vec![AstItemType::Struct],
            ReferenceType::Define,
            vec![".*DTO$"],
            vec![], // Empty = allowed everywhere
            "DTO in wrong location",
        )];

        let violations = check_code_with_config(code, rules, "src/models/user.rs");
        assert!(
            violations.is_empty(),
            "Empty allowed_paths should mean 'allowed everywhere'"
        );
    }

    // ========== Rule enabled flag tests ==========

    #[test]
    fn test_disabled_rule_is_skipped() {
        let code = r#"
            pub struct UserDTO { id: i32 }
        "#;

        let rules = vec![NamingRule {
            enabled: false, // Disabled
            item_types: vec![AstItemType::Struct],
            reference_type: ReferenceType::Define,
            name_patterns: vec![".*DTO$".to_string()],
            allowed_paths: vec!["^.*/api/.*\\.rs$".to_string()],
            message: "DTO in wrong location".to_string(),
        }];

        let violations = check_code_with_config(code, rules, "src/models/user.rs");
        assert!(
            violations.is_empty(),
            "Disabled rule should not produce violations"
        );
    }

    #[test]
    fn test_enabled_rule_is_checked() {
        let code = r#"
            pub struct UserDTO { id: i32 }
        "#;

        let rules = vec![NamingRule {
            enabled: true, // Explicitly enabled
            item_types: vec![AstItemType::Struct],
            reference_type: ReferenceType::Define,
            name_patterns: vec![".*DTO$".to_string()],
            allowed_paths: vec!["^.*/api/.*\\.rs$".to_string()],
            message: "DTO in wrong location".to_string(),
        }];

        let violations = check_code_with_config(code, rules, "src/models/user.rs");
        assert_eq!(violations.len(), 1, "Enabled rule should produce violation");
    }

    #[test]
    fn test_rule_enabled_by_default() {
        // Test that rules are enabled by default when loaded from TOML
        let toml = r#"
            [checkers.e1904_allowed_names]
            enabled = true

            [[checkers.e1904_allowed_names.rules]]
            item_types = ["struct"]
            reference_type = "define"
            name_patterns = [".*DTO$"]
            allowed_paths = ["^.*/api/.*$"]
            message = "DTO in wrong place"
        "#;

        let config = AnalyzerConfig::from_toml(toml).unwrap();
        let e1904_config: E1904Config = config
            .get_checker_config("e1904_allowed_names")
            .expect("Failed to load config");

        assert!(
            e1904_config.rules[0].enabled,
            "Rule should be enabled by default"
        );
    }

    #[test]
    fn test_rule_can_be_disabled_via_toml() {
        let toml = r#"
            [checkers.e1904_allowed_names]
            enabled = true

            [[checkers.e1904_allowed_names.rules]]
            enabled = false
            item_types = ["struct"]
            reference_type = "define"
            name_patterns = [".*DTO$"]
            allowed_paths = ["^.*/api/.*$"]
            message = "DTO in wrong place"
        "#;

        let config = AnalyzerConfig::from_toml(toml).unwrap();
        let e1904_config: E1904Config = config
            .get_checker_config("e1904_allowed_names")
            .expect("Failed to load config");

        assert!(
            !e1904_config.rules[0].enabled,
            "Rule should be disabled via TOML"
        );
    }

    // ========== Wildcard item_types ("*") tests ==========

    #[test]
    fn test_wildcard_item_type_matches_struct() {
        let code = r#"
            pub struct BadName { id: i32 }
        "#;

        let rules = vec![make_rule(
            vec![AstItemType::All], // Wildcard
            ReferenceType::Define,
            vec!["^Bad.*"],
            vec!["^.*/good/.*$"],
            "Bad naming",
        )];

        let violations = check_code_with_config(code, rules, "src/models/user.rs");
        assert_eq!(violations.len(), 1, "Wildcard should match struct");
    }

    #[test]
    fn test_wildcard_item_type_matches_enum() {
        let code = r#"
            pub enum BadState { A, B }
        "#;

        let rules = vec![make_rule(
            vec![AstItemType::All], // Wildcard
            ReferenceType::Define,
            vec!["^Bad.*"],
            vec!["^.*/good/.*$"],
            "Bad naming",
        )];

        let violations = check_code_with_config(code, rules, "src/models/user.rs");
        assert_eq!(violations.len(), 1, "Wildcard should match enum");
    }

    #[test]
    fn test_wildcard_item_type_matches_function() {
        let code = r#"
            fn bad_function() {}
        "#;

        let rules = vec![make_rule(
            vec![AstItemType::All], // Wildcard
            ReferenceType::Define,
            vec!["^bad_.*"],
            vec!["^.*/good/.*$"],
            "Bad naming",
        )];

        let violations = check_code_with_config(code, rules, "src/models/user.rs");
        assert_eq!(violations.len(), 1, "Wildcard should match function");
    }

    #[test]
    fn test_wildcard_in_toml() {
        let toml = r#"
            [checkers.e1904_allowed_names]
            enabled = true

            [[checkers.e1904_allowed_names.rules]]
            item_types = ["*"]
            reference_type = "define"
            name_patterns = ["^Bad.*"]
            allowed_paths = ["^.*/good/.*$"]
            message = "Bad naming"
        "#;

        let config = AnalyzerConfig::from_toml(toml).unwrap();
        let e1904_config: E1904Config = config
            .get_checker_config("e1904_allowed_names")
            .expect("Failed to load config");

        assert_eq!(e1904_config.rules[0].item_types, vec![AstItemType::All]);
    }

    #[test]
    fn test_wildcard_with_other_types() {
        let code = r#"
            pub struct BadStruct { id: i32 }
            pub enum BadEnum { A }
        "#;

        // Using wildcard should match both
        let rules = vec![make_rule(
            vec![AstItemType::All],
            ReferenceType::Define,
            vec!["^Bad.*"],
            vec!["^.*/good/.*$"],
            "Bad naming",
        )];

        let violations = check_code_with_config(code, rules, "src/models/user.rs");
        assert_eq!(
            violations.len(),
            2,
            "Wildcard should match multiple item types"
        );
    }

    // ========== Reference type "use" tests ==========

    #[test]
    fn test_use_reference_type_matches_use_statements() {
        let code = r#"
            use std::collections::HashMap;
        "#;

        let rules = vec![make_rule(
            vec![AstItemType::Use],
            ReferenceType::Use, // Use reference type
            vec![".*HashMap.*"],
            vec!["^.*/utils/.*$"],
            "HashMap should only be used in utils",
        )];

        let violations = check_code_with_config(code, rules, "src/models/user.rs");
        assert_eq!(
            violations.len(),
            1,
            "Use reference type should match use statements"
        );
    }

    #[test]
    fn test_use_reference_type_in_toml() {
        let toml = r#"
            [checkers.e1904_allowed_names]
            enabled = true

            [[checkers.e1904_allowed_names.rules]]
            item_types = ["use"]
            reference_type = "use"
            name_patterns = [".*sqlx.*\\*"]
            allowed_paths = ["^.*/db/.*$"]
            message = "Wildcard sqlx imports only in db module"
        "#;

        let config = AnalyzerConfig::from_toml(toml).unwrap();
        let e1904_config: E1904Config = config
            .get_checker_config("e1904_allowed_names")
            .expect("Failed to load config");

        assert_eq!(e1904_config.rules[0].reference_type, ReferenceType::Use);
    }

    #[test]
    fn test_define_reference_type_does_not_match_use_statements() {
        let code = r#"
            use std::collections::HashMap;
        "#;

        let rules = vec![make_rule(
            vec![AstItemType::Use],
            ReferenceType::Define, // Wrong reference type
            vec![".*HashMap.*"],
            vec!["^.*/utils/.*$"],
            "Should not match",
        )];

        let violations = check_code_with_config(code, rules, "src/models/user.rs");
        assert!(
            violations.is_empty(),
            "Define reference type should not match use statements"
        );
    }

    // ========== Config loading tests ==========

    #[test]
    fn test_config_loading_from_toml() {
        let toml = r#"
            [checkers.e1904_allowed_names]
            enabled = true
            severity = 3
            categories = ["compliance"]

            [[checkers.e1904_allowed_names.rules]]
            item_types = ["struct"]
            reference_type = "define"
            name_patterns = [".*DTO$"]
            allowed_paths = ["^.*/api/.*\\.rs$"]
            message = "DTO must be in api directory"
        "#;

        let config = AnalyzerConfig::from_toml(toml).unwrap();
        let e1904_config: E1904Config = config
            .get_checker_config("e1904_allowed_names")
            .expect("Failed to load E1904 config");

        assert!(e1904_config.enabled);
        assert_eq!(e1904_config.rules.len(), 1);
        assert_eq!(e1904_config.rules[0].item_types, vec![AstItemType::Struct]);
        assert_eq!(
            e1904_config.rules[0].name_patterns,
            vec![".*DTO$".to_string()]
        );
    }

    #[test]
    fn test_config_loading_with_multiple_rules() {
        let toml = r#"
            [checkers.e1904_allowed_names]
            enabled = true

            [[checkers.e1904_allowed_names.rules]]
            item_types = ["struct"]
            reference_type = "define"
            name_patterns = [".*DTO$"]
            allowed_paths = ["^.*/api/.*$"]
            message = "DTOs must be in api"

            [[checkers.e1904_allowed_names.rules]]
            item_types = ["enum"]
            reference_type = "define"
            name_patterns = [".*Error$"]
            allowed_paths = ["^.*/errors/.*$"]
            message = "Errors must be in errors directory"
        "#;

        let config = AnalyzerConfig::from_toml(toml).unwrap();
        let e1904_config: E1904Config = config
            .get_checker_config("e1904_allowed_names")
            .expect("Failed to load E1904 config");

        assert_eq!(e1904_config.rules.len(), 2);
        assert_eq!(e1904_config.rules[0].item_types, vec![AstItemType::Struct]);
        assert_eq!(e1904_config.rules[1].item_types, vec![AstItemType::Enum]);
    }

    #[test]
    fn test_config_with_severity_as_integer() {
        let toml = r#"
            [checkers.e1904_allowed_names]
            enabled = true
            severity = 2
        "#;

        let config = AnalyzerConfig::from_toml(toml).unwrap();
        let e1904_config: E1904Config = config
            .get_checker_config("e1904_allowed_names")
            .expect("Failed to load E1904 config");

        assert_eq!(e1904_config.severity, crate::config::SeverityLevel::Medium);
    }

    #[test]
    fn test_config_with_severity_as_string() {
        let toml = r#"
            [checkers.e1904_allowed_names]
            enabled = true
            severity = "low"
        "#;

        let config = AnalyzerConfig::from_toml(toml).unwrap();
        let e1904_config: E1904Config = config
            .get_checker_config("e1904_allowed_names")
            .expect("Failed to load E1904 config");

        assert_eq!(e1904_config.severity, crate::config::SeverityLevel::Low);
    }

    #[test]
    fn test_invalid_item_type_rejected() {
        let toml = r#"
            [checkers.e1904_allowed_names]
            enabled = true

            [[checkers.e1904_allowed_names.rules]]
            item_types = ["invalid_type"]
            reference_type = "define"
            name_patterns = [".*"]
            allowed_paths = [".*"]
        "#;

        let config = AnalyzerConfig::from_toml(toml).unwrap();
        let result: Result<E1904Config, _> = config.get_checker_config("e1904_allowed_names");

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("invalid_type"));
    }

    #[test]
    fn test_invalid_reference_type_rejected() {
        let toml = r#"
            [checkers.e1904_allowed_names]
            enabled = true

            [[checkers.e1904_allowed_names.rules]]
            item_types = ["struct"]
            reference_type = "invalid_ref"
            name_patterns = [".*"]
            allowed_paths = [".*"]
        "#;

        let config = AnalyzerConfig::from_toml(toml).unwrap();
        let result: Result<E1904Config, _> = config.get_checker_config("e1904_allowed_names");

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("invalid_ref"));
    }

    #[test]
    fn test_invalid_regex_handled_gracefully() {
        let code = r#"
            pub struct UserDTO { id: i32 }
        "#;

        let rules = vec![NamingRule {
            enabled: true,
            item_types: vec![AstItemType::Struct],
            reference_type: ReferenceType::Define,
            name_patterns: vec!["[invalid regex".to_string()], // Invalid regex
            allowed_paths: vec!["^.*/api/.*\\.rs$".to_string()],
            message: "Should not match".to_string(),
        }];

        // Invalid regex should be skipped gracefully, not cause a panic
        let violations = check_code_with_config(code, rules, "src/models/user.rs");
        assert!(violations.is_empty());
    }

    // ========== All item types tests ==========

    #[test]
    fn test_const_item_type() {
        let code = r#"
            const BAD_CONST: i32 = 42;
        "#;

        let rules = vec![make_rule(
            vec![AstItemType::Const],
            ReferenceType::Define,
            vec!["^BAD_.*"],
            vec!["^.*/good/.*$"],
            "Bad const naming",
        )];

        let violations = check_code_with_config(code, rules, "src/models/user.rs");
        assert_eq!(violations.len(), 1, "Should match const items");
    }

    #[test]
    fn test_static_item_type() {
        let code = r#"
            static BAD_STATIC: i32 = 42;
        "#;

        let rules = vec![make_rule(
            vec![AstItemType::Static],
            ReferenceType::Define,
            vec!["^BAD_.*"],
            vec!["^.*/good/.*$"],
            "Bad static naming",
        )];

        let violations = check_code_with_config(code, rules, "src/models/user.rs");
        assert_eq!(violations.len(), 1, "Should match static items");
    }

    #[test]
    fn test_mod_item_type() {
        let code = r#"
            mod bad_module {}
        "#;

        let rules = vec![make_rule(
            vec![AstItemType::Mod],
            ReferenceType::Define,
            vec!["^bad_.*"],
            vec!["^.*/good/.*$"],
            "Bad module naming",
        )];

        let violations = check_code_with_config(code, rules, "src/models/user.rs");
        assert_eq!(violations.len(), 1, "Should match mod items");
    }

    #[test]
    fn test_trait_item_type() {
        let code = r#"
            trait BadTrait {}
        "#;

        let rules = vec![make_rule(
            vec![AstItemType::Trait],
            ReferenceType::Define,
            vec!["^Bad.*"],
            vec!["^.*/good/.*$"],
            "Bad trait naming",
        )];

        let violations = check_code_with_config(code, rules, "src/models/user.rs");
        assert_eq!(violations.len(), 1, "Should match trait items");
    }
}
