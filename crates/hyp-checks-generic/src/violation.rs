//! Violation types and reporting

use serde::{Deserialize, Serialize};

/// Severity level of a checker (1-3)
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[repr(u8)]
pub enum CheckerSeverity {
    /// Severity 1 - Informational, style issues
    Low = 1,
    /// Severity 2 - Moderate issues, potential problems
    Medium = 2,
    /// Severity 3 - Critical issues, unsafe patterns
    High = 3,
}

impl CheckerSeverity {
    /// Parse from u8 (1-3)
    pub fn from_u8(value: u8) -> Option<Self> {
        match value {
            1 => Some(Self::Low),
            2 => Some(Self::Medium),
            3 => Some(Self::High),
            _ => None,
        }
    }

    /// Convert to u8
    pub fn as_u8(&self) -> u8 {
        match self {
            Self::Low => 1,
            Self::Medium => 2,
            Self::High => 3,
        }
    }
}

/// Severity level of a violation (for backward compatibility)
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Severity {
    /// Low severity - style or minor issues
    Low,
    /// Medium severity - potential problems
    Medium,
    /// High severity - likely bugs or serious issues
    High,
}

impl From<CheckerSeverity> for Severity {
    fn from(checker_severity: CheckerSeverity) -> Self {
        match checker_severity {
            CheckerSeverity::Low => Severity::Low,
            CheckerSeverity::Medium => Severity::Medium,
            CheckerSeverity::High => Severity::High,
        }
    }
}

/// A detected code violation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Violation {
    /// Problem code (e.g., "E1001")
    pub code: String,

    /// Human-readable name
    pub name: String,

    /// Severity level
    pub severity: Severity,

    /// Description of the problem
    pub message: String,

    /// File path where violation was found
    pub file_path: String,

    /// Line number (1-indexed)
    pub line: usize,

    /// Column number (1-indexed)
    pub column: usize,

    /// Optional suggestion for fixing
    pub suggestion: Option<String>,
}

impl Violation {
    /// Create a new violation
    pub fn new(
        code: impl Into<String>,
        name: impl Into<String>,
        severity: Severity,
        message: impl Into<String>,
        file_path: impl Into<String>,
        line: usize,
        column: usize,
    ) -> Self {
        Self {
            code: code.into(),
            name: name.into(),
            severity,
            message: message.into(),
            file_path: file_path.into(),
            line,
            column,
            suggestion: None,
        }
    }

    /// Add a suggestion to this violation
    pub fn with_suggestion(mut self, suggestion: impl Into<String>) -> Self {
        self.suggestion = Some(suggestion.into());
        self
    }
}
