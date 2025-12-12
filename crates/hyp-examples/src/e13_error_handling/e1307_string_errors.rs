/// E1307: Using String for error types
/// Severity: MED
/// LLM confusion: 2 (LOW)
///
/// Description: Using plain strings as error types loses important information about what kind of
/// error occurred. Strings can't be pattern-matched or handled differently based on error type,
/// and they don't follow standard error conventions. It's like throwing exceptions with just a
/// message string instead of specific exception classes - you can't catch specific errors to
/// handle them differently. Fix by creating proper error enums or using existing error types.
///
/// ## The Stringly-Typed Problem
///
/// ```text
/// fn process() -> Result<(), String> {
///     Err("Something went wrong".to_string())
/// }
///
/// // Caller can't do this:
/// match result {
///     Err(NetworkError::Timeout) => retry(),     // Can't!
///     Err(NetworkError::NotFound) => skip(),     // Can't!
///     Err(msg) => println!("{}", msg),           // Only option
/// }
/// ```
///
/// ## Why This Matters
///
/// 1. **No programmatic handling**: Can't match on error types
/// 2. **Lost semantics**: "connection failed" vs NetworkError::ConnectionFailed
/// 3. **No error chaining**: Can't use source() or cause()
/// 4. **Poor API design**: Callers can't handle specific errors
///
/// ## The Right Solutions
///
/// ### Option 1: Use thiserror for custom errors
/// ```rust
/// use thiserror::Error;
///
/// #[derive(Error, Debug)]
/// pub enum ConfigError {
///     #[error("File not found: {0}")]
///     NotFound(String),
///     #[error("Parse error: {0}")]
///     ParseError(#[from] std::num::ParseIntError),
/// }
/// ```
///
/// ### Option 2: Simple error enum
/// ```rust
/// #[derive(Debug)]
/// pub enum AppError {
///     NotFound,
///     InvalidInput,
///     NetworkError(String),
/// }
/// ```
///
/// ### Option 3: Use anyhow for applications
/// ```rust,no_run
/// use anyhow::{Context, Result};
///
/// fn process() -> Result<()> {
///     let content = std::fs::read_to_string("file.txt")
///         .context("Failed to read config file")?;
///     Ok(())
/// }
/// ```
///
/// Mitigation: Use `#![warn(clippy::string_slice_as_bytes)]` and create custom error enums or
/// use the `thiserror` crate to derive Error implementations. String errors should only be used
/// in quick prototypes, never in production code.

// ============================================================================
// DANGEROUS PATTERNS - NEVER DO THIS
// ============================================================================

/// PROBLEM E1307: String errors lose type information
pub fn e1307_bad_string_errors() -> Result<i32, String> {
    // PROBLEM E1307: String errors lose type information
    Err("Something went wrong".to_string())
}

/// PROBLEM E1307: format! for errors loses structure
pub fn e1307_bad_format_error(value: i32) -> Result<i32, String> {
    if value < 0 {
        Err(format!("Value {} is negative", value))
    } else {
        Ok(value)
    }
}

/// PROBLEM E1307: Converting typed errors to strings
pub fn e1307_bad_to_string_error() -> Result<String, String> {
    std::fs::read_to_string("config.txt").map_err(|e| e.to_string())
}

/// Entry point for problem demonstration
pub fn e1307_entry() -> Result<(), Box<dyn std::error::Error>> {
    let _ = e1307_bad_string_errors();
    Ok(())
}

// ============================================================================
// GOOD ALTERNATIVES
// ============================================================================

/// GOOD: Proper error enum
#[derive(Debug, PartialEq)]
pub enum ValidationError {
    Negative(i32),
    TooLarge(i32),
    Empty,
}

impl std::fmt::Display for ValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ValidationError::Negative(v) => write!(f, "Value {} is negative", v),
            ValidationError::TooLarge(v) => write!(f, "Value {} exceeds maximum", v),
            ValidationError::Empty => write!(f, "Value is empty"),
        }
    }
}

impl std::error::Error for ValidationError {}

/// GOOD: Return typed error
pub fn e1307_good_typed_error(value: i32) -> Result<i32, ValidationError> {
    if value < 0 {
        Err(ValidationError::Negative(value))
    } else if value > 1000 {
        Err(ValidationError::TooLarge(value))
    } else {
        Ok(value)
    }
}

/// GOOD: Error enum with source
#[derive(Debug)]
pub enum ConfigError {
    IoError(std::io::Error),
    ParseError(std::num::ParseIntError),
    MissingField(String),
}

impl std::fmt::Display for ConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConfigError::IoError(e) => write!(f, "IO error: {}", e),
            ConfigError::ParseError(e) => write!(f, "Parse error: {}", e),
            ConfigError::MissingField(field) => write!(f, "Missing field: {}", field),
        }
    }
}

impl std::error::Error for ConfigError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            ConfigError::IoError(e) => Some(e),
            ConfigError::ParseError(e) => Some(e),
            ConfigError::MissingField(_) => None,
        }
    }
}

impl From<std::io::Error> for ConfigError {
    fn from(e: std::io::Error) -> Self {
        ConfigError::IoError(e)
    }
}

impl From<std::num::ParseIntError> for ConfigError {
    fn from(e: std::num::ParseIntError) -> Self {
        ConfigError::ParseError(e)
    }
}

/// GOOD: Preserve typed errors
pub fn e1307_good_preserve_error() -> Result<String, ConfigError> {
    let content = std::fs::read_to_string("config.txt")?;
    Ok(content)
}

/// GOOD: Callers can match on error type
pub fn e1307_good_handle_errors() {
    match e1307_good_typed_error(-5) {
        Ok(v) => println!("Value: {}", v),
        Err(ValidationError::Negative(_)) => println!("Using absolute value"),
        Err(ValidationError::TooLarge(_)) => println!("Capping at maximum"),
        Err(ValidationError::Empty) => println!("Using default"),
    }
}

/// GOOD: Use Box<dyn Error> for mixed error types
pub fn e1307_good_boxed_error() -> Result<i32, Box<dyn std::error::Error>> {
    let content = std::fs::read_to_string("number.txt")?;
    let num: i32 = content.trim().parse()?;
    Ok(num)
}

// ============================================================================
// GOOD EXAMPLES unit tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_good_typed_error_negative() {
        let result = e1307_good_typed_error(-5);
        assert_eq!(result, Err(ValidationError::Negative(-5)));
    }

    #[test]
    fn test_good_typed_error_too_large() {
        let result = e1307_good_typed_error(2000);
        assert_eq!(result, Err(ValidationError::TooLarge(2000)));
    }

    #[test]
    fn test_good_typed_error_valid() {
        let result = e1307_good_typed_error(500);
        assert_eq!(result, Ok(500));
    }

    #[test]
    fn test_good_preserve_error_is_io_error() {
        let result = e1307_good_preserve_error();
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), ConfigError::IoError(_)));
    }

    #[test]
    fn test_error_display() {
        let err = ValidationError::Negative(-10);
        assert_eq!(format!("{}", err), "Value -10 is negative");
    }
}
