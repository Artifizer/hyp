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
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash, Default)]
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
    pub fn get_checker_config<T>(&self, key: &str) -> T
    where
        T: serde::de::DeserializeOwned + Default,
    {
        self.checkers
            .get(key)
            .and_then(|v| serde_json::from_value(v.clone()).ok())
            .unwrap_or_default()
    }
}
