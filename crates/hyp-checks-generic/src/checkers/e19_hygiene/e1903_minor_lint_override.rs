//! E1903: Minor Lint Override Detection
//!
//! Detects when code uses `#[allow(...)]` attributes to override minor/stylistic Clippy lints.
//! While these may have legitimate overrides, tracking them helps maintain code consistency.

use crate::{checker::Checker, define_checker, violation::Violation};
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::path::Path;
use syn::{spanned::Spanned, visit::Visit};

/// Default minor/stylistic lints
const DEFAULT_MINOR_LINTS: &[&str] = &[
    // Imports
    "wildcard_imports",
    "enum_glob_use",
    "single_component_path_imports",
    // Style
    "single_match",
    "single_match_else",
    "redundant_closure",
    "redundant_closure_for_method_calls",
    "match_bool",
    "if_not_else",
    // Naming
    "module_name_repetitions",
    "similar_names",
    "many_single_char_names",
    // Documentation
    "missing_docs_in_private_items",
    "missing_errors_doc",
    "missing_panics_doc",
    "missing_safety_doc",
    // String operations
    "format_push_string",
    "string_add",
    "string_add_assign",
    // Shadowing
    "shadow_reuse",
    "shadow_same",
    "shadow_unrelated",
    // Debug/development
    "todo",
    "unimplemented",
    "print_stdout",
    "print_stderr",
    "dbg_macro",
    "use_debug",
    // Misc style
    "use_self",
    "implicit_return",
    "unnecessary_wraps",
    "items_after_statements",
    "single_char_lifetime_names",
    "needless_pass_by_value",
    "trivially_copy_pass_by_ref",
];

/// A rule for detecting minor lint overrides
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LintOverrideRule {
    /// Whether this rule is enabled (default: true)
    #[serde(default = "default_enabled")]
    pub enabled: bool,
    /// Clippy lint names to detect (without "clippy::" prefix)
    #[serde(default = "default_minor_lints")]
    pub clippy_lints: Vec<String>,
    /// Regex patterns for paths where check applies (empty = everywhere)
    #[serde(default)]
    pub paths: Vec<String>,
    /// Custom message template with placeholders: {lint}, {path}, {line}
    #[serde(default = "default_message")]
    pub message: String,
}

fn default_enabled() -> bool {
    true
}

fn default_minor_lints() -> Vec<String> {
    DEFAULT_MINOR_LINTS.iter().map(|s| s.to_string()).collect()
}

fn default_message() -> String {
    "Minor lint override '#[allow(clippy::{lint})]' detected - review if suppression is necessary"
        .to_string()
}

impl LintOverrideRule {
    fn compile_path_patterns(&self) -> Result<Vec<Regex>, String> {
        self.paths
            .iter()
            .map(|p| Regex::new(p).map_err(|e| format!("Invalid path pattern '{}': {}", p, e)))
            .collect()
    }
}

define_checker! {
    /// Checker for E1903: Minor lint override detection
    E1903MinorLintOverride,
    code = "E1903",
    name = "Minor lint override detected",
    suggestions = "Consider if the suppression is truly necessary. Minor lints help maintain consistent style and documentation.",
    target_items = [Struct, Enum, Trait, Function, Const, Static, Type, Use, Module, Impl],
    config_entry_name = "e1903_minor_lint_override",
    config = E1903Config {
        enabled: bool = true,
        severity: crate::config::SeverityLevel = crate::config::SeverityLevel::Low,
        categories: Vec<crate::config::CheckerCategory> = vec![crate::config::CheckerCategory::Compliance],
        /// Rules for detecting minor lint overrides
        rules: Vec<LintOverrideRule> = vec![LintOverrideRule {
            enabled: true,
            clippy_lints: default_minor_lints(),
            paths: vec![],
            message: default_message(),
        }],
    },
    check_item(self, item, file_path) {
        let mut visitor = LintOverrideVisitor {
            violations: Vec::new(),
            file_path,
            checker: self,
        };
        visitor.visit_item(item);
        Ok(visitor.violations)
    }
}

struct LintOverrideVisitor<'a> {
    violations: Vec<Violation>,
    file_path: &'a str,
    checker: &'a E1903MinorLintOverride,
}

