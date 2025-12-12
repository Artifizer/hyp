//! Configuration types with GTS schema support

use crate::violation::CheckerSeverity;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Category of a checker
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum CheckerCategory {
    /// Operations - not safe for production or performance
    Operations,
    /// Complexity - Rust or cognitive complexity patterns
    Complexity,
    /// Compliance - project-specific requirements
    Compliance,
}

impl CheckerCategory {
    /// Parse a category from a string.
    pub fn parse_category(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "operations" => Some(CheckerCategory::Operations),
            "complexity" => Some(CheckerCategory::Complexity),
            "compliance" => Some(CheckerCategory::Compliance),
            _ => None,
        }
    }

    /// Convert to string
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Operations => "operations",
            Self::Complexity => "complexity",
            Self::Compliance => "compliance",
        }
    }
}

/// Severity level configuration that can be deserialized from strings or integers
#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq, Hash, Default)]
#[serde(rename_all = "lowercase")]
pub enum SeverityLevel {
    /// Low severity (1)
    Low = 1,
    /// Medium severity (2)
    Medium = 2,
    /// High severity (3)
    #[default]
    High = 3,
}

// Custom deserializer to support both integers and strings
impl<'de> serde::Deserialize<'de> for SeverityLevel {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct SeverityVisitor;

        impl<'de> serde::de::Visitor<'de> for SeverityVisitor {
            type Value = SeverityLevel;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("an integer (1-3) or a string (low/medium/high)")
            }

            fn visit_i64<E>(self, value: i64) -> Result<SeverityLevel, E>
            where
                E: serde::de::Error,
            {
                match value {
                    1 => Ok(SeverityLevel::Low),
                    2 => Ok(SeverityLevel::Medium),
                    3 => Ok(SeverityLevel::High),
                    _ => Err(E::custom(format!("invalid severity level: {}", value))),
                }
            }

            fn visit_u64<E>(self, value: u64) -> Result<SeverityLevel, E>
            where
                E: serde::de::Error,
            {
                self.visit_i64(value as i64)
            }

            fn visit_str<E>(self, value: &str) -> Result<SeverityLevel, E>
            where
                E: serde::de::Error,
            {
                match value.to_lowercase().as_str() {
                    "low" => Ok(SeverityLevel::Low),
                    "medium" => Ok(SeverityLevel::Medium),
                    "high" => Ok(SeverityLevel::High),
                    _ => Err(E::custom(format!("invalid severity level: {}", value))),
                }
            }
        }

        deserializer.deserialize_any(SeverityVisitor)
    }
}

impl SeverityLevel {
    /// Convert to u8 representation
    pub fn as_u8(self) -> u8 {
        match self {
            Self::Low => 1,
            Self::Medium => 2,
            Self::High => 3,
        }
    }

    /// Convert from u8, returns None if invalid
    pub fn from_u8(value: u8) -> Option<Self> {
        match value {
            1 => Some(Self::Low),
            2 => Some(Self::Medium),
            3 => Some(Self::High),
            _ => None,
        }
    }
}

impl From<SeverityLevel> for CheckerSeverity {
    fn from(level: SeverityLevel) -> Self {
        match level {
            SeverityLevel::Low => CheckerSeverity::Low,
            SeverityLevel::Medium => CheckerSeverity::Medium,
            SeverityLevel::High => CheckerSeverity::High,
        }
    }
}

/// Main analyzer configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AnalyzerConfig {
    /// Per-checker configurations (raw YAML values)
    #[serde(default)]
    pub checkers: HashMap<String, serde_json::Value>,
}

impl AnalyzerConfig {
    /// Load configuration from YAML string (legacy support)
    pub fn from_yaml(yaml: &str) -> Result<Self, serde_yaml::Error> {
        serde_yaml::from_str(yaml)
    }

    /// Load configuration from TOML string
    pub fn from_toml(toml_str: &str) -> Result<Self, toml::de::Error> {
        toml::from_str(toml_str)
    }

    /// Get configuration for a specific checker by key
    ///
    /// This is a generic method that can retrieve configuration for any checker.
    /// The configuration type must implement `DeserializeOwned` and `Default`.
    ///
    /// # Errors
    /// Returns an error if the configuration exists but cannot be deserialized.
    pub fn get_checker_config<T>(&self, key: &str) -> Result<T, String>
    where
        T: serde::de::DeserializeOwned + Default,
    {
        match self.checkers.get(key) {
            Some(v) => serde_json::from_value::<T>(v.clone()).map_err(|e| {
                format!(
                    "Invalid configuration for checker '{}': {}",
                    key, e
                )
            }),
            None => Ok(T::default()),
        }
    }

