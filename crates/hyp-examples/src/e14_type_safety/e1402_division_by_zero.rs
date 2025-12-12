/// E1402: Division by zero
/// Severity: HIGH
/// LLM confusion: 2 (LOW)
///
/// Description: Dividing by zero crashes the program immediately. This code doesn't check if the
/// divisor is zero before dividing, which will cause a runtime crash with certain inputs. It's
/// like a calculator that crashes when you press divide by zero. Fix by checking for zero before
/// dividing, using checked_div() which returns None instead of crashing, or ensuring the divisor
/// is never zero through program logic.
///
/// ## The Crash Problem
///
/// ```text
/// fn average(sum: i32, count: i32) -> i32 {
///     sum / count  // If count is 0 → PANIC!
/// }
///
/// average(100, 0);  // Program crashes immediately
/// ```
///
/// ## Why This Matters
///
/// 1. **Immediate crash**: No chance to recover or show error message
/// 2. **User data loss**: Unsaved work is lost when program crashes
/// 3. **Security risk**: Denial of service via crafted inputs
/// 4. **Silent in tests**: May not be caught if tests don't cover zero case
///
/// ## The Right Solutions
///
/// ### Option 1: Use checked_div
/// ```rust
/// fn safe_divide(x: i32, y: i32) -> Option<i32> {
///     x.checked_div(y)  // Returns None for division by zero
/// }
/// ```
///
/// ### Option 2: Validate before dividing
/// ```rust
/// fn divide(x: i32, y: i32) -> Result<i32, &'static str> {
///     if y == 0 {
///         Err("Division by zero")
///     } else {
///         Ok(x / y)
///     }
/// }
/// ```
///
/// ### Option 3: Use NonZero types
/// ```rust
/// use std::num::NonZeroI32;
///
/// fn divide(x: i32, y: NonZeroI32) -> i32 {
///     x / y.get()  // Cannot be zero by construction
/// }
/// ```
///
/// Mitigation: Use `checked_div()` which returns `None` for division by zero instead of panicking.
/// Add assertions or validation to ensure divisors are non-zero. Use the `NonZeroI32` type when
/// you need to guarantee a value is never zero.

// ============================================================================
// DANGEROUS PATTERNS - NEVER DO THIS
// ============================================================================

/// PROBLEM E1402: No check for zero divisor
pub fn e1402_bad_division(x: i32, y: i32) -> i32 {
    // PROBLEM E1402: No check for zero divisor
    x / y
}

/// PROBLEM E1402: Average without count check
pub fn e1402_bad_average(values: &[i32]) -> i32 {
    let sum: i32 = values.iter().sum();
    sum / values.len() as i32 // Crashes if values is empty!
}

/// PROBLEM E1402: Percentage calculation
pub fn e1402_bad_percentage(part: i32, total: i32) -> i32 {
    (part * 100) / total // Crashes if total is 0
}

/// Entry point for problem demonstration
pub fn e1402_entry() -> Result<(), Box<dyn std::error::Error>> {
    Ok(())
}

// ============================================================================
// GOOD ALTERNATIVES
// ============================================================================

/// GOOD: Use checked_div
pub fn e1402_good_checked_div(x: i32, y: i32) -> Option<i32> {
    x.checked_div(y)
}

/// GOOD: Validate before dividing
pub fn e1402_good_validate(x: i32, y: i32) -> Result<i32, &'static str> {
    if y == 0 {
        Err("Division by zero")
    } else {
        Ok(x / y)
    }
}

/// GOOD: Use NonZero type
pub fn e1402_good_nonzero(x: i32, y: std::num::NonZeroI32) -> i32 {
    x / y.get() // Cannot be zero by construction
}

/// GOOD: Safe average with Option
pub fn e1402_good_average(values: &[i32]) -> Option<i32> {
    if values.is_empty() {
        None
    } else {
        Some(values.iter().sum::<i32>() / values.len() as i32)
    }
}

/// GOOD: Safe percentage with Result
pub fn e1402_good_percentage(part: i32, total: i32) -> Result<i32, &'static str> {
    if total == 0 {
        Err("Cannot calculate percentage of zero total")
    } else {
        Ok((part * 100) / total)
    }
}

/// GOOD: Return default on division by zero
pub fn e1402_good_with_default(x: i32, y: i32, default: i32) -> i32 {
    x.checked_div(y).unwrap_or(default)
}

/// GOOD: Float division (returns infinity, not crash)
pub fn e1402_good_float_division(x: f64, y: f64) -> f64 {
    x / y // Returns inf or NaN, doesn't crash
}

// ============================================================================
// GOOD EXAMPLES unit tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use std::num::NonZeroI32;

    #[test]
    fn test_checked_div_success() {
        assert_eq!(e1402_good_checked_div(10, 2), Some(5));
    }

    #[test]
    fn test_checked_div_zero() {
        assert_eq!(e1402_good_checked_div(10, 0), None);
    }

    #[test]
    fn test_validate_success() {
        assert_eq!(e1402_good_validate(10, 2), Ok(5));
    }

    #[test]
    fn test_validate_zero() {
        assert_eq!(e1402_good_validate(10, 0), Err("Division by zero"));
    }

    #[test]
    fn test_nonzero() {
        let y = NonZeroI32::new(2).unwrap();
        assert_eq!(e1402_good_nonzero(10, y), 5);
    }

    #[test]
    fn test_average_empty() {
        assert_eq!(e1402_good_average(&[]), None);
    }

    #[test]
    fn test_average_values() {
        assert_eq!(e1402_good_average(&[10, 20, 30]), Some(20));
    }

    #[test]
    fn test_with_default() {
        assert_eq!(e1402_good_with_default(10, 0, -1), -1);
        assert_eq!(e1402_good_with_default(10, 2, -1), 5);
    }

    #[test]
    fn test_float_division_infinity() {
        let result = e1402_good_float_division(1.0, 0.0);
        assert!(result.is_infinite());
    }
}
