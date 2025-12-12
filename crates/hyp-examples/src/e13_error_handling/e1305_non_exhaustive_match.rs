/// E1305: Non-exhaustive match on Result/Option
/// Severity: HIGH
/// LLM confusion: 2 (LOW)
///
/// Description: This code uses a wildcard pattern (`_`) to catch all error cases, which means
/// errors are not properly examined or handled. Different errors might need different responses
/// (like retrying vs. giving up), but this treats them all the same. It's like having a try/catch
/// that just returns a default value for any error without checking what went wrong. Fix by
/// explicitly matching on error types or at least logging the error before providing a default.
///
/// ## The Blanket Catch Problem
///
/// ```text
/// match result {
///     Ok(v) => v,
///     _ => 0,  // What error? Network? Parse? Permission? Who knows!
/// }
/// ```
///
/// ## Why This Matters
///
/// 1. **Lost information**: You can't handle different errors differently
/// 2. **Silent failures**: Serious errors get the same treatment as minor ones
/// 3. **Debugging difficulty**: No way to know what actually failed
/// 4. **Incorrect recovery**: A "retry-able" error gets same handling as fatal error
///
/// ## The Right Solutions
///
/// ### Option 1: Match explicitly on error cases
/// ```rust
/// match result {
///     Ok(v) => v,
///     Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
///         eprintln!("File not found, using default");
///         0
///     }
///     Err(e) => {
///         eprintln!("Unexpected error: {}", e);
///         0
///     }
/// }
/// ```
///
/// ### Option 2: Use unwrap_or_else to see the error
/// ```rust
/// result.unwrap_or_else(|e| {
///     eprintln!("Error occurred: {}", e);
///     0
/// })
/// ```
///
/// ### Option 3: Log before defaulting
/// ```rust
/// result.inspect_err(|e| eprintln!("Warning: {}", e)).unwrap_or(0)
/// ```
///
/// Mitigation: Avoid using `_` to catch Result/Option error cases. Use explicit pattern matching
/// or combinators like `unwrap_or_else()` that give you access to the error value. Log errors
/// before converting them to default values.

// ============================================================================
// DANGEROUS PATTERNS - NEVER DO THIS
// ============================================================================

/// PROBLEM E1305: Wildcard catches all errors without examination
#[allow(clippy::manual_unwrap_or_default)]
pub fn e1305_bad_non_exhaustive_match(result: Result<i32, String>) -> i32 {
    // PROBLEM E1305: Not handling all cases
    result.unwrap_or(0)
}

/// PROBLEM E1305: Same problem with Option
pub fn e1305_bad_option_wildcard(opt: Option<String>) -> String {
    match opt {
        Some(s) => s,
        _ => String::new(), // Why was it None? We'll never know
    }
}

/// PROBLEM E1305: Nested wildcards hide multiple failure points
pub fn e1305_bad_nested_wildcards(data: Result<Option<i32>, String>) -> i32 {
    match data {
        Ok(Some(v)) => v,
        _ => -1, // Was it Err? Was it Ok(None)? No way to tell
    }
}

/// Entry point for problem demonstration
pub fn e1305_entry() -> Result<(), Box<dyn std::error::Error>> {
    let result: Result<i32, String> = Ok(42);
    let _ = e1305_bad_non_exhaustive_match(result);
    Ok(())
}

// ============================================================================
// GOOD ALTERNATIVES
// ============================================================================

/// GOOD: Explicit error matching with logging
pub fn e1305_good_explicit_match(result: Result<i32, String>) -> i32 {
    match result {
        Ok(v) => v,
        Err(e) => {
            eprintln!("Error occurred: {}", e);
            0
        }
    }
}

/// GOOD: Use unwrap_or_else to see the error
pub fn e1305_good_unwrap_or_else(result: Result<i32, String>) -> i32 {
    result.unwrap_or_else(|e| {
        eprintln!("Falling back to default due to: {}", e);
        0
    })
}

/// GOOD: Explicit Option handling
pub fn e1305_good_option_explicit(opt: Option<String>) -> String {
    match opt {
        Some(s) => s,
        None => {
            eprintln!("No value provided, using empty string");
            String::new()
        }
    }
}

/// GOOD: Handle different error types differently
#[derive(Debug)]
pub enum ProcessError {
    NotFound,
    PermissionDenied,
    InvalidFormat(String),
}

pub fn e1305_good_typed_errors(result: Result<i32, ProcessError>) -> i32 {
    match result {
        Ok(v) => v,
        Err(ProcessError::NotFound) => {
            eprintln!("Resource not found, using default");
            0
        }
        Err(ProcessError::PermissionDenied) => {
            eprintln!("Permission denied - this might be a configuration issue");
            -1
        }
        Err(ProcessError::InvalidFormat(msg)) => {
            eprintln!("Invalid format: {} - check input data", msg);
            -2
        }
    }
}

/// GOOD: Nested results with explicit handling
pub fn e1305_good_nested_explicit(data: Result<Option<i32>, String>) -> i32 {
    match data {
        Ok(Some(v)) => v,
        Ok(None) => {
            eprintln!("Value was None (missing data)");
            0
        }
        Err(e) => {
            eprintln!("Error fetching data: {}", e);
            -1
        }
    }
}

/// GOOD: Use inspect_err before defaulting
pub fn e1305_good_inspect_err(result: Result<i32, String>) -> i32 {
    result
        .inspect_err(|e| eprintln!("Warning: {}", e))
        .unwrap_or(0)
}

// ============================================================================
// GOOD EXAMPLES unit tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_good_explicit_match_ok() {
        let result = e1305_good_explicit_match(Ok(42));
        assert_eq!(result, 42);
    }

    #[test]
    fn test_good_explicit_match_err() {
        let result = e1305_good_explicit_match(Err("error".to_string()));
        assert_eq!(result, 0);
    }

    #[test]
    fn test_good_typed_errors_not_found() {
        let result = e1305_good_typed_errors(Err(ProcessError::NotFound));
        assert_eq!(result, 0);
    }

    #[test]
    fn test_good_typed_errors_permission() {
        let result = e1305_good_typed_errors(Err(ProcessError::PermissionDenied));
        assert_eq!(result, -1);
    }

    #[test]
    fn test_good_nested_explicit_none() {
        let result = e1305_good_nested_explicit(Ok(None));
        assert_eq!(result, 0);
    }

    #[test]
    fn test_good_nested_explicit_err() {
        let result = e1305_good_nested_explicit(Err("failed".to_string()));
        assert_eq!(result, -1);
    }
}
