//! E1902: Medium Lint Override Detection
//!
//! Detects when code uses `#[allow(...)]` attributes to override medium-severity Clippy lints.
//! These include performance, integer safety, and complexity lints that should usually be fixed.

use crate::{checker::Checker, define_checker, violation::Violation};
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::path::Path;
use syn::{spanned::Spanned, visit::Visit};

/// Default medium-severity lints
const DEFAULT_MEDIUM_LINTS: &[&str] = &[
    // Integer/arithmetic safety
    "arithmetic_side_effects",
    "integer_arithmetic",
    "float_arithmetic",
    "integer_division",
    // Type conversions
    "as_conversions",
    "cast_possible_truncation",
    "cast_possible_wrap",
    "cast_sign_loss",
    "cast_precision_loss",
    "cast_lossless",
    // Clone/copy efficiency
    "clone_on_copy",
    "redundant_clone",
    "unnecessary_to_owned",
    "needless_borrow",
    // Complexity
    "cognitive_complexity",
    "too_many_arguments",
    "too_many_lines",
    "type_complexity",
    // Collection efficiency
    "needless_collect",
    "or_fun_call",
    "expect_fun_call",
    "iter_nth",
    // Memory
    "large_stack_arrays",
    "large_types_passed_by_value",
    "box_collection",
    "vec_box",
    // Suspicious patterns
    "mutable_key_type",
    "suspicious_arithmetic_impl",
    "suspicious_op_assign_impl",
    // Error handling
    "map_err_ignore",
    "result_large_err",
    // Concurrency
    "mutex_atomic",
    "mutex_integer",
];

/// A rule for detecting medium lint overrides
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LintOverrideRule {
    /// Whether this rule is enabled (default: true)
    #[serde(default = "default_enabled")]
    pub enabled: bool,
    /// Clippy lint names to detect (without "clippy::" prefix)
    #[serde(default = "default_medium_lints")]
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

fn default_medium_lints() -> Vec<String> {
    DEFAULT_MEDIUM_LINTS.iter().map(|s| s.to_string()).collect()
}

fn default_message() -> String {
    "Medium lint override '#[allow(clippy::{lint})]' detected - consider fixing the underlying issue".to_string()
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
    /// Checker for E1902: Medium lint override detection
    E1902MediumLintOverride,
    code = "E1902",
    name = "Medium lint override detected",
    suggestions = "Consider fixing the underlying issue. Medium lints help with performance, integer safety, and code complexity.",
    target_items = [Struct, Enum, Trait, Function, Const, Static, Type, Use, Module, Impl],
    config_entry_name = "e1902_medium_lint_override",
    config = E1902Config {
        enabled: bool = true,
        severity: crate::config::SeverityLevel = crate::config::SeverityLevel::Medium,
        categories: Vec<crate::config::CheckerCategory> = vec![crate::config::CheckerCategory::Compliance],
        /// Rules for detecting medium lint overrides
        rules: Vec<LintOverrideRule> = vec![LintOverrideRule {
            enabled: true,
            clippy_lints: default_medium_lints(),
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
    checker: &'a E1902MediumLintOverride,
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
        let checker = E1902MediumLintOverride::default();
        let file = syn::parse_file(code).expect("Failed to parse");
        let mut violations = Vec::new();
        for item in &file.items {
            violations.extend(checker.check_item(item, file_path).unwrap());
        }
        violations
    }

    #[test]
    fn test_detects_cognitive_complexity_override() {
        let code = r#"
            #[allow(clippy::cognitive_complexity)]
            fn complex_function() {
                // complex logic
            }
        "#;

        let violations = check_code_with_default_config(code, "src/main.rs");
        assert_eq!(violations.len(), 1);
        assert!(violations[0].message.contains("cognitive_complexity"));
    }

    #[test]
    fn test_detects_as_conversions_override() {
        let code = r#"
            #[allow(clippy::as_conversions)]
            fn convert(x: i64) -> i32 {
                x as i32
            }
        "#;

        let violations = check_code_with_default_config(code, "src/main.rs");
        assert_eq!(violations.len(), 1);
        assert!(violations[0].message.contains("as_conversions"));
    }

    #[test]
    fn test_detects_clone_on_copy_override() {
        let code = r#"
            #[allow(clippy::clone_on_copy)]
            fn inefficient() {
                let x: i32 = 5;
                let y = x.clone();
            }
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
            "Critical lint should not be flagged by E1902"
        );
    }

    #[test]
    fn test_ignores_minor_lints() {
        let code = r#"
            #[allow(clippy::wildcard_imports)]
            use std::collections::*;
        "#;

        let violations = check_code_with_default_config(code, "src/main.rs");
        assert!(
            violations.is_empty(),
            "Minor lint should not be flagged by E1902"
        );
    }
}
