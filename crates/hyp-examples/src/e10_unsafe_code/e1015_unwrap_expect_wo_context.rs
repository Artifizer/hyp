/// E1015: Unwrap/expect without context
/// Severity: HIGH
/// LLM confusion: 3 (MED)
///
/// Description: Using unwrap() or expect() crashes the program if the value is None or Err, but
/// provides no context about what went wrong or where. It's like catching an exception and just
/// printing "error" with no details - you know something failed but not what or why. This makes
/// debugging very difficult. Fix by using proper error handling with Result/Option, or at minimum
/// use expect() with a descriptive message explaining what was expected.
///
/// Mitigation: Use `#![warn(clippy::unwrap_used)]` to catch unwrap calls. Prefer pattern matching,
/// `if let`, or the `?` operator for error handling. If unwrap is necessary, add a comment
/// explaining why it's safe. Use `expect("descriptive message")` instead of bare `unwrap()`.

pub fn e1015_bad_unwrap_expect_wo_context() {
    let data = Some(42);

    // PROBLEM E1002: direct unwrap/expect
    // PROBLEM E1015: unwrap without context
    let _value = data.unwrap();

    // PROBLEM E1002: direct unwrap/expect
    // PROBLEM E1015: expect with poor message
    let _value2 = data.expect("failed");
}

pub fn e1015_entry() -> Result<(), Box<dyn std::error::Error>> {
    e1015_bad_unwrap_expect_wo_context();
    Ok(())
}

// ============================================================================
// GOOD EXAMPLES - Proper alternatives
// ============================================================================

use crate::test_constants::MAGIC_I32;

/// GOOD: Use pattern matching for clear control flow
pub fn e1015_good_pattern_matching(data: Option<i32>) -> i32 {
    match data {
        Some(value) => value,
        None => {
            eprintln!("Warning: data was None, using default");
            0
        }
    }
}

/// GOOD: Use if let for simple cases
pub fn e1015_good_if_let(data: Option<i32>) {
    if let Some(value) = data {
        println!("Got value: {}", value);
    } else {
        println!("No value available");
    }
}

/// GOOD: Return Result to caller
pub fn e1015_good_return_result(data: Option<i32>) -> Result<i32, &'static str> {
    data.ok_or("required data was not provided")
}

/// GOOD: Use expect with descriptive message
pub fn e1015_expect_with_context(config_path: &str) -> String {
    std::fs::read_to_string(config_path)
        .unwrap_or_else(|_| panic!("Failed to read config file '{}' - file must exist and be readable",
            config_path))
}

/// GOOD: Use unwrap_or with default
pub fn e1015_good_unwrap_or_default(data: Option<i32>) -> i32 {
    data.unwrap_or(0) // Clear default value
}

/// GOOD: Use unwrap_or_else for computed defaults
pub fn e1015_good_unwrap_or_else(data: Option<i32>) -> i32 {
    data.unwrap_or_else(|| {
        eprintln!("Warning: using fallback value");
        e1015_good_compute_fallback()
    })
}

fn e1015_good_compute_fallback() -> i32 {
    MAGIC_I32
}

/// GOOD: Chain with map and unwrap_or
pub fn e1015_good_map_chain(data: Option<String>) -> usize {
    data.map(|s| s.len()).unwrap_or(0)
}

// ============================================================================
// GOOD EXAMPLES unit tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn e1015_good_pattern_matching_handles_none() {
        let value = e1015_good_pattern_matching(None);
        assert_eq!(value, 0);
    }

    #[test]
    fn e1015_good_return_result_errors_on_none() {
        let result = e1015_good_return_result(None);
        assert!(result.is_err());
    }

    #[test]
    fn e1015_good_unwrap_or_default_returns_value() {
        assert_eq!(e1015_good_unwrap_or_default(Some(5)), 5);
        assert_eq!(e1015_good_unwrap_or_default(None), 0);
    }

    #[test]
    fn e1015_good_map_chain_counts_length() {
        let size = e1015_good_map_chain(Some("abc".to_string()));
        assert_eq!(size, 3);
    }
}
