/// E1001: Direct call of panic() in production code
/// Severity: HIGH
/// LLM confusion: 2 (LOW)
///
/// Description: Calling panic!() directly in production code immediately crashes the entire program
/// when the condition is met. This is like throwing an unhandled exception that terminates the
/// application - there's no way for callers to recover or handle the error gracefully. Instead of
/// crashing, return a Result type so callers can decide how to handle the error (retry, log, use
/// a default value, etc.).
///
/// Mitigation: Use `#![warn(clippy::panic)]` to catch direct panic calls. Return `Result<T, E>`
/// instead of panicking. Reserve panic!() only for truly unrecoverable errors or bugs that should
/// never happen. Use `expect()` with a clear message for cases that should be impossible.

pub fn e1001_bad_direct_panic(value: i32) {
    if value > 40 {
        // PROBLEM E1001: Direct panic in production code
        panic!("Value too large!");
    }
}

pub fn e1001_entry() -> Result<(), Box<dyn std::error::Error>> {
    e1001_bad_direct_panic(0);
    Ok(())
}

// ============================================================================
// GOOD EXAMPLES - Proper alternatives
// ============================================================================

use crate::test_constants::MAGIC_I32;

/// Simple error type for validation
#[derive(Debug, Clone, PartialEq)]
pub enum ValidationError {
    ValueTooLarge { value: i32, max: i32 },
}

/// GOOD: Return Result with proper error type instead of panicking
fn e1001_good_return_result(value: i32) -> Result<(), ValidationError> {
    const MAX_VALUE: i32 = 40;
    if value > MAX_VALUE {
        return Err(ValidationError::ValueTooLarge { value, max: MAX_VALUE });
    }
    Ok(())
}

/// GOOD: Use Option for optional success
fn e1001_good_return_option(value: i32) -> Option<i32> {
    if value > MAGIC_I32 {
        return None;
    }
    Some(value * MAGIC_I32)
}

/// GOOD: Provide a validated constructor
struct ValidatedValue(i32);

impl ValidatedValue {
    fn new(value: i32) -> Result<Self, String> {
        if value > 40 {
            return Err(format!("Value {} exceeds maximum of 40", value));
        }
        Ok(Self(value))
    }
}

// ============================================================================
// GOOD EXAMPLES unit tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn e1001_good_result_rejects_large_value() {
        let result = e1001_good_return_result(41);
        assert!(result.is_err());
    }

    #[test]
    fn e1001_good_option_handles_large() {
        let result = e1001_good_return_option(100);
        assert!(result.is_none());
    }

    #[test]
    fn e1001_good_validated_constructor_accepts_small() {
        let value = ValidatedValue::new(10);
        assert!(value.is_ok());
    }
}
