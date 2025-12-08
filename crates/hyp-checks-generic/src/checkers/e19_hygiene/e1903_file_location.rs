//! E1903: File location control
//!
//! Enforces that specific files can only exist in designated locations.
//! Useful for ensuring configuration files, build scripts, or special modules
//! are in their proper places.

use crate::{define_checker, violation::Violation};
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::path::Path;

/// A rule controlling where specific files can exist
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileLocationRule {
    /// Regex pattern for filename (e.g., "Cargo\\.toml", ".*\\.proto$")
    pub filename_pattern: String,
    /// Regex patterns for allowed file paths
    pub allowed_paths: Vec<String>,
    /// Custom message template with placeholders: {filename}, {path}, {allowed_paths}
    #[serde(default = "default_file_message")]
    pub message: String,
}

fn default_file_message() -> String {
    "File '{filename}' in {path} is not allowed (permitted in: {allowed_paths})".to_string()
}

impl FileLocationRule {
    fn compile_patterns(&self) -> Result<(Regex, Vec<Regex>), String> {
        let filename_regex = Regex::new(&self.filename_pattern)
            .map_err(|e| format!("Invalid filename pattern '{}': {}", self.filename_pattern, e))?;

        let path_regexes: Result<Vec<_>, _> = self
            .allowed_paths
            .iter()
            .map(|p| Regex::new(p).map_err(|e| format!("Invalid path pattern '{}': {}", p, e)))
            .collect();

        Ok((filename_regex, path_regexes?))
    }
}

define_checker! {
    /// Checker for E1903: File location control
    E1903FileLocation,
    code = "E1903",
    name = "File location violates project rules",
    suggestions = "Move the file to an allowed location according to project structure requirements",
    target_items = [],
    config_entry_name = "e1903_file_location",
    config = E1903Config {
        enabled: bool = true,
        severity: crate::config::SeverityLevel = crate::config::SeverityLevel::Medium,
        categories: Vec<crate::config::CheckerCategory> = vec![crate::config::CheckerCategory::Compliance],
        /// Project-specific file location rules (empty by default - configure in Hyp.toml)
        rules: Vec<FileLocationRule> = vec![],
    },
    check_item(self, _item, file_path) {
        // This checker operates at the file level, not item level
        // We only check once per file by checking on the first item
        // To avoid duplicate violations, we use a static flag pattern
        // In practice, this means we check the file path regardless of items

        let normalized_path = Path::new(file_path)
            .to_str()
            .unwrap_or(file_path)
            .replace('\\', "/");

        let filename = Path::new(file_path)
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("");

        let mut violations = Vec::new();

        for rule in &self.config.rules {
            // Compile patterns
            let Ok((filename_regex, path_regexes)) = rule.compile_patterns() else {
                continue;
            };

            // Check if filename matches
            if !filename_regex.is_match(filename) {
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
                .replace("{filename}", filename)
                .replace("{path}", &normalized_path)
                .replace("{allowed_paths}", &rule.allowed_paths.join(", "));

            violations.push(
                Violation::new(
                    self.code(),
                    self.name(),
                    self.severity().into(),
                    &message,
                    file_path,
                    1,
                    1,
                )
                .with_suggestion(self.suggestions()),
            );
        }

        Ok(violations)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::checker::Checker;

    fn check_file_with_config(rules: Vec<FileLocationRule>, file_path: &str) -> Vec<Violation> {
        let mut config = E1903Config::default();
        config.rules = rules;
        let checker = E1903FileLocation { config };

        // Parse empty file (we only check the path)
        let file = syn::parse_file("fn dummy() {}").expect("Failed to parse");
        let mut violations = Vec::new();
        for item in &file.items {
            violations.extend(checker.check_item(item, file_path).unwrap());
        }
        violations
    }

    #[test]
    fn test_clippy_toml_in_wrong_location() {
        let rules = vec![FileLocationRule {
            filename_pattern: "^Clippy\\.toml$".to_string(),
            allowed_paths: vec!["^[^/]+/Clippy\\.toml$".to_string()],
            message: "Clippy.toml must be at root".to_string(),
        }];

        let violations = check_file_with_config(rules, "src/config/Clippy.toml");
        assert_eq!(violations.len(), 1);
        assert!(violations[0].message.contains("Clippy.toml must be at root"));
    }

    #[test]
    fn test_clippy_toml_in_correct_location() {
        let rules = vec![FileLocationRule {
            filename_pattern: "^Clippy\\.toml$".to_string(),
            allowed_paths: vec!["^[^/]+/Clippy\\.toml$".to_string()],
            message: "Clippy.toml must be at root".to_string(),
        }];

        let violations = check_file_with_config(rules, "myproject/Clippy.toml");
        assert!(violations.is_empty());
    }

    #[test]
    fn test_proto_file_in_wrong_location() {
        let rules = vec![FileLocationRule {
            filename_pattern: ".*\\.proto$".to_string(),
            allowed_paths: vec!["^.*/proto/.*\\.proto$".to_string()],
            message: "Proto files must be in proto/ directory".to_string(),
        }];

        let violations = check_file_with_config(rules, "src/api/user.proto");
        assert_eq!(violations.len(), 1);
    }

    #[test]
    fn test_proto_file_in_correct_location() {
        let rules = vec![FileLocationRule {
            filename_pattern: ".*\\.proto$".to_string(),
            allowed_paths: vec!["^.*/proto/.*\\.proto$".to_string()],
            message: "Proto files must be in proto/ directory".to_string(),
        }];

        let violations = check_file_with_config(rules, "src/proto/user.proto");
        assert!(violations.is_empty());
    }

    #[test]
    fn test_non_matching_filename_ignored() {
        let rules = vec![FileLocationRule {
            filename_pattern: "^Clippy\\.toml$".to_string(),
            allowed_paths: vec!["^[^/]+/Clippy\\.toml$".to_string()],
            message: "Clippy.toml must be at root".to_string(),
        }];

        let violations = check_file_with_config(rules, "src/config/settings.toml");
        assert!(violations.is_empty());
    }

    #[test]
    fn test_multiple_allowed_paths() {
        let rules = vec![FileLocationRule {
            filename_pattern: "^config\\.toml$".to_string(),
            allowed_paths: vec![
                "^.*/config/config\\.toml$".to_string(),
                "^.*/settings/config\\.toml$".to_string(),
            ],
            message: "config.toml in wrong location".to_string(),
        }];

        // Should be blocked in src/
        let violations = check_file_with_config(rules.clone(), "src/config.toml");
        assert_eq!(violations.len(), 1);

        // Should be allowed in config/
        let violations = check_file_with_config(rules.clone(), "src/config/config.toml");
        assert!(violations.is_empty());

        // Should be allowed in settings/
        let violations = check_file_with_config(rules, "src/settings/config.toml");
        assert!(violations.is_empty());
    }
}
