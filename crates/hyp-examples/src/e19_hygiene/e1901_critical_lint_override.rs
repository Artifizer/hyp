/// E1901: Critical Lint Override Detection
/// Severity: HIGH
/// LLM confusion: 4 (MEDIUM-HIGH)
///
/// Description: Detects when code uses `#[allow(...)]` attributes to override
/// critical Clippy lints like unsafe_code, panic, unwrap_used, etc.
/// LLMs often add these attributes to "fix" warnings instead of addressing root causes.
///
/// ## Why This Matters
///
/// 1. **Bypasses safety checks**: Critical lints protect against unsafe code and panics
/// 2. **LLM anti-pattern**: AI tends to suppress warnings rather than fix issues
/// 3. **Code review blind spot**: Overrides can hide dangerous code
/// 4. **Undermines static analysis**: Defeats the purpose of lint enforcement
///
/// ## The Right Solutions
///
/// ### Option 1: Fix the underlying issue
/// ```rust
/// // Instead of suppressing unwrap_used:
/// // #[allow(clippy::unwrap_used)]
/// // fn get_value() -> i32 { Some(42).unwrap() }
///
/// // Fix it:
/// fn get_value() -> Option<i32> { Some(42) }
/// ```
///
/// ### Option 2: Use proper error handling
/// ```rust
/// fn get_value() -> Result<i32, &'static str> {
///     Some(42).ok_or("value not found")
/// }
/// ```
///
/// Mitigation: Configure E1901 in Hyp.toml to allow overrides in specific paths
/// (e.g., test files or FFI bindings).

// ============================================================================
// PROBLEMATIC PATTERNS
// ============================================================================

/// PROBLEM E1901: Suppressing unsafe_code warning
#[allow(unsafe_code)]
pub fn e1901_bad_unsafe_override() {
    // This hides potentially dangerous unsafe code
    unsafe {
        let ptr: *const i32 = std::ptr::null();
        let _ = *ptr; // Would crash!
    }
}

/// PROBLEM E1901: Suppressing panic warning
#[allow(clippy::panic)]
pub fn e1901_bad_panic_override() {
    panic!("This panic is hidden from linters");
}

/// PROBLEM E1901: Suppressing unwrap_used warning
#[allow(clippy::unwrap_used)]
pub fn e1901_bad_unwrap_override() -> i32 {
    let maybe: Option<i32> = None;
    maybe.unwrap() // Would panic at runtime!
}

/// PROBLEM E1901: Suppressing expect_used warning
#[allow(clippy::expect_used)]
pub fn e1901_bad_expect_override() -> String {
    let result: Result<String, &str> = Err("failed");
    result.expect("this will panic")
}

/// PROBLEM E1901: Multiple critical overrides stacked
#[allow(clippy::unwrap_used)]
#[allow(clippy::panic)]
pub fn e1901_bad_multiple_overrides() {
    if false {
        panic!("hidden panic");
    }
    Some(1).unwrap();
}

/// Entry point for problem demonstration
pub fn e1901_entry() -> Result<(), Box<dyn std::error::Error>> {
    // Note: These functions would panic if actually called
    // e1901_bad_unsafe_override();
    // e1901_bad_panic_override();
    // e1901_bad_unwrap_override();
    Ok(())
}

// ============================================================================
// GOOD ALTERNATIVES
// ============================================================================

/// GOOD: Proper error handling instead of unwrap
pub fn e1901_good_result_handling() -> Result<i32, &'static str> {
    let maybe: Option<i32> = Some(42);
    maybe.ok_or("value not found")
}

/// GOOD: Using safe defaults
pub fn e1901_good_with_default() -> i32 {
    let maybe: Option<i32> = None;
    maybe.unwrap_or(0)
}

/// GOOD: Graceful error propagation
pub fn e1901_good_question_mark() -> Result<i32, &'static str> {
    let maybe: Option<i32> = Some(42);
    Ok(maybe.ok_or("missing")?)
}

/// GOOD: Match for explicit control flow
pub fn e1901_good_match_handling(input: Option<i32>) -> i32 {
    match input {
        Some(val) => val,
        None => {
            eprintln!("Warning: using default value");
            0
        }
    }
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_good_result_handling() {
        assert!(e1901_good_result_handling().is_ok());
    }

    #[test]
    fn test_good_with_default() {
        assert_eq!(e1901_good_with_default(), 0);
    }

    #[test]
    fn test_good_question_mark() {
        assert!(e1901_good_question_mark().is_ok());
    }

    #[test]
    fn test_good_match_handling() {
        assert_eq!(e1901_good_match_handling(Some(5)), 5);
        assert_eq!(e1901_good_match_handling(None), 0);
    }
}
