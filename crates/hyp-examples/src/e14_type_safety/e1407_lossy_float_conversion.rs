/// E1407: Lossy float to int conversion
/// Severity: MED
/// LLM confusion: 3 (MED)
///
/// Description: Converting floating-point numbers to integers with 'as' truncates the decimal part
/// and can overflow if the float is too large. For example, converting a very large float to a
/// 32-bit integer produces undefined behavior. It's like rounding a decimal to a whole number but
/// also risking overflow if the number is too big. Fix by checking that floats are in valid range
/// before converting, or use explicit rounding functions.
///
/// ## The Double Problem
///
/// ```text
/// let x: f64 = 3.7;
/// let y: i32 = x as i32;  // y = 3 (truncated, not rounded)
///
/// let big: f64 = 1e20;
/// let z: i32 = big as i32;  // Undefined behavior! Overflow
/// ```
///
/// ## Why This Matters
///
/// 1. **Truncation surprise**: 3.9 becomes 3, not 4
/// 2. **Overflow UB**: Large floats cause undefined behavior
/// 3. **NaN handling**: NaN as i32 is undefined
/// 4. **Infinity issues**: Infinity as i32 is undefined
///
/// ## The Right Solutions
///
/// ### Option 1: Bounds check before conversion
/// ```rust
/// fn safe_to_i32(x: f64) -> Option<i32> {
///     if x.is_nan() || x < i32::MIN as f64 || x > i32::MAX as f64 {
///         None
///     } else {
///         Some(x as i32)
///     }
/// }
/// ```
///
/// ### Option 2: Use explicit rounding
/// ```rust
/// fn round_to_i32(x: f64) -> Option<i32> {
///     let rounded = x.round();
///     if rounded < i32::MIN as f64 || rounded > i32::MAX as f64 {
///         None
///     } else {
///         Some(rounded as i32)
///     }
/// }
/// ```
///
/// ### Option 3: Use saturating conversion
/// ```rust
/// fn saturating_to_i32(x: f64) -> i32 {
///     x.clamp(i32::MIN as f64, i32::MAX as f64) as i32
/// }
/// ```
///
/// Mitigation: Use `#![warn(clippy::cast_possible_truncation)]` and
/// `#![warn(clippy::float_to_int_without_bounds)]`. Check that floats are in valid range before
/// converting. Consider using `round()`, `floor()`, or `ceil()` to make rounding explicit.

// ============================================================================
// DANGEROUS PATTERNS - NEVER DO THIS
// ============================================================================

/// PROBLEM E1407: Can lose fractional part and overflow
pub fn e1407_bad_conversion(x: f64) -> i32 {
    // PROBLEM E1407: Can lose fractional part and overflow
    x as i32
}

/// PROBLEM E1407: No NaN check
pub fn e1407_bad_no_nan_check(x: f64) -> i32 {
    x as i32 // NaN as i32 is undefined!
}

/// PROBLEM E1407: No infinity check
pub fn e1407_bad_no_inf_check(x: f64) -> i32 {
    x as i32 // Infinity as i32 is undefined!
}

/// PROBLEM E1407: Truncation instead of rounding
pub fn e1407_bad_truncation(x: f64) -> i32 {
    x as i32 // 3.9 becomes 3, not 4
}

/// Entry point for problem demonstration
pub fn e1407_entry() -> Result<(), Box<dyn std::error::Error>> {
    Ok(())
}

// ============================================================================
// GOOD ALTERNATIVES
// ============================================================================

/// GOOD: Full bounds and special value check
pub fn e1407_good_safe_convert(x: f64) -> Option<i32> {
    if x.is_nan() || x.is_infinite() {
        return None;
    }
    if x < i32::MIN as f64 || x > i32::MAX as f64 {
        return None;
    }
    Some(x as i32)
}

/// GOOD: Round before converting
pub fn e1407_good_round(x: f64) -> Option<i32> {
    if x.is_nan() || x.is_infinite() {
        return None;
    }
    let rounded = x.round();
    if rounded < i32::MIN as f64 || rounded > i32::MAX as f64 {
        return None;
    }
    Some(rounded as i32)
}

