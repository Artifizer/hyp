/// E1902: Inline directive control
/// Severity: HIGH
/// LLM confusion: 4 (MEDIUM-HIGH)
///
/// Description: Prevents bypassing project rules with inline directives like
/// #[allow(clippy::...)] in unauthorized locations. Critical for maintaining
/// code standards in AI-generated code where LLMs might add suppression attributes.
///
/// ## Why This Matters
///
/// 1. **Prevents rule bypassing**: Stops developers/LLMs from silencing important warnings
/// 2. **Enforces standards**: Ensures project-level configuration is respected
/// 3. **Code review**: Makes it easier to spot when rules are being circumvented
/// 4. **AI safety**: LLMs often add #[allow(...)] to "fix" warnings instead of addressing root causes
///
/// ## The Right Solutions
///
/// ### Option 1: Fix the underlying issue
/// ```rust
/// // Instead of suppressing the warning:
/// // #[allow(clippy::unwrap_used)]
/// // fn process(data: Option<i32>) -> i32 { data.unwrap() }
///
/// // Fix the code:
/// fn process(data: Option<i32>) -> i32 {
///     data.unwrap_or(0)
/// }
/// ```
///
/// ### Option 2: Use project-level configuration
/// ```toml
/// # In Clippy.toml or .cargo/config.toml
/// [lints.clippy]
/// unwrap_used = "allow"
/// ```
///
/// ### Option 3: Restrict to test code only
/// ```rust
/// #[cfg(test)]
/// mod tests {
///     #[allow(dead_code)]  // OK in tests
///     fn helper() {}
/// }
/// ```
///
/// Mitigation: Configure E1902 in Hyp.toml to specify which directives are
/// allowed and in which file paths. Use empty allowed_paths to block everywhere.

// ============================================================================
// PROBLEMATIC PATTERNS
// ============================================================================
// Note: Functions use snake_case following Rust naming conventions:
// e1902_bad_clippy_bypass, e1902_good_no_unwrap, etc.

/// PROBLEM E1902: Clippy allow directive in production code
#[allow(clippy::unwrap_used)]
pub fn e1902_bad_clippy_bypass() -> i32 {
    Some(42).unwrap()
}

/// PROBLEM E1902: Warning suppression outside tests
#[allow(dead_code)]
pub fn e1902_bad_dead_code_allow() {
    // This helper is actually dead code but we're hiding it
}

/// PROBLEM E1902: Multiple suppressions stacked
#[allow(clippy::unwrap_used)]
#[allow(clippy::expect_used)]
pub fn e1902_bad_multiple_suppressions() -> String {
    Some("data".to_string()).expect("should exist")
}

/// PROBLEM E1902: Suppressing complexity warnings instead of refactoring
#[allow(clippy::cognitive_complexity)]
pub fn e1902_bad_complexity_suppress(x: i32) -> i32 {
    if x > 0 {
        if x < 10 {
            if x % 2 == 0 {
                x * 2
            } else {
                x * 3
            }
        } else {
            x + 10
        }
    } else {
        0
    }
}

/// Entry point for problem demonstration
pub fn e1902_entry() -> Result<(), Box<dyn std::error::Error>> {
    let _ = e1902_bad_clippy_bypass();
    e1902_bad_dead_code_allow();
    let _ = e1902_bad_multiple_suppressions();
    let _ = e1902_bad_complexity_suppress(5);
    Ok(())
}

// ============================================================================
// GOOD ALTERNATIVES
// ============================================================================

/// GOOD: Fix the code instead of suppressing warnings
pub fn e1902_good_no_unwrap() -> i32 {
    Some(42).unwrap_or(0)
}

/// GOOD: Actually use the function or remove it
pub fn e1902_good_used_function() -> String {
    "This function is actually used".to_string()
}

/// GOOD: Refactor complex code instead of suppressing
pub fn e1902_good_simplified(x: i32) -> i32 {
    match x {
        x if x <= 0 => 0,
        x if x >= 10 => x + 10,
        x if x % 2 == 0 => x * 2,
        x => x * 3,
    }
}

/// GOOD: Use Result for error handling
pub fn e1902_good_result_handling() -> Result<String, &'static str> {
    Some("data".to_string()).ok_or("data not found")
}

// ============================================================================
// GOOD EXAMPLES unit tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_good_no_unwrap() {
        assert_eq!(e1902_good_no_unwrap(), 42);
    }

    #[test]
    fn test_good_used_function() {
        assert!(!e1902_good_used_function().is_empty());
    }

    #[test]
    fn test_good_simplified() {
        assert_eq!(e1902_good_simplified(-1), 0);
        assert_eq!(e1902_good_simplified(4), 8);
        assert_eq!(e1902_good_simplified(5), 15);
        assert_eq!(e1902_good_simplified(15), 25);
    }

    #[test]
    fn test_good_result_handling() {
        assert!(e1902_good_result_handling().is_ok());
    }

    // Note: #[allow(...)] in test modules might be permitted by E1902 config
    #[allow(dead_code)]
    fn test_helper() {
        // Test-only helpers can use allow directives
    }
}
