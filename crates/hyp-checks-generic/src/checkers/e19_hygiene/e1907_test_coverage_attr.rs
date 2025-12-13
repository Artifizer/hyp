//! E1907: Test Coverage Attribute Checker
//!
//! Detects test modules that have `#[cfg(test)]` but are missing the
//! `#[cfg_attr(coverage_nightly, coverage(off))]` attribute for excluding
//! tests from code coverage instrumentation.

use crate::{checker::Checker, define_checker, violation::Violation};
use syn::visit::Visit;

/// Default feature name for coverage configuration
const DEFAULT_COVERAGE_FEATURE: &str = "coverage_nightly";

define_checker! {
    /// Checker for E1907: Test modules missing coverage attribute
    E1907TestCoverageAttr,
    code = "E1907",
    name = "Test module missing coverage attribute",
    suggestions = "Add #[cfg_attr(coverage_nightly, coverage(off))] alongside #[cfg(test)]",
    target_items = [Module],
    config_entry_name = "e1907_test_coverage_attr",
    config = E1907Config {
        enabled: bool = true,
        severity: crate::config::SeverityLevel = crate::config::SeverityLevel::High,
        categories: Vec<crate::config::CheckerCategory> = vec![crate::config::CheckerCategory::Compliance],
        /// The feature name used for coverage configuration (default: "coverage_nightly")
        coverage_feature: String = DEFAULT_COVERAGE_FEATURE.to_string(),
    },
    check_item(self, item, file_path) {
        let mut visitor = TestCoverageVisitor {
            violations: Vec::new(),
            file_path,
            checker: self,
        };
        visitor.visit_item(item);
        Ok(visitor.violations)
    }
}

struct TestCoverageVisitor<'a> {
    violations: Vec<Violation>,
    file_path: &'a str,
    checker: &'a E1907TestCoverageAttr,
}

