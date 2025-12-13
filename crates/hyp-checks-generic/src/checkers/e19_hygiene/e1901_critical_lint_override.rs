//! E1901: Critical Lint Override Detection
//!
//! Detects when code uses `#[allow(...)]` attributes to override critical Clippy lints.
//! LLMs often add these attributes to "fix" warnings instead of addressing root causes,
//! which undermines code safety and static analysis value.

use crate::{checker::Checker, define_checker, violation::Violation};
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::path::Path;
use syn::{spanned::Spanned, visit::Visit};

/// Default critical lints that should rarely be overridden
const DEFAULT_CRITICAL_LINTS: &[&str] = &[
    // Unsafe code
    "unsafe_code",
    "unsafe_op_in_unsafe_fn",
    "undocumented_unsafe_blocks",
    "multiple_unsafe_ops_per_block",
    // Panicking
    "panic",
    "panic_in_result_fn",
    "unreachable",
    // Unwrap/Expect
    "unwrap_used",
    "expect_used",
    "unwrap_in_result",
    "get_unwrap",
    "panicking_unwrap",
    // Transmute
    "transmute",
    "transmute_bytes_to_str",
    "transmute_ptr_to_ptr",
    "transmute_undefined_repr",
    "useless_transmute",
    "wrong_transmute",
    // Allow overrides
    "allow_attributes",
    "allow_attributes_without_reason",
    "blanket_clippy_restriction_lints",
    // Memory safety
    "invalid_atomic_ordering",
    "invalid_regex",
    "uninit_vec",
    "uninit_assumed_init",
    "mem_replace_with_uninit",
    // Concurrency
    "await_holding_lock",
    "await_holding_refcell_ref",
    // Indexing
    "indexing_slicing",
    "out_of_bounds_indexing",
];

