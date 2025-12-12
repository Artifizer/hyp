//! Main analyzer orchestration

use crate::{
    checker::Checker,
    config::{AnalyzerConfig, CheckerCategory},
    violation::Violation,
    AnalyzerError, Result,
};
use std::collections::HashSet;
use std::fs;
use std::path::Path;
use walkdir::WalkDir;

/// Filtering options for the analyzer
#[derive(Debug, Clone, Default)]
pub struct AnalyzerFilters {
    /// Minimum severity level (1-3). If set, only checkers with this severity or higher will run
    pub min_severity: Option<u8>,

    /// Specific categories to include. If set, only checkers in these categories will run
    pub categories: Option<HashSet<CheckerCategory>>,

    /// Whether to check unit tests (default: false, tests are skipped)
    pub check_tests: bool,
}

/// Information about a checker for display purposes
#[derive(Debug, Clone)]
pub struct CheckerInfo {
    /// Checker code (e.g., "E1001")
    pub code: String,
    /// Checker name
    pub name: String,
    /// Severity level (1-3)
    pub severity: u8,
    /// Categories
    pub categories: Vec<String>,
}

/// Extended information about a checker including suggestions
#[derive(Debug, Clone)]
pub struct CheckerGuideline {
    /// Checker code (e.g., "E1001")
    pub code: String,
    /// Checker name
    pub name: String,
    /// Suggestions for fixing violations
    pub suggestions: String,
}

/// Main analyzer that coordinates all checkers
pub struct Analyzer {
    config: AnalyzerConfig,
    checkers: Vec<Box<dyn Checker>>,
    filters: AnalyzerFilters,
}

/// Check if a file/module has the #[cfg(test)] attribute
fn is_test_module(file: &syn::File) -> bool {
    file.attrs.iter().any(|attr| {
        if attr.path().is_ident("cfg") {
            if let syn::Meta::List(meta_list) = &attr.meta {
                let tokens = meta_list.tokens.to_string();
                return tokens.contains("test");
            }
        }
        false
    })
}

/// Check if an item is a test function or test module
fn is_test_item(item: &syn::Item) -> bool {
    match item {
        syn::Item::Fn(func) => {
            // Check for #[test] or #[cfg(test)] attributes
            func.attrs.iter().any(|attr| {
                attr.path().is_ident("test")
                    || (attr.path().is_ident("cfg") && {
                        if let syn::Meta::List(meta_list) = &attr.meta {
                            meta_list.tokens.to_string().contains("test")
                        } else {
                            false
                        }
                    })
            })
        }
        syn::Item::Mod(module) => {
            // Check for #[cfg(test)] attribute on module
            module.attrs.iter().any(|attr| {
                if attr.path().is_ident("cfg") {
                    if let syn::Meta::List(meta_list) = &attr.meta {
                        return meta_list.tokens.to_string().contains("test");
                    }
                }
                false
            })
        }
        _ => false,
    }
}

impl Analyzer {
    /// Get information about all available checkers (regardless of config/filters)
    pub fn all_checkers() -> Vec<CheckerInfo> {
        crate::registry::get_all_checkers()
            .iter()
            .map(|reg| CheckerInfo {
                code: reg.descriptor.code.to_string(),
                name: reg.descriptor.name.to_string(),
                severity: reg.descriptor.default_severity.as_u8(),
                categories: reg
                    .descriptor
                    .default_categories
                    .iter()
                    .map(|cat| cat.as_str().to_string())
                    .collect(),
            })
            .collect()
    }

    /// Create a new analyzer with the given configuration
    pub fn new(config: AnalyzerConfig) -> Self {
        Self::new_with_filters(config, AnalyzerFilters::default())
    }

    /// Create a new analyzer with configuration and filters (uses default checkers)
    pub fn new_with_filters(config: AnalyzerConfig, filters: AnalyzerFilters) -> Self {
        Self::new_with_checkers(config, filters, crate::registry::get_all_checkers())
    }

    /// Create a new analyzer with configuration, filters, and registered checkers
    pub fn new_with_checkers(
        config: AnalyzerConfig,
        filters: AnalyzerFilters,
        checker_factories: Vec<crate::registry::CheckerRegistration>,
    ) -> Self {
        let mut all_checkers: Vec<Box<dyn Checker>> = Vec::new();

        // Create checkers using the provided factories
        for registration in checker_factories {
            if let Some(checker) = (registration.factory)(&config) {
                all_checkers.push(checker);
            }
        }

        // Apply filters
        let checkers = all_checkers
            .into_iter()
            .filter(|checker| {
                // Filter by severity
                if let Some(min_sev) = filters.min_severity {
                    if checker.severity().as_u8() < min_sev {
                        return false;
                    }
                }

                // Filter by category
                if let Some(ref cats) = filters.categories {
                    if !checker.categories().iter().any(|c| cats.contains(c)) {
                        return false;
                    }
                }

                true
            })
            .collect();

        Self {
            config,
            checkers,
            filters,
        }
    }

