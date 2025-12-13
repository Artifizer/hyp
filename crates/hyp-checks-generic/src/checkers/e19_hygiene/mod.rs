//! E19: Code Hygiene
//!
//! Checkers that enforce project-specific conventions, naming rules, and code organization.
//! These checkers help maintain consistent code structure and prevent bypassing of project standards.

pub mod e1901_critical_lint_override;
pub mod e1902_medium_lint_override;
pub mod e1903_minor_lint_override;
pub mod e1904_allowed_names;
pub mod e1905_suspicious_code;
pub mod e1906_file_location;
pub mod e1907_test_coverage_attr;
pub mod e1908_unsafe_justification;

pub use e1901_critical_lint_override::{E1901Config, E1901CriticalLintOverride};
pub use e1902_medium_lint_override::{E1902Config, E1902MediumLintOverride};
pub use e1903_minor_lint_override::{E1903Config, E1903MinorLintOverride};
pub use e1904_allowed_names::{E1904AllowedNames, E1904Config};
pub use e1905_suspicious_code::{E1905Config, E1905SuspiciousCode};
pub use e1906_file_location::{E1906Config, E1906FileLocation};
pub use e1907_test_coverage_attr::{E1907Config, E1907TestCoverageAttr};
pub use e1908_unsafe_justification::{E1908Config, E1908UnsafeJustification};

pub mod registry;