/// A rule for detecting critical lint overrides
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LintOverrideRule {
    /// Whether this rule is enabled (default: true)
    #[serde(default = "default_enabled")]
    pub enabled: bool,
    /// Clippy lint names to detect (without "clippy::" prefix)
    #[serde(default = "default_critical_lints")]
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

fn default_critical_lints() -> Vec<String> {
    DEFAULT_CRITICAL_LINTS
        .iter()
        .map(|s| s.to_string())
        .collect()
}

fn default_message() -> String {
    "Critical lint override '#[allow(clippy::{lint})]' detected - fix the underlying issue instead"
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
    /// Checker for E1901: Critical lint override detection
    E1901CriticalLintOverride,
    code = "E1901",
    name = "Critical lint override detected",
    suggestions = "Fix the underlying issue instead of suppressing the lint. Critical lints protect against unsafe code, panics, and memory safety issues.",
    target_items = [Struct, Enum, Trait, Function, Const, Static, Type, Use, Module, Impl],
    config_entry_name = "e1901_critical_lint_override",
    config = E1901Config {
        enabled: bool = true,
        severity: crate::config::SeverityLevel = crate::config::SeverityLevel::High,
        categories: Vec<crate::config::CheckerCategory> = vec![crate::config::CheckerCategory::Compliance],
        /// Rules for detecting critical lint overrides
        rules: Vec<LintOverrideRule> = vec![LintOverrideRule {
            enabled: true,
            clippy_lints: default_critical_lints(),
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
    checker: &'a E1901CriticalLintOverride,
}

impl<'a> LintOverrideVisitor<'a> {
    fn check_attributes(&mut self, attrs: &[syn::Attribute], span: proc_macro2::Span) {
        let normalized_path = Path::new(self.file_path)
            .to_str()
            .unwrap_or(self.file_path)
            .replace('\\', "/");

        for attr in attrs {
            // Convert attribute to string for pattern matching
            let attr_str = quote::quote!(#attr).to_string();

            // Check if this is an #[allow(...)] attribute
            if !attr_str.contains("allow") {
                continue;
            }

            for rule in &self.checker.config.rules {
                // Skip disabled rules
                if !rule.enabled {
                    continue;
                }

                // Check path restrictions
                let path_regexes = match rule.compile_path_patterns() {
                    Ok(r) => r,
                    Err(_) => continue,
                };

                // If paths specified, check if current path matches
                if !path_regexes.is_empty() {
                    let path_matches = path_regexes.iter().any(|re| re.is_match(&normalized_path));
                    if !path_matches {
                        continue;
                    }
                }

                // Check for each prohibited lint
                for lint in &rule.clippy_lints {
                    // Match patterns like #[allow(clippy::lint_name)] or #[allow(lint_name)]
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
        let checker = E1901CriticalLintOverride::default();
        let file = syn::parse_file(code).expect("Failed to parse");
        let mut violations = Vec::new();
        for item in &file.items {
            violations.extend(checker.check_item(item, file_path).unwrap());
        }
        violations
    }

    #[test]
    fn test_detects_unwrap_used_override() {
        let code = r#"
            #[allow(clippy::unwrap_used)]
            fn dangerous() {
                Some(42).unwrap();
            }
        "#;

        let violations = check_code_with_default_config(code, "src/main.rs");
        assert_eq!(violations.len(), 1);
        assert!(violations[0].message.contains("unwrap_used"));
    }

    #[test]
    fn test_detects_unsafe_code_override() {
        let code = r#"
            #[allow(unsafe_code)]
            fn use_unsafe() {
                unsafe { std::ptr::null::<i32>().read() };
            }
        "#;

        let violations = check_code_with_default_config(code, "src/main.rs");
        assert_eq!(violations.len(), 1);
        assert!(violations[0].message.contains("unsafe_code"));
    }

    #[test]
    fn test_detects_panic_override() {
        let code = r#"
            #[allow(clippy::panic)]
            fn may_panic() {
                panic!("oops");
            }
        "#;

        let violations = check_code_with_default_config(code, "src/main.rs");
        assert_eq!(violations.len(), 1);
        assert!(violations[0].message.contains("panic"));
    }

    #[test]
    fn test_ignores_non_critical_lints() {
        let code = r#"
            #[allow(clippy::wildcard_imports)]
            use std::collections::*;
        "#;

        let violations = check_code_with_default_config(code, "src/main.rs");
        assert!(
            violations.is_empty(),
            "Non-critical lint should not be flagged"
        );
    }

    #[test]
    fn test_ignores_derive_attributes() {
        let code = r#"
            #[derive(Debug, Clone)]
            struct User { id: i32 }
        "#;

        let violations = check_code_with_default_config(code, "src/main.rs");
        assert!(violations.is_empty());
    }

    #[test]
    fn test_custom_rule_with_path_filter() {
        let code = r#"
            #[allow(clippy::unwrap_used)]
            fn test_helper() {}
        "#;

        // Create config that only applies to src/ paths (positive match)
        let mut config = E1901Config::default();
        config.rules = vec![LintOverrideRule {
            enabled: true,
            clippy_lints: vec!["unwrap_used".to_string()],
            paths: vec!["^src/.*$".to_string()],
            message: default_message(),
        }];
        let checker = E1901CriticalLintOverride { config };

        let file = syn::parse_file(code).expect("Failed to parse");
        let mut violations = Vec::new();
        for item in &file.items {
            violations.extend(checker.check_item(item, "src/main.rs").unwrap());
        }
        assert_eq!(violations.len(), 1, "Should flag in src/");

        let mut violations_test = Vec::new();
        for item in &file.items {
            violations_test.extend(checker.check_item(item, "tests/integration.rs").unwrap());
        }
        assert!(violations_test.is_empty(), "Should not flag in tests/");
    }

    #[test]
    fn test_disabled_rule_is_skipped() {
        let code = r#"
            #[allow(clippy::unwrap_used)]
            fn dangerous() {}
        "#;

        let mut config = E1901Config::default();
        config.rules = vec![LintOverrideRule {
            enabled: false,
            clippy_lints: vec!["unwrap_used".to_string()],
            paths: vec![],
            message: default_message(),
        }];
        let checker = E1901CriticalLintOverride { config };

        let file = syn::parse_file(code).expect("Failed to parse");
        let mut violations = Vec::new();
        for item in &file.items {
            violations.extend(checker.check_item(item, "src/main.rs").unwrap());
        }
        assert!(
            violations.is_empty(),
            "Disabled rule should not produce violations"
        );
    }
}
