//! Hyp Analyzer
//!
//! A Rust code analyzer for detecting common problems, anti-patterns,
//! and code quality issues.
//!
//! This crate provides:
//! - Static analysis of Rust code using syn
//! - Detection of unsafe code patterns
//! - Complexity metrics
//! - Error handling analysis
//! - Pluggable checker architecture
//! - GTS schema-based configuration
//!
//! # Example
//!
//! ```no_run
//! use hyp_analyzer::{Analyzer, config::AnalyzerConfig};
//! use std::path::Path;
//!
//! let analyzer = Analyzer::with_defaults().expect("Failed to create analyzer");
//! let violations = analyzer.analyze(Path::new("src")).unwrap();
//!
//! for violation in violations {
//!     println!("{}: {} at {}:{}",
//!         violation.code,
//!         violation.message,
//!         violation.file_path,
//!         violation.line
//!     );
//! }
//! ```

#![warn(missing_docs)]

use thiserror::Error;

pub mod analyzer;
pub mod checker;
pub mod checker_config_macro;
pub mod checkers;
pub mod cli_helper;
pub mod config;
pub mod registry;
pub mod violation;

pub use analyzer::{Analyzer, AnalyzerFilters, CheckerGuideline, CheckerInfo};
pub use checker::{Checker, CheckerDescriptor, ItemType};
pub use cli_helper::{
    filter_registrations_with_config, find_config_file, load_config, parse_categories,
    print_default_config, print_validation_results, run_validation, split_csv, CliOptions,
    CliOutputFormat, FunctionValidation, ValidationSummary,
};
pub use config::{AnalyzerConfig, CheckerCategory, SeverityLevel};
pub use registry::{get_all_checkers, CheckerFactory, CheckerRegistration};
pub use violation::{CheckerSeverity, Severity, Violation};

// Re-export checker configs for convenience
pub use checkers::e10_unsafe_code::{
    E1001Config, E1002Config, E1003Config, E1007Config, E1008Config, E1010Config,
    E1013Config, E1014Config, E1015Config, E1016Config,
};
pub use checkers::e11_code_surface_complexity::{
    E1101Config, E1102Config, E1103Config, E1104Config, E1105Config, E1106Config, E1107Config,
    E1108Config, E1109Config,
};
pub use checkers::e12_code_pattern_complexity::{
    E1201Config, E1203Config, E1204Config, E1210Config, E1211Config, E1213Config, E1217Config,
};
pub use checkers::e13_error_handling::{
    E1301Config, E1302Config, E1303Config, E1304Config, E1305Config, E1306Config, E1307Config,
    E1308Config, E1310Config,
};
pub use checkers::e14_type_safety::{
    E1401Config, E1402Config, E1403Config, E1404Config, E1405Config, E1406Config, E1407Config,
    E1408Config, E1409Config,
};
pub use checkers::e15_concurrency::{E1503Config, E1506Config, E1508Config, E1509Config, E1510Config};
pub use checkers::e16_memory_safety::{
    E1603Config, E1604Config, E1605Config, E1606Config, E1607Config, E1609Config, E1610Config,
};
pub use checkers::e17_performance::{
    E1701Config, E1702Config, E1703Config, E1704Config, E1705Config, E1706Config, E1707Config,
    E1708Config, E1709Config, E1710Config,
};
pub use checkers::e18_api_design::{
    E1801Config, E1802Config, E1803Config, E1804Config, E1805Config, E1806Config, E1807Config,
    E1808Config, E1809Config, E1810Config,
};
pub use checkers::e19_hygiene::{
    E1901Config, E1902Config, E1903Config, E1904Config,
};

/// Errors that can occur during code analysis
#[derive(Error, Debug)]
pub enum AnalyzerError {
    /// IO error while reading files
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// Parse error while analyzing code
    #[error("Parse error: {0}")]
    Parse(String),

    /// Configuration error
    #[error("Configuration error: {0}")]
    Config(String),

    /// Serialization error
    #[error("Serialization error: {0}")]
    Serialization(String),
}

impl From<serde_yaml::Error> for AnalyzerError {
    fn from(err: serde_yaml::Error) -> Self {
        AnalyzerError::Config(err.to_string())
    }
}

impl From<serde_json::Error> for AnalyzerError {
    fn from(err: serde_json::Error) -> Self {
        AnalyzerError::Serialization(err.to_string())
    }
}

/// Result type for analyzer operations
pub type Result<T> = std::result::Result<T, AnalyzerError>;