    /// Create analyzer with default configuration
    pub fn with_defaults() -> Self {
        Self::new(AnalyzerConfig::default())
    }

    /// Analyze a single Rust source file
    pub fn analyze_file(&self, path: &Path) -> Result<Vec<Violation>> {
        let content = fs::read_to_string(path).map_err(AnalyzerError::Io)?;

        let syntax = syn::parse_file(&content).map_err(|e| AnalyzerError::Parse(e.to_string()))?;

        let file_path = path.to_string_lossy().to_string();
        let mut violations = Vec::new();

        // Detect if this is a test module/file
        let is_test_file = is_test_module(&syntax);

        // Run all enabled checkers on each item
        // Pass the source content so checkers can calculate line numbers
        for item in &syntax.items {
            // Check if this specific item is a test
            let is_test_item = is_test_file || is_test_item(item);

            // Skip test items if check_tests is false
            if is_test_item && !self.filters.check_tests {
                continue;
            }

            for checker in &self.checkers {
                violations.extend(checker.check_item(item, &file_path)?);
            }
        }

        Ok(violations)
    }

    /// Analyze all Rust files in a directory recursively
    pub fn analyze_directory(&self, path: &Path) -> Result<Vec<Violation>> {
        let mut violations = Vec::new();

        for entry in WalkDir::new(path)
            .follow_links(true)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let path = entry.path();

            // Only analyze .rs files
            if path.extension().and_then(|s| s.to_str()) == Some("rs") {
                // Skip target directory
                if path.components().any(|c| c.as_os_str() == "target") {
                    continue;
                }

                match self.analyze_file(path) {
                    Ok(file_violations) => violations.extend(file_violations),
                    Err(e) => {
                        eprintln!("Warning: Failed to analyze {}: {}", path.display(), e);
                    }
                }
            }
        }

        Ok(violations)
    }

    /// Analyze a path (file or directory)
    pub fn analyze(&self, path: &Path) -> Result<Vec<Violation>> {
        if path.is_file() {
            self.analyze_file(path)
        } else if path.is_dir() {
            self.analyze_directory(path)
        } else {
            Err(AnalyzerError::Config(format!(
                "Path does not exist: {}",
                path.display()
            )))
        }
    }

    /// Get the current configuration
    pub fn config(&self) -> &AnalyzerConfig {
        &self.config
    }

    /// Get the number of enabled checkers
    pub fn checker_count(&self) -> usize {
        self.checkers.len()
    }

    /// Get information about currently enabled checkers
    pub fn enabled_checkers(&self) -> Vec<CheckerInfo> {
        self.checkers
            .iter()
            .map(|c| CheckerInfo {
                code: c.code().to_string(),
                name: c.name().to_string(),
                severity: c.severity().as_u8(),
                categories: c
                    .categories()
                    .iter()
                    .map(|cat| cat.as_str().to_string())
                    .collect(),
            })
            .collect()
    }

    /// Get guidelines for currently enabled checkers (code, name, suggestions)
    pub fn enabled_guidelines(&self) -> Vec<CheckerGuideline> {
        self.checkers
            .iter()
            .map(|c| CheckerGuideline {
                code: c.code().to_string(),
                name: c.name().to_string(),
                suggestions: c.suggestions().to_string(),
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_analyze_file_with_panic() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(
            file,
            r#"
            fn test() {{
                panic!("error");
            }}
        "#
        )
        .unwrap();

        let analyzer = Analyzer::with_defaults();
        let violations = analyzer.analyze_file(file.path()).unwrap();

        assert!(violations.iter().any(|v| v.code == "E1001"));
    }

    #[test]
    fn test_analyze_file_with_long_function() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "fn long() {{").unwrap();
        for i in 0..300 {
            writeln!(file, "    let x{} = {};", i, i).unwrap();
        }
        writeln!(file, "}}").unwrap();

        let analyzer = Analyzer::with_defaults();
        let violations = analyzer.analyze_file(file.path()).unwrap();

        assert!(violations.iter().any(|v| v.code == "E1106"));
    }
}