impl<'a> LintOverrideVisitor<'a> {
    fn check_attributes(&mut self, attrs: &[syn::Attribute], span: proc_macro2::Span) {
        let normalized_path = Path::new(self.file_path)
            .to_str()
            .unwrap_or(self.file_path)
            .replace('\\', "/");

        for attr in attrs {
            let attr_str = quote::quote!(#attr).to_string();

            if !attr_str.contains("allow") {
                continue;
            }

            for rule in &self.checker.config.rules {
                if !rule.enabled {
                    continue;
                }

                let path_regexes = match rule.compile_path_patterns() {
                    Ok(r) => r,
                    Err(_) => continue,
                };

                if !path_regexes.is_empty() {
                    let path_matches = path_regexes.iter().any(|re| re.is_match(&normalized_path));
                    if !path_matches {
                        continue;
                    }
                }

                for lint in &rule.clippy_lints {
                    let clippy_pattern = format!("clippy :: {}", lint);
                    let simple_pattern = format!("({})", lint);

                    if attr_str.contains(&clippy_pattern)
                        || (attr_str.contains(&simple_pattern) && !attr_str.contains("clippy"))
                    {
                        let message = rule
                            .message
                            .replace("{lint}", lint)
                            .replace("{path}", &normalized_path)
                            .replace("{line}", &span.start().line.to_string());

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
        }
    }
}

impl<'a> Visit<'a> for LintOverrideVisitor<'a> {
    fn visit_item_struct(&mut self, node: &'a syn::ItemStruct) {
        self.check_attributes(&node.attrs, node.ident.span());
        syn::visit::visit_item_struct(self, node);
    }

    fn visit_item_enum(&mut self, node: &'a syn::ItemEnum) {
        self.check_attributes(&node.attrs, node.ident.span());
        syn::visit::visit_item_enum(self, node);
    }

    fn visit_item_trait(&mut self, node: &'a syn::ItemTrait) {
        self.check_attributes(&node.attrs, node.ident.span());
        syn::visit::visit_item_trait(self, node);
    }

    fn visit_item_fn(&mut self, node: &'a syn::ItemFn) {
        self.check_attributes(&node.attrs, node.sig.ident.span());
        syn::visit::visit_item_fn(self, node);
    }

    fn visit_item_const(&mut self, node: &'a syn::ItemConst) {
        self.check_attributes(&node.attrs, node.ident.span());
        syn::visit::visit_item_const(self, node);
    }

    fn visit_item_static(&mut self, node: &'a syn::ItemStatic) {
        self.check_attributes(&node.attrs, node.ident.span());
        syn::visit::visit_item_static(self, node);
    }

    fn visit_item_type(&mut self, node: &'a syn::ItemType) {
        self.check_attributes(&node.attrs, node.ident.span());
        syn::visit::visit_item_type(self, node);
    }

    fn visit_item_mod(&mut self, node: &'a syn::ItemMod) {
        self.check_attributes(&node.attrs, node.ident.span());
        syn::visit::visit_item_mod(self, node);
    }

    fn visit_item_impl(&mut self, node: &'a syn::ItemImpl) {
        self.check_attributes(&node.attrs, node.span());
        syn::visit::visit_item_impl(self, node);
    }

    fn visit_item_use(&mut self, node: &'a syn::ItemUse) {
        self.check_attributes(&node.attrs, node.span());
        syn::visit::visit_item_use(self, node);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn check_code_with_default_config(code: &str, file_path: &str) -> Vec<Violation> {
        let checker = E1903MinorLintOverride::default();
        let file = syn::parse_file(code).expect("Failed to parse");
        let mut violations = Vec::new();
        for item in &file.items {
            violations.extend(checker.check_item(item, file_path).unwrap());
        }
        violations
    }

    #[test]
    fn test_detects_wildcard_imports_override() {
        let code = r#"
            #[allow(clippy::wildcard_imports)]
            use std::collections::*;
        "#;

        let violations = check_code_with_default_config(code, "src/main.rs");
        assert_eq!(violations.len(), 1);
        assert!(violations[0].message.contains("wildcard_imports"));
    }

    #[test]
    fn test_detects_todo_override() {
        let code = r#"
            #[allow(clippy::todo)]
            fn work_in_progress() {
                todo!("implement later");
            }
        "#;

        let violations = check_code_with_default_config(code, "src/main.rs");
        assert_eq!(violations.len(), 1);
        assert!(violations[0].message.contains("todo"));
    }

    #[test]
    fn test_detects_missing_docs_override() {
        let code = r#"
            #[allow(clippy::missing_docs_in_private_items)]
            fn undocumented() {}
        "#;

        let violations = check_code_with_default_config(code, "src/main.rs");
        assert_eq!(violations.len(), 1);
    }

    #[test]
    fn test_ignores_critical_lints() {
        let code = r#"
            #[allow(clippy::unwrap_used)]
            fn dangerous() {}
        "#;

        let violations = check_code_with_default_config(code, "src/main.rs");
        assert!(
            violations.is_empty(),
            "Critical lint should not be flagged by E1903"
        );
    }

    #[test]
    fn test_ignores_medium_lints() {
        let code = r#"
            #[allow(clippy::cognitive_complexity)]
            fn complex() {}
        "#;

        let violations = check_code_with_default_config(code, "src/main.rs");
        assert!(
            violations.is_empty(),
            "Medium lint should not be flagged by E1903"
        );
    }

    #[test]
    fn test_path_filter_excludes_tests() {
        let code = r#"
            #[allow(clippy::wildcard_imports)]
            use std::*;
        "#;

        let mut config = E1903Config::default();
        config.rules = vec![LintOverrideRule {
            enabled: true,
            clippy_lints: vec!["wildcard_imports".to_string()],
            paths: vec!["^src/.*$".to_string()], // Only src/
            message: default_message(),
        }];
        let checker = E1903MinorLintOverride { config };

        let file = syn::parse_file(code).expect("Failed to parse");

        let mut violations_src = Vec::new();
        for item in &file.items {
            violations_src.extend(checker.check_item(item, "src/lib.rs").unwrap());
        }
        assert_eq!(violations_src.len(), 1, "Should flag in src/");

        let mut violations_tests = Vec::new();
        for item in &file.items {
            violations_tests.extend(checker.check_item(item, "tests/test.rs").unwrap());
        }
        assert!(violations_tests.is_empty(), "Should not flag in tests/");
    }
}
