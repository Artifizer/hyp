/// E1303: Ignoring errors with let _ =
/// Severity: HIGH
/// LLM confusion: 3 (MED)
///
/// Description: Using `let _ =` explicitly throws away the result of an operation, including any
/// errors that occurred. This is like catching an exception and doing nothing with it - critical
/// operations might fail silently, making bugs nearly impossible to track down. Imagine deleting
/// a file and not checking if it actually got deleted. Fix by properly handling the error with
/// if/match statements, or at minimum logging what went wrong before ignoring it.
///
/// ## The Silent Failure Problem
///
/// ```text
/// Code: let _ = std::fs::remove_file("important.txt");
///          ↓
/// File is locked by another process? Error silently ignored
///          ↓
/// File still exists but program thinks it's deleted
///          ↓
/// Later operations fail mysteriously or corrupt data
/// ```
///
/// ## Why This Matters
///
/// 1. **Silent failures**: Critical operations fail without any indication
/// 2. **Data corruption**: Writes may fail but program continues as if they succeeded
/// 3. **Security issues**: Permission errors ignored could mean unauthorized access
/// 4. **Debugging nightmare**: No logs, no traces, no way to know what went wrong
///
/// ## The Right Solutions
///
/// ### Option 1: Handle the error properly
/// ```rust,no_run
/// if let Err(e) = std::fs::remove_file("temp.txt") {
///     eprintln!("Warning: Could not remove temp file: {}", e);
/// }
/// ```
///
/// ### Option 2: Log before ignoring (when truly optional)
/// ```rust,no_run
/// let _ = std::fs::remove_file("temp.txt")
///     .inspect_err(|e| eprintln!("Note: temp cleanup failed: {}", e));
/// ```
///
/// ### Option 3: Propagate with ?
/// ```rust,no_run
/// fn cleanup() -> std::io::Result<()> {
///     std::fs::remove_file("temp.txt")?;
///     Ok(())
/// }
/// ```
///
/// Mitigation: Use `#![warn(clippy::let_underscore_must_use)]` to catch this pattern. If you
/// truly need to ignore an error, use `.ok()` or add a comment explaining why. Better yet,
/// log the error before ignoring it.

// ============================================================================
// DANGEROUS PATTERNS - NEVER DO THIS
// ============================================================================

/// PROBLEM E1303: Error silently ignored with let _ =
pub fn e1303_bad_ignored_errors() {
    let _ = std::fs::remove_file("temp.txt"); // PROBLEM: Error silently ignored
}

/// PROBLEM E1303: Multiple ignored errors in sequence
pub fn e1303_bad_multiple_ignored() {
    let _ = std::fs::create_dir("output");
    let _ = std::fs::write("output/data.txt", "content");
    let _ = std::fs::remove_file("output/old.txt");
    // Any of these could fail and we'd never know!
}

/// PROBLEM E1303: Ignoring network/IO errors
pub fn e1303_bad_network_ignored() {
    use std::io::Write;
    let mut file = Vec::new();
    let _ = file.write_all(b"data"); // What if the write fails?
    let _ = file.flush(); // What if flush fails?
}

/// Entry point for problem demonstration
pub fn e1303_entry() -> Result<(), Box<dyn std::error::Error>> {
    e1303_bad_ignored_errors();
    Ok(())
}

// ============================================================================
// GOOD ALTERNATIVES
// ============================================================================

/// GOOD: Handle error with if let
pub fn e1303_good_handle_error() {
    if let Err(e) = std::fs::remove_file("temp.txt") {
        eprintln!("Warning: Could not remove temp file: {}", e);
    }
}

/// GOOD: Use match for different error handling
pub fn e1303_good_match_error() {
    match std::fs::remove_file("temp.txt") {
        Ok(()) => println!("Temp file cleaned up"),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            // File already gone, that's fine
        }
        Err(e) => {
            eprintln!("Error removing temp file: {}", e);
        }
    }
}

/// GOOD: Log before ignoring with inspect_err
pub fn e1303_good_log_before_ignore() {
    // When the operation is truly optional, at least log it
    let _ = std::fs::remove_file("temp.txt")
        .inspect_err(|e| eprintln!("Note: temp cleanup failed (non-critical): {}", e));
}

/// GOOD: Propagate error to caller
pub fn e1303_good_propagate() -> std::io::Result<()> {
    std::fs::remove_file("temp.txt")?;
    Ok(())
}

/// GOOD: Collect errors for batch operations
pub fn e1303_good_collect_errors(files: &[&str]) -> Vec<std::io::Error> {
    files
        .iter()
        .filter_map(|f| std::fs::remove_file(f).err())
        .collect()
}

/// GOOD: Use ok() with explicit comment when truly optional
pub fn e1303_good_explicit_optional() {
    // Cleanup is best-effort; failure is acceptable
    // because the temp directory will be cleared on reboot anyway
    std::fs::remove_file("temp.txt").ok();
}

// ============================================================================
// GOOD EXAMPLES unit tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_good_handle_error_doesnt_panic() {
        // Should not panic even if file doesn't exist
        e1303_good_handle_error();
    }

    #[test]
    fn test_good_match_error_handles_not_found() {
        // Should handle NotFound gracefully
        e1303_good_match_error();
    }

    #[test]
    fn test_good_collect_errors() {
        let errors = e1303_good_collect_errors(&["nonexistent1.txt", "nonexistent2.txt"]);
        assert_eq!(errors.len(), 2);
    }

    #[test]
    fn test_good_propagate_returns_error() {
        let result = e1303_good_propagate();
        assert!(result.is_err());
    }
}