/// GOOD: Floor (round toward negative infinity)
pub fn e1407_good_floor(x: f64) -> Option<i32> {
    if x.is_nan() || x.is_infinite() {
        return None;
    }
    let floored = x.floor();
    if floored < i32::MIN as f64 || floored > i32::MAX as f64 {
        return None;
    }
    Some(floored as i32)
}

/// GOOD: Ceiling (round toward positive infinity)
pub fn e1407_good_ceil(x: f64) -> Option<i32> {
    if x.is_nan() || x.is_infinite() {
        return None;
    }
    let ceiled = x.ceil();
    if ceiled < i32::MIN as f64 || ceiled > i32::MAX as f64 {
        return None;
    }
    Some(ceiled as i32)
}

/// GOOD: Saturating conversion (clamp to range)
pub fn e1407_good_saturating(x: f64) -> i32 {
    if x.is_nan() {
        return 0; // Or some other default
    }
    if x <= i32::MIN as f64 {
        return i32::MIN;
    }
    if x >= i32::MAX as f64 {
        return i32::MAX;
    }
    x as i32
}

/// GOOD: Truncate toward zero (explicit)
pub fn e1407_good_trunc(x: f64) -> Option<i32> {
    if x.is_nan() || x.is_infinite() {
        return None;
    }
    let truncated = x.trunc();
    if truncated < i32::MIN as f64 || truncated > i32::MAX as f64 {
        return None;
    }
    Some(truncated as i32)
}

/// GOOD: Return Result with error info
pub fn e1407_good_result(x: f64) -> Result<i32, &'static str> {
    if x.is_nan() {
        return Err("Cannot convert NaN to integer");
    }
    if x.is_infinite() {
        return Err("Cannot convert infinity to integer");
    }
    if x < i32::MIN as f64 {
        return Err("Value too small for i32");
    }
    if x > i32::MAX as f64 {
        return Err("Value too large for i32");
    }
    Ok(x.round() as i32)
}

// ============================================================================
// GOOD EXAMPLES unit tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_safe_convert_normal() {
        assert_eq!(e1407_good_safe_convert(42.7), Some(42));
    }

    #[test]
    fn test_safe_convert_nan() {
        assert_eq!(e1407_good_safe_convert(f64::NAN), None);
    }

    #[test]
    fn test_safe_convert_infinity() {
        assert_eq!(e1407_good_safe_convert(f64::INFINITY), None);
        assert_eq!(e1407_good_safe_convert(f64::NEG_INFINITY), None);
    }

    #[test]
    fn test_safe_convert_overflow() {
        assert_eq!(e1407_good_safe_convert(1e20), None);
    }

    #[test]
    fn test_round() {
        assert_eq!(e1407_good_round(3.4), Some(3));
        assert_eq!(e1407_good_round(3.5), Some(4));
        assert_eq!(e1407_good_round(-3.5), Some(-4));
    }

    #[test]
    fn test_floor() {
        assert_eq!(e1407_good_floor(3.9), Some(3));
        assert_eq!(e1407_good_floor(-3.1), Some(-4));
    }

    #[test]
    fn test_ceil() {
        assert_eq!(e1407_good_ceil(3.1), Some(4));
        assert_eq!(e1407_good_ceil(-3.9), Some(-3));
    }

    #[test]
    fn test_saturating() {
        assert_eq!(e1407_good_saturating(1e20), i32::MAX);
        assert_eq!(e1407_good_saturating(-1e20), i32::MIN);
        assert_eq!(e1407_good_saturating(f64::NAN), 0);
    }

    #[test]
    fn test_result_errors() {
        assert!(e1407_good_result(f64::NAN).is_err());
        assert!(e1407_good_result(1e20).is_err());
        assert_eq!(e1407_good_result(42.5), Ok(43)); // Rounds
    }
}