    /// Get all configured checker keys
    pub fn configured_checker_keys(&self) -> impl Iterator<Item = &String> {
        self.checkers.keys()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_severity_level_from_integer() {
        let toml = r#"
            [checkers.test_checker]
            enabled = true
            severity = 2
        "#;
        let config = AnalyzerConfig::from_toml(toml).unwrap();

        #[allow(dead_code)]
        #[derive(Debug, Deserialize, Default)]
        struct TestConfig {
            #[serde(default)]
            enabled: bool,
            #[serde(default)]
            severity: SeverityLevel,
        }

        let checker_config: TestConfig = config.get_checker_config("test_checker").unwrap();
        assert_eq!(checker_config.severity, SeverityLevel::Medium);
    }

    #[test]
    fn test_severity_level_from_string() {
        let toml = r#"
            [checkers.test_checker]
            enabled = true
            severity = "low"
        "#;
        let config = AnalyzerConfig::from_toml(toml).unwrap();

        #[allow(dead_code)]
        #[derive(Debug, Deserialize, Default)]
        struct TestConfig {
            #[serde(default)]
            enabled: bool,
            #[serde(default)]
            severity: SeverityLevel,
        }

        let checker_config: TestConfig = config.get_checker_config("test_checker").unwrap();
        assert_eq!(checker_config.severity, SeverityLevel::Low);
    }

    #[test]
    fn test_severity_level_invalid_integer() {
        let toml = r#"
            [checkers.test_checker]
            enabled = true
            severity = 5
        "#;
        let config = AnalyzerConfig::from_toml(toml).unwrap();

        #[allow(dead_code)]
        #[derive(Debug, Deserialize, Default)]
        struct TestConfig {
            #[serde(default)]
            enabled: bool,
            #[serde(default)]
            severity: SeverityLevel,
        }

        let result: Result<TestConfig, _> = config.get_checker_config("test_checker");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("invalid severity level"));
    }

    #[test]
    fn test_config_wrong_type() {
        let toml = r#"
            [checkers.test_checker]
            enabled = "not_a_boolean"
        "#;
        let config = AnalyzerConfig::from_toml(toml).unwrap();

        #[allow(dead_code)]
        #[derive(Debug, Deserialize, Default)]
        struct TestConfig {
            enabled: bool,
        }

        let result: Result<TestConfig, _> = config.get_checker_config("test_checker");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("invalid type"));
    }

    #[test]
    fn test_config_missing_returns_default() {
        let toml = r#"
            [checkers.other_checker]
            enabled = true
        "#;
        let config = AnalyzerConfig::from_toml(toml).unwrap();

        #[allow(dead_code)]
        #[derive(Debug, Deserialize, Default)]
        struct TestConfig {
            #[serde(default)]
            enabled: bool,
        }

        let checker_config: TestConfig = config.get_checker_config("nonexistent").unwrap();
        assert!(!checker_config.enabled); // default is false
    }

    #[test]
    fn test_checker_category_parse() {
        assert_eq!(CheckerCategory::parse_category("operations"), Some(CheckerCategory::Operations));
        assert_eq!(CheckerCategory::parse_category("COMPLIANCE"), Some(CheckerCategory::Compliance));
        assert_eq!(CheckerCategory::parse_category("Complexity"), Some(CheckerCategory::Complexity));
        assert_eq!(CheckerCategory::parse_category("invalid"), None);
    }

    #[test]
    fn test_severity_level_as_u8() {
        assert_eq!(SeverityLevel::Low.as_u8(), 1);
        assert_eq!(SeverityLevel::Medium.as_u8(), 2);
        assert_eq!(SeverityLevel::High.as_u8(), 3);
    }

    #[test]
    fn test_severity_level_from_u8() {
        assert_eq!(SeverityLevel::from_u8(1), Some(SeverityLevel::Low));
        assert_eq!(SeverityLevel::from_u8(2), Some(SeverityLevel::Medium));
        assert_eq!(SeverityLevel::from_u8(3), Some(SeverityLevel::High));
        assert_eq!(SeverityLevel::from_u8(0), None);
        assert_eq!(SeverityLevel::from_u8(4), None);
    }
}