impl<'a> TestCoverageVisitor<'a> {
    /// Check if a module has the #[cfg(test)] attribute
    fn has_cfg_test(attrs: &[syn::Attribute]) -> bool {
        for attr in attrs {
            let attr_str = quote::quote!(#attr).to_string();
            // Check for #[cfg(test)] pattern
            if attr_str.contains("cfg") && attr_str.contains("test") {
                // More precise check: look for cfg(test) or cfg ( test )
                if attr_str.contains("cfg (test)")
                    || attr_str.contains("cfg(test)")
                    || (attr_str.contains("cfg") && attr_str.contains("( test )"))
                {
                    return true;
                }
            }
        }
        false
    }

    /// Check if a module has the #[cfg_attr(feature, coverage(off))] attribute
    fn has_coverage_attr(&self, attrs: &[syn::Attribute]) -> bool {
        let feature_name = &self.checker.config.coverage_feature;

        for attr in attrs {
            let attr_str = quote::quote!(#attr).to_string();
            // Check for #[cfg_attr(coverage_feature, coverage(off))] pattern
            if attr_str.contains("cfg_attr") && attr_str.contains("coverage") {
                // Check if it contains the feature name and coverage(off)
                if attr_str.contains(feature_name)
                    && (attr_str.contains("coverage (off)") || attr_str.contains("coverage(off)"))
                {
                    return true;
                }
            }
        }
        false
    }

    fn check_module(&mut self, node: &syn::ItemMod) {
        // Only check modules that have #[cfg(test)]
        if !Self::has_cfg_test(&node.attrs) {
            return;
        }

        // Check if it also has the coverage attribute
        if !self.has_coverage_attr(&node.attrs) {
            let feature = &self.checker.config.coverage_feature;
            let message = format!(
                "Test module 'mod {}' has #[cfg(test)] but is missing #[cfg_attr({}, coverage(off))]",
                node.ident, feature
            );

            self.violations.push(
                Violation::new(
                    self.checker.code(),
                    self.checker.name(),
                    self.checker.severity().into(),
                    &message,
                    self.file_path,
                    node.ident.span().start().line,
                    node.ident.span().start().column + 1,
                )
                .with_suggestion(&format!(
                    "Add #[cfg_attr({}, coverage(off))] attribute to exclude test module from coverage",
                    feature
                )),
            );
        }
    }
}

impl<'a> Visit<'a> for TestCoverageVisitor<'a> {
    fn visit_item_mod(&mut self, node: &'a syn::ItemMod) {
        self.check_module(node);
        // Continue visiting nested modules
        syn::visit::visit_item_mod(self, node);
    }
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use super::*;

    fn check_code_with_default_config(code: &str, file_path: &str) -> Vec<Violation> {
        let checker = E1907TestCoverageAttr::default();
        let file = syn::parse_file(code).expect("Failed to parse");
        let mut violations = Vec::new();
        for item in &file.items {
            violations.extend(checker.check_item(item, file_path).unwrap());
        }
        violations
    }

    fn check_code_with_config(code: &str, config: E1907Config, file_path: &str) -> Vec<Violation> {
        let checker = E1907TestCoverageAttr { config };
        let file = syn::parse_file(code).expect("Failed to parse");
        let mut violations = Vec::new();
        for item in &file.items {
            violations.extend(checker.check_item(item, file_path).unwrap());
        }
        violations
    }

    #[test]
    fn test_detects_missing_coverage_attr() {
        let code = r#"
            #[cfg(test)]
            mod tests {
                use super::*;

                #[test]
                fn test_example() {
                    assert!(true);
                }
            }
        "#;

        let violations = check_code_with_default_config(code, "src/lib.rs");
        assert_eq!(violations.len(), 1);
        assert!(violations[0].message.contains("missing"));
        assert!(violations[0].message.contains("coverage_nightly"));
    }

    #[test]
    fn test_passes_with_coverage_attr() {
        let code = r#"
            #[cfg(test)]
            #[cfg_attr(coverage_nightly, coverage(off))]
            mod tests {
                use super::*;

                #[test]
                fn test_example() {
                    assert!(true);
                }
            }
        "#;

        let violations = check_code_with_default_config(code, "src/lib.rs");
        assert!(
            violations.is_empty(),
            "Should not flag modules with coverage attr"
        );
    }

    #[test]
    fn test_ignores_non_test_modules() {
        let code = r#"
            mod utils {
                pub fn helper() {}
            }
        "#;

        let violations = check_code_with_default_config(code, "src/lib.rs");
        assert!(violations.is_empty(), "Should not flag non-test modules");
    }

    #[test]
    fn test_custom_coverage_feature() {
        let code = r#"
            #[cfg(test)]
            #[cfg_attr(custom_coverage, coverage(off))]
            mod tests {
                #[test]
                fn test_example() {}
            }
        "#;

        // With default config, should flag because default feature is "coverage_nightly"
        let violations = check_code_with_default_config(code, "src/lib.rs");
        assert_eq!(violations.len(), 1, "Should flag with wrong feature name");

        // With custom feature config, should pass
        let mut config = E1907Config::default();
        config.coverage_feature = "custom_coverage".to_string();
        let violations = check_code_with_config(code, config, "src/lib.rs");
        assert!(
            violations.is_empty(),
            "Should pass with matching custom feature"
        );
    }

    #[test]
    fn test_detects_nested_test_modules() {
        let code = r#"
            mod outer {
                #[cfg(test)]
                mod tests {
                    #[test]
                    fn test_example() {}
                }
            }
        "#;

        let violations = check_code_with_default_config(code, "src/lib.rs");
        assert_eq!(violations.len(), 1, "Should detect nested test modules");
    }

    #[test]
    fn test_disabled_checker() {
        let code = r#"
            #[cfg(test)]
            mod tests {
                #[test]
                fn test_example() {}
            }
        "#;

        let mut config = E1907Config::default();
        config.enabled = false;
        let checker = E1907TestCoverageAttr { config };

        // When disabled, check_item should still be called but violations filtered elsewhere
        let file = syn::parse_file(code).expect("Failed to parse");
        let mut violations = Vec::new();
        for item in &file.items {
            violations.extend(checker.check_item(item, "src/lib.rs").unwrap());
        }
        // The checker itself still returns violations; filtering happens at the analyzer level
        assert!(!violations.is_empty());
    }
}
