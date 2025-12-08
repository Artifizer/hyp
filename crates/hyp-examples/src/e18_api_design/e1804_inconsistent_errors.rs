/// E1804: Inconsistent error types
/// Severity: MED
/// LLM confusion: 2 (LOW)
///
/// Description: Using different error types for similar functions makes error handling inconsistent
/// and difficult. One function returns `io::Error`, another returns `String`. Callers can't handle
/// errors uniformly. Fix by using consistent error types across your API - define a custom error
/// type or use a common error type.
///
/// ## The Mixed Error Types Problem
///
/// ```text
/// fn read_config() -> Result<Config, io::Error> { ... }
/// fn parse_config(s: &str) -> Result<Config, String> { ... }
/// fn validate_config(c: &Config) -> Result<(), Box<dyn Error>> { ... }
///
/// // Caller has to handle 3 different error types!
/// ```
///
/// ## Why This Matters
///
/// 1. **Inconsistent handling**: Different match arms for each type
/// 2. **No unified error chain**: Can't use `?` easily
/// 3. **Poor error messages**: String errors lose context
/// 4. **API confusion**: Users don't know what to expect
///
/// ## The Right Solutions
///
/// ### Option 1: Custom error enum with thiserror
/// ```rust
/// #[derive(thiserror::Error, Debug)]
/// pub enum ConfigError {
///     #[error("IO error: {0}")]
///     Io(#[from] std::io::Error),
///     #[error("Parse error: {0}")]
///     Parse(String),
///     #[error("Validation error: {0}")]
///     Validation(String),
/// }
/// ```
///
/// ### Option 2: Use anyhow for applications
/// ```rust
/// use anyhow::Result;
///
/// fn read_config() -> Result<Config> {
///     let content = std::fs::read_to_string("config.toml")?;
///     let config = parse(&content)?;
///     Ok(config)
/// }
/// ```
///
/// Mitigation: Define a custom error enum for your crate using `thiserror`. Use the same error
/// type for all functions in a module or crate. Use `anyhow` for applications where you just need
/// to propagate errors. Consistent error types make APIs easier to use.

use std::fmt;

// ============================================================================
// PROBLEMATIC PATTERNS
// ============================================================================

/// PROBLEM E1804: Returns io::Error
pub fn e1804_bad_io_error() -> Result<i32, std::io::Error> {
    Ok(42)
}

/// PROBLEM E1804: Returns String (inconsistent with above)
pub fn e1804_bad_string_error() -> Result<i32, String> {
    Ok(42)
}

/// PROBLEM E1804: Returns Box<dyn Error> (yet another type)
pub fn e1804_bad_boxed_error() -> Result<i32, Box<dyn std::error::Error>> {
    Ok(42)
}

/// PROBLEM E1804: Returns custom error (inconsistent)
#[derive(Debug)]
pub struct CustomError(String);

impl fmt::Display for CustomError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::error::Error for CustomError {}

pub fn e1804_bad_custom_error() -> Result<i32, CustomError> {
    Ok(42)
}

/// Entry point for problem demonstration
pub fn e1804_entry() -> Result<(), Box<dyn std::error::Error>> {
    // Caller has to handle all different error types!
    let _ = e1804_bad_io_error()?;
    let _ = e1804_bad_string_error().map_err(|e| Box::new(CustomError(e)))?;
    let _ = e1804_bad_boxed_error()?;
    Ok(())
}

// ============================================================================
// GOOD ALTERNATIVES
// ============================================================================

/// GOOD: Unified error type for the module
#[derive(Debug)]
pub enum AppError {
    Io(std::io::Error),
    Parse(String),
    Validation(String),
    NotFound(String),
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AppError::Io(e) => write!(f, "IO error: {}", e),
            AppError::Parse(msg) => write!(f, "Parse error: {}", msg),
            AppError::Validation(msg) => write!(f, "Validation error: {}", msg),
            AppError::NotFound(msg) => write!(f, "Not found: {}", msg),
        }
    }
}

impl std::error::Error for AppError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            AppError::Io(e) => Some(e),
            _ => None,
        }
    }
}

// Implement From for easy conversion with ?
impl From<std::io::Error> for AppError {
    fn from(e: std::io::Error) -> Self {
        AppError::Io(e)
    }
}

/// Type alias for convenience
pub type AppResult<T> = Result<T, AppError>;

/// GOOD: All functions use consistent error type
pub fn e1804_good_read() -> AppResult<String> {
    // Can use ? with io::Error thanks to From impl
    Ok("data".to_string())
}

pub fn e1804_good_parse(data: &str) -> AppResult<i32> {
    data.parse().map_err(|_| AppError::Parse("Invalid integer".to_string()))
}

pub fn e1804_good_validate(value: i32) -> AppResult<i32> {
    if value < 0 {
        return Err(AppError::Validation("Value must be non-negative".to_string()));
    }
    Ok(value)
}

/// GOOD: Easy error chaining with consistent types
pub fn e1804_good_pipeline() -> AppResult<i32> {
    let data = e1804_good_read()?;
    let parsed = e1804_good_parse(&data)?;
    let validated = e1804_good_validate(parsed)?;
    Ok(validated)
}

/// GOOD: Using Result type alias
pub fn e1804_good_with_alias() -> AppResult<()> {
    let _ = e1804_good_read()?;
    Ok(())
}

/// GOOD: Error with context
impl AppError {
    pub fn with_context(self, context: &str) -> Self {
        match self {
            AppError::Parse(msg) => AppError::Parse(format!("{}: {}", context, msg)),
            AppError::Validation(msg) => AppError::Validation(format!("{}: {}", context, msg)),
            AppError::NotFound(msg) => AppError::NotFound(format!("{}: {}", context, msg)),
            other => other,
        }
    }
}

// ============================================================================
// GOOD EXAMPLES unit tests
// ============================================================================


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_consistent_errors() {
        let result = e1804_good_read();
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_error() {
        let result = e1804_good_parse("not a number");
        assert!(matches!(result, Err(AppError::Parse(_))));
    }

    #[test]
    fn test_validation_error() {
        let result = e1804_good_validate(-1);
        assert!(matches!(result, Err(AppError::Validation(_))));
    }

    #[test]
    fn test_error_display() {
        let err = AppError::Parse("bad input".to_string());
        assert_eq!(format!("{}", err), "Parse error: bad input");
    }

    #[test]
    fn test_error_with_context() {
        let err = AppError::Parse("bad input".to_string()).with_context("config file");
        assert!(format!("{}", err).contains("config file"));
    }
}
