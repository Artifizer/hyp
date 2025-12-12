//! Checker trait and metadata types

use crate::{config::CheckerCategory, violation::CheckerSeverity, Result, Violation};
use syn::Item;

/// Descriptor for registering a checker with default metadata
#[derive(Debug, Clone)]
pub struct CheckerDescriptor {
    /// Unique identifier (e.g., "E1001")
    pub code: String,
    /// Human-readable name
    pub name: String,
    /// Default severity (can be overridden in config)
    pub default_severity: CheckerSeverity,
    /// Default categories (can be overridden in config)
    pub default_categories: Vec<CheckerCategory>,
}

/// Types of AST items that checkers can analyze
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ItemType {
    /// Function or method
    Function,
    /// Struct definition
    Struct,
    /// Enum definition
    Enum,
    /// Trait definition
    Trait,
    /// Impl block
    Impl,
    /// Module
    Module,
    /// Const item
    Const,
    /// Static item
    Static,
    /// Union definition
    Union,
    /// Type alias
    Type,
    /// Use statement
    Use,
}

/// Core trait for all checkers.
/// This trait is object-safe and can be used with `Box<dyn Checker>`.
pub trait Checker: Send + Sync {
    /// Unique identifier for this checker (e.g., "E1001")
    fn code(&self) -> &str;

    /// Human-readable name
    fn name(&self) -> &str;

    /// Suggestion for how to fix violations from this checker
    fn suggestions(&self) -> &str;

    /// Severity level (1-3)
    fn severity(&self) -> CheckerSeverity;

    /// Categories this checker belongs to (at least one)
    fn categories(&self) -> &[CheckerCategory];

    /// Types of AST items this checker analyzes
    fn target_items(&self) -> &[ItemType];

    /// Check an AST item and return any violations found
    fn check_item(&self, item: &Item, file_path: &str) -> Result<Vec<Violation>>;

    /// Whether this checker is enabled
    fn is_enabled(&self) -> bool {
        true
    }

    /// Update the configuration for this checker
    fn set_config(&mut self, _config: Box<dyn std::any::Any>) -> Result<()> {
        Ok(())
    }

    /// Get the descriptor for this checker (built from trait methods)
    fn descriptor(&self) -> CheckerDescriptor {
        CheckerDescriptor {
            code: self.code().to_string(),
            name: self.name().to_string(),
            default_severity: self.severity(),
            default_categories: self.categories().to_vec(),
        }
    }
}
