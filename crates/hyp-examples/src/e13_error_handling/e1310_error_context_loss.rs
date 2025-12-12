/// E1310: Error context loss
/// Severity: MED
/// LLM confusion: 3 (MED)
///
/// Description: This code discards the original error (using `|_|`) when converting it to a
/// different error type, losing valuable debugging information like which file failed to open,
/// why it failed (permissions? doesn't exist?), etc. It's like catching an exception and throwing
/// a new one with just a generic message, losing the original stack trace and details. Fix by
/// preserving the original error in the new error, either by wrapping it or including its message.
///
/// ## The Lost Context Problem
///
/// ```text
/// std::fs::read_to_string("config.txt")
///     .map_err(|_| "Failed to read file")?
///         ↓
/// Error: "Failed to read file"
///         ↓
/// But WHY? Permission denied? Not found? Disk full? No way to know!
/// ```
///
/// ## Why This Matters
///
/// 1. **Lost debugging info**: The actual error is thrown away
/// 2. **Support nightmare**: Users report "Failed to read file" with no details
/// 3. **No error chain**: source()/cause() returns nothing
/// 4. **Harder fixes**: Can't distinguish between different failure modes
///
/// ## The Right Solutions
///
/// ### Option 1: Include original error in message
/// ```rust,no_run
/// std::fs::read_to_string("config.txt")
///     .map_err(|e| format!("Failed to read config: {}", e))?;
/// ```
///
/// ### Option 2: Wrap error with context (anyhow)
/// ```rust,no_run
/// use anyhow::Context;
///
/// std::fs::read_to_string("config.txt")
///     .context("Failed to read config file")?;
/// ```
///
/// ### Option 3: Create error enum with source
/// ```rust
/// #[derive(Debug)]
/// enum AppError {
///     ConfigRead(std::io::Error),
/// }
///
/// impl std::error::Error for AppError {
///     fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
///         match self {
///             AppError::ConfigRead(e) => Some(e),
///         }
///     }
/// }
/// # impl std::fmt::Display for AppError {
/// #     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
/// #         write!(f, "config error")
/// #     }
/// # }
/// ```
///
/// Mitigation: Use `map_err(|e| ...)` to preserve error context. Use the `anyhow` crate for
/// applications or `thiserror` for libraries to maintain error chains. Never use `|_|` to
/// discard errors - always preserve the original error information.

// ============================================================================
// DANGEROUS PATTERNS - NEVER DO THIS
// ============================================================================

/// PROBLEM E1310: Original error context is lost
pub fn e1310_bad_context_loss() -> Result<i32, Box<dyn std::error::Error>> {
    // PROBLEM E1310: Original error context is lost
    let data = std::fs::read_to_string("config.txt").map_err(|_| "Failed to read file")?;
    Ok(data.len() as i32)
}

/// PROBLEM E1310: Discarding error in map_err
pub fn e1310_bad_discard_error() -> Result<String, String> {
    std::fs::read_to_string("data.txt")
        .map_err(|_| "Read failed".to_string()) // What failed? Why?
}

/// PROBLEM E1310: Generic error message hides details
pub fn e1310_bad_generic_message() -> Result<i32, String> {
    let content = std::fs::read_to_string("number.txt").map_err(|_| "Error".to_string())?;
    content.trim().parse().map_err(|_| "Error".to_string()) // Two different errors, same message!
}

/// Entry point for problem demonstration
pub fn e1310_entry() -> Result<(), Box<dyn std::error::Error>> {
    let _ = e1310_bad_context_loss();
    Ok(())
}

// ============================================================================
// GOOD ALTERNATIVES
// ============================================================================

/// GOOD: Include original error in message
pub fn e1310_good_include_error() -> Result<i32, String> {
    let data = std::fs::read_to_string("config.txt")
        .map_err(|e| format!("Failed to read config: {}", e))?;
    Ok(data.len() as i32)
}

/// GOOD: Error enum that wraps original error
#[derive(Debug)]
pub enum ConfigError {
    ReadError { path: String, source: std::io::Error },
    ParseError { content: String, source: std::num::ParseIntError },
}

impl std::fmt::Display for ConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConfigError::ReadError { path, source } => {
                write!(f, "Failed to read '{}': {}", path, source)
            }
            ConfigError::ParseError { content, source } => {
                write!(f, "Failed to parse '{}': {}", content, source)
            }
        }
    }
}

impl std::error::Error for ConfigError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            ConfigError::ReadError { source, .. } => Some(source),
            ConfigError::ParseError { source, .. } => Some(source),
        }
    }
}

/// GOOD: Preserve error with context
pub fn e1310_good_with_context(path: &str) -> Result<i32, ConfigError> {
    let content = std::fs::read_to_string(path).map_err(|e| ConfigError::ReadError {
        path: path.to_string(),
        source: e,
    })?;

    let num = content.trim().parse().map_err(|e| ConfigError::ParseError {
        content: content.clone(),
        source: e,
    })?;

    Ok(num)
}

/// GOOD: Chain errors with Box<dyn Error>
pub fn e1310_good_boxed_chain() -> Result<i32, Box<dyn std::error::Error>> {
    let content = std::fs::read_to_string("config.txt")
        .map_err(|e| -> Box<dyn std::error::Error> {
            format!("Config read failed: {}", e).into()
        })?;

    let num: i32 = content.trim().parse().map_err(|e| -> Box<dyn std::error::Error> {
        format!("Config parse failed (content: '{}'): {}", content.trim(), e).into()
    })?;

    Ok(num)
}

/// GOOD: Add file path context
pub fn e1310_good_path_context(path: &str) -> Result<String, String> {
    std::fs::read_to_string(path)
        .map_err(|e| format!("Failed to read '{}': {} (kind: {:?})", path, e, e.kind()))
}

/// GOOD: Distinguish between error types in message
pub fn e1310_good_distinguish_errors() -> Result<i32, String> {
    let content = std::fs::read_to_string("number.txt")
        .map_err(|e| format!("IO error reading file: {}", e))?;

    content
        .trim()
        .parse()
        .map_err(|e| format!("Parse error (input: '{}'): {}", content.trim(), e))
}

// ============================================================================
// GOOD EXAMPLES unit tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use std::error::Error;

    #[test]
    fn test_good_include_error_has_details() {
        let result = e1310_good_include_error();
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.contains("Failed to read config"));
        assert!(err.contains("No such file")); // Original error included
    }

    #[test]
    fn test_good_with_context_preserves_source() {
        let result = e1310_good_with_context("nonexistent.txt");
        assert!(result.is_err());
        let err = result.unwrap_err();

        // Check that source() works
        assert!(err.source().is_some());
    }

    #[test]
    fn test_good_path_context_includes_path() {
        let result = e1310_good_path_context("missing_file.txt");
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.contains("missing_file.txt"));
    }

    #[test]
    fn test_config_error_display() {
        let err = ConfigError::ReadError {
            path: "test.txt".to_string(),
            source: std::io::Error::new(std::io::ErrorKind::NotFound, "not found"),
        };
        let msg = format!("{}", err);
        assert!(msg.contains("test.txt"));
        assert!(msg.contains("not found"));
    }
}
