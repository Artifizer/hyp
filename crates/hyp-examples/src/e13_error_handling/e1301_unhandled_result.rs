/// E1301: Unhandled Result values
/// Severity: HIGH
/// LLM confusion: 3 (MED)
///
/// Description: This code calls a function that can fail (returns a Result type) but doesn't
/// check whether it succeeded or failed. This means errors are silently ignored, which can lead
/// to bugs that are hard to track down. Fix by handling the Result with pattern matching, the
/// `?` operator, or explicitly handling both success and error cases.
///
/// ## The Silent Failure Problem
///
/// ```text
/// Code: let _file = std::fs::read_to_string("config.txt");
///          ↓
/// File doesn't exist? No error shown, program continues
///          ↓
/// Later code assumes file was read successfully
///          ↓
/// Mysterious failures with no indication of the root cause
/// ```
///
/// ## Why This Matters
///
/// 1. **Silent failures**: Errors happen but you never know about them
/// 2. **Hard to debug**: No error messages, no stack traces, no clues
/// 3. **Incorrect behavior**: Program continues with invalid/missing data
/// 4. **Data corruption**: Writes may fail but program thinks they succeeded
///
/// ## The Right Solutions
///
/// ### Option 1: Use the ? operator (propagate errors)
/// ```rust
/// fn read_config() -> Result<String, std::io::Error> {
///     let content = std::fs::read_to_string("config.txt")?;
///     Ok(content)
/// }
/// ```
///
/// ### Option 2: Handle with match
/// ```rust,no_run
/// match std::fs::read_to_string("config.txt") {
///     Ok(content) => println!("Config: {}", content),
///     Err(e) => eprintln!("Failed to read config: {}", e),
/// }
/// ```
///
/// ### Option 3: Use unwrap_or_else for defaults
/// ```rust,no_run
/// let content = std::fs::read_to_string("config.txt")
///     .unwrap_or_else(|e| {
///         eprintln!("Using default config: {}", e);
///         String::from("default=true")
///     });
/// ```
///
/// Mitigation: Use `#![deny(unused_must_use)]` to make ignoring Results a compilation error.
/// Always handle Results with `match`, `if let`, `?`, or `.unwrap()` with a comment explaining
/// why panicking is acceptable.

// ============================================================================
// DANGEROUS PATTERNS - NEVER DO THIS
// ============================================================================

/// PROBLEM E1301: Result completely ignored
pub fn e1301_bad_unhandled_result() {
    let _file = std::fs::read_to_string("config.txt"); // PROBLEM: Result not handled
}

/// PROBLEM E1301: Result assigned but never checked
pub fn e1301_bad_assigned_but_unchecked() {
    let result = std::fs::write("log.txt", "data");
    // result is never examined - did the write succeed?
    println!("Write 'completed'"); // Lies! We don't know if it completed.
}

/// PROBLEM E1301: Result in a chain, error swallowed
pub fn e1301_bad_chained_ignored() {
    let _ = std::fs::create_dir("temp")
        .and_then(|_| std::fs::write("temp/file.txt", "data"));
    // Both operations could fail silently
}

/// Entry point for problem demonstration
pub fn e1301_entry() -> Result<(), Box<dyn std::error::Error>> {
    e1301_bad_unhandled_result();
    Ok(())
}

// ============================================================================
// GOOD EXAMPLES
// ============================================================================

/// GOOD: Propagate error with ? operator
pub fn e1301_good_propagate() -> Result<String, std::io::Error> {
    let content = std::fs::read_to_string("config.txt")?;
    Ok(content)
}

/// GOOD: Handle both cases with match
pub fn e1301_good_match_handling() {
    match std::fs::read_to_string("config.txt") {
        Ok(content) => println!("Config loaded: {} bytes", content.len()),
        Err(e) => eprintln!("Failed to load config: {}", e),
    }
}

/// GOOD: Use if let when you only care about success
pub fn e1301_good_if_let() -> Option<String> {
    std::fs::read_to_string("config.txt").ok()
}

/// GOOD: Provide default on error with logging
pub fn e1301_good_with_default() -> String {
    std::fs::read_to_string("config.txt").unwrap_or_else(|e| {
        eprintln!("Warning: Could not read config ({}), using defaults", e);
        String::from("default_value=true")
    })
}

/// GOOD: Convert to Option when error details don't matter
pub fn e1301_good_to_option() -> Option<String> {
    std::fs::read_to_string("config.txt").ok()
}

// ============================================================================
// GOOD EXAMPLES unit tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_good_propagate_returns_error_for_missing_file() {
        let result = e1301_good_propagate();
        assert!(result.is_err());
    }

    #[test]
    fn test_good_with_default_returns_default() {
        let content = e1301_good_with_default();
        assert!(content.contains("default_value"));
    }

    #[test]
    fn test_good_to_option_returns_none_for_missing() {
        let result = e1301_good_to_option();
        assert!(result.is_none());
    }
}
