/// E1306: Swallowing errors without logging
/// Severity: MED
/// LLM confusion: 3 (MED)
///
/// Description: This code converts errors to None (using `.ok()`) without logging what went
/// wrong. When something fails, there's no record of it anywhere - no log file, no error message,
/// nothing. You'll just see None and have no idea why the operation failed. It's like a function
/// that returns null on error but doesn't tell you what error occurred. Fix by logging errors
/// before converting them to Option, or return Result instead to preserve error information.
///
/// ## The Black Hole Problem
///
/// ```text
/// result.ok().and_then(|s| s.parse().ok())
///     ↓
/// Returns None
///     ↓
/// Why? File not found? Parse error? Permission denied?
///     ↓
/// No logs, no traces, no way to know
/// ```
///
/// ## Why This Matters
///
/// 1. **Invisible failures**: Operations fail but leave no trace
/// 2. **Debugging nightmare**: "It just returns None" with no explanation
/// 3. **Support burden**: Users can't provide useful error reports
/// 4. **Silent data loss**: Write failures go unnoticed
///
/// ## The Right Solutions
///
/// ### Option 1: Log before converting with inspect_err
/// ```rust,no_run
/// std::fs::read_to_string("file.txt")
///     .inspect_err(|e| eprintln!("Read failed: {}", e))
///     .ok()
/// ```
///
/// ### Option 2: Return Result to preserve error info
/// ```rust,no_run
/// fn read_config() -> Result<String, std::io::Error> {
///     std::fs::read_to_string("config.txt")
/// }
/// ```
///
/// ### Option 3: Use map_err + ok for controlled conversion
/// ```rust,no_run
/// std::fs::read_to_string("file.txt")
///     .map_err(|e| {
///         log::warn!("File read failed: {}", e);
///         e
///     })
///     .ok()
/// ```
///
/// Mitigation: Use a logging framework and log errors before converting to Option. Consider
/// whether Option is the right return type - Result preserves error information. Use
/// `inspect_err()` to log errors while still converting to Option if needed.

// ============================================================================
// DANGEROUS PATTERNS - NEVER DO THIS
// ============================================================================

/// PROBLEM E1306: Error converted to None without logging
pub fn e1306_bad_swallow_errors() -> Option<i32> {
    let result = std::fs::read_to_string("file.txt");
    // PROBLEM E1306: Error converted to None without logging
    result.ok().and_then(|s| s.parse().ok())
}

/// PROBLEM E1306: Multiple errors swallowed in chain
pub fn e1306_bad_chain_swallow() -> Option<String> {
    std::fs::read_to_string("config.txt")
        .ok()? // Swallowed!
        .lines()
        .next()
        .map(|s| s.to_string())
}

/// PROBLEM E1306: ok() in map hides errors
pub fn e1306_bad_ok_in_map(paths: &[&str]) -> Vec<String> {
    paths
        .iter()
        .filter_map(|p| std::fs::read_to_string(p).ok())
        .collect()
}

/// Entry point for problem demonstration
pub fn e1306_entry() -> Result<(), Box<dyn std::error::Error>> {
    let _ = e1306_bad_swallow_errors();
    Ok(())
}

// ============================================================================
// GOOD ALTERNATIVES
// ============================================================================

/// GOOD: Log before converting to Option
pub fn e1306_good_log_before_ok() -> Option<i32> {
    let content = std::fs::read_to_string("file.txt")
        .inspect_err(|e| eprintln!("Failed to read file: {}", e))
        .ok()?;

    content
        .trim()
        .parse()
        .inspect_err(|e| eprintln!("Failed to parse content: {}", e))
        .ok()
}

/// GOOD: Return Result to preserve error information
pub fn e1306_good_return_result() -> Result<i32, Box<dyn std::error::Error>> {
    let content = std::fs::read_to_string("file.txt")?;
    let num = content.trim().parse()?;
    Ok(num)
}

/// GOOD: Custom error type preserves context
#[derive(Debug)]
pub enum ReadParseError {
    ReadFailed(std::io::Error),
    ParseFailed(std::num::ParseIntError),
}

impl std::fmt::Display for ReadParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ReadParseError::ReadFailed(e) => write!(f, "Failed to read: {}", e),
            ReadParseError::ParseFailed(e) => write!(f, "Failed to parse: {}", e),
        }
    }
}

impl std::error::Error for ReadParseError {}

pub fn e1306_good_typed_error() -> Result<i32, ReadParseError> {
    let content = std::fs::read_to_string("file.txt").map_err(ReadParseError::ReadFailed)?;
    let num = content
        .trim()
        .parse()
        .map_err(ReadParseError::ParseFailed)?;
    Ok(num)
}

/// GOOD: Log errors in filter_map
pub fn e1306_good_logged_filter_map(paths: &[&str]) -> Vec<String> {
    paths
        .iter()
        .filter_map(|p| {
            std::fs::read_to_string(p)
                .inspect_err(|e| eprintln!("Skipping {}: {}", p, e))
                .ok()
        })
        .collect()
}

/// GOOD: Collect both successes and failures
pub fn e1306_good_partition_results<'a>(
    paths: &'a [&'a str],
) -> (Vec<String>, Vec<(&'a str, std::io::Error)>) {
    let mut successes = Vec::new();
    let mut failures = Vec::new();

    for path in paths {
        match std::fs::read_to_string(path) {
            Ok(content) => successes.push(content),
            Err(e) => failures.push((*path, e)),
        }
    }

    (successes, failures)
}

// ============================================================================
// GOOD EXAMPLES unit tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_good_log_before_ok_returns_none_for_missing() {
        let result = e1306_good_log_before_ok();
        assert!(result.is_none());
    }

    #[test]
    fn test_good_return_result_gives_error() {
        let result = e1306_good_return_result();
        assert!(result.is_err());
    }

    #[test]
    fn test_good_typed_error_preserves_context() {
        let result = e1306_good_typed_error();
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(matches!(err, ReadParseError::ReadFailed(_)));
    }

    #[test]
    fn test_good_partition_results() {
        let paths = ["nonexistent1.txt", "nonexistent2.txt"];
        let (successes, failures) = e1306_good_partition_results(&paths);
        assert!(successes.is_empty());
        assert_eq!(failures.len(), 2);
    }
}
