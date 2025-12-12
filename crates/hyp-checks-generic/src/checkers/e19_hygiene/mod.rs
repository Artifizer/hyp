//! E19: Code Hygiene
//!
//! Checkers that enforce project-specific conventions, naming rules, and code organization.
//! These checkers help maintain consistent code structure and prevent bypassing of project standards.

pub mod e1901_allowed_names;
pub mod e1902_inline_directives;
pub mod e1903_file_location;
pub mod e1904_unsafe_justification;

pub use e1901_allowed_names::{E1901AllowedNames, E1901Config};
pub use e1902_inline_directives::{E1902Config, E1902InlineDirectives};
pub use e1903_file_location::{E1903Config, E1903FileLocation};
pub use e1904_unsafe_justification::{E1904Config, E1904UnsafeJustification};

pub mod registry;
