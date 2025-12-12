/// E1404: Narrowing conversions (as)
/// Severity: MED
/// LLM confusion: 3 (MED)
///
/// Description: Converting from a larger number type to a smaller one can lose data. For example,
/// converting a 64-bit number to 32-bit truncates (cuts off) values that don't fit, silently losing
/// the upper bits. It's like trying to fit a 10-digit number into a 5-digit display - the extra
/// digits just disappear. Fix by using try_into() which returns an error if data would be lost,
/// or validate the range before converting.
///
/// ## The Truncation Problem
///
/// ```text
/// let big: i64 = 5_000_000_000;  // 5 billion
/// let small: i32 = big as i32;   // Truncated to 705,032,704!
///
/// // The upper bits are silently discarded
/// ```
///
/// ## Why This Matters
///
/// 1. **Silent data loss**: No error, no warning, just wrong values
/// 2. **Financial errors**: Large monetary amounts truncated
/// 3. **Security bugs**: Size checks bypassed by truncation
/// 4. **Hard to debug**: Values are "just wrong" with no obvious cause
///
/// ## The Right Solutions
///
/// ### Option 1: Use try_into() for checked conversion
/// ```rust
/// fn safe_convert(x: i64) -> Option<i32> {
///     x.try_into().ok()
/// }
/// ```
///
/// ### Option 2: Validate range before converting
/// ```rust
/// fn convert_with_check(x: i64) -> Result<i32, &'static str> {
///     if x > i32::MAX as i64 || x < i32::MIN as i64 {
///         Err("Value out of range")
///     } else {
///         Ok(x as i32)
///     }
/// }
/// ```
///
/// ### Option 3: Use saturating conversion
/// ```rust
/// fn saturating_convert(x: i64) -> i32 {
///     x.clamp(i32::MIN as i64, i32::MAX as i64) as i32
/// }
/// ```
///
/// Mitigation: Use `#![warn(clippy::cast_possible_truncation)]` to catch narrowing casts. Use
/// `try_into()` or `try_from()` for checked conversions. Validate that values are in range
/// before casting with `as`.

// ============================================================================
// DANGEROUS PATTERNS - NEVER DO THIS
// ============================================================================

/// PROBLEM E1404: Can lose data with 'as' cast
pub fn e1404_bad_narrowing(x: i64) -> i32 {
    // PROBLEM E1404: Can lose data with 'as' cast
    x as i32
}

/// PROBLEM E1404: u64 to u32 truncation
pub fn e1404_bad_u64_to_u32(x: u64) -> u32 {
    x as u32 // Loses upper 32 bits!
}

/// PROBLEM E1404: usize to u8 (very dangerous)
pub fn e1404_bad_usize_to_u8(x: usize) -> u8 {
    x as u8 // Can lose almost all data!
}

/// PROBLEM E1404: Float to int truncation
pub fn e1404_bad_float_to_int(x: f64) -> i32 {
    x as i32 // Truncates decimal AND can overflow!
}

/// Entry point for problem demonstration
pub fn e1404_entry() -> Result<(), Box<dyn std::error::Error>> {
    Ok(())
}

// ============================================================================
// GOOD ALTERNATIVES
// ============================================================================

/// GOOD: Use try_into for checked conversion
pub fn e1404_good_try_into(x: i64) -> Option<i32> {
    x.try_into().ok()
}

/// GOOD: Validate range before converting
pub fn e1404_good_validate(x: i64) -> Result<i32, &'static str> {
    if x > i32::MAX as i64 || x < i32::MIN as i64 {
        Err("Value out of i32 range")
    } else {
        Ok(x as i32)
    }
}

/// GOOD: Use saturating/clamping conversion
pub fn e1404_good_saturating(x: i64) -> i32 {
    x.clamp(i32::MIN as i64, i32::MAX as i64) as i32
}

/// GOOD: TryFrom with proper error handling
pub fn e1404_good_try_from(x: u64) -> Result<u32, std::num::TryFromIntError> {
    u32::try_from(x)
}

/// GOOD: Safe usize to u8 with bounds check
pub fn e1404_good_usize_to_u8(x: usize) -> Option<u8> {
    if x <= u8::MAX as usize {
        Some(x as u8)
    } else {
        None
    }
}

/// GOOD: Float to int with bounds check
pub fn e1404_good_float_to_int(x: f64) -> Option<i32> {
    if x.is_nan() || x < i32::MIN as f64 || x > i32::MAX as f64 {
        None
    } else {
        Some(x as i32)
    }
}

/// GOOD: Use larger type to avoid conversion entirely
pub fn e1404_good_use_larger_type(values: &[i32]) -> i64 {
    values.iter().map(|&x| x as i64).sum()
}

/// GOOD: Explicit truncation with wrapping semantics
pub fn e1404_good_explicit_truncate(x: u64) -> u32 {
    // When truncation is intentional (e.g., for hashing)
    (x & 0xFFFF_FFFF) as u32
}

// ============================================================================
// GOOD EXAMPLES unit tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_try_into_success() {
        assert_eq!(e1404_good_try_into(1000), Some(1000));
    }

    #[test]
    fn test_try_into_overflow() {
        assert_eq!(e1404_good_try_into(5_000_000_000), None);
    }

    #[test]
    fn test_validate_success() {
        assert_eq!(e1404_good_validate(1000), Ok(1000));
    }

    #[test]
    fn test_validate_overflow() {
        assert!(e1404_good_validate(5_000_000_000).is_err());
    }

    #[test]
    fn test_saturating_clamps() {
        assert_eq!(e1404_good_saturating(5_000_000_000), i32::MAX);
        assert_eq!(e1404_good_saturating(-5_000_000_000), i32::MIN);
        assert_eq!(e1404_good_saturating(1000), 1000);
    }

    #[test]
    fn test_usize_to_u8() {
        assert_eq!(e1404_good_usize_to_u8(100), Some(100));
        assert_eq!(e1404_good_usize_to_u8(300), None);
    }

    #[test]
    fn test_float_to_int() {
        assert_eq!(e1404_good_float_to_int(42.7), Some(42));
        assert_eq!(e1404_good_float_to_int(f64::NAN), None);
        assert_eq!(e1404_good_float_to_int(f64::MAX), None);
    }

    #[test]
    fn test_larger_type_no_overflow() {
        let values = vec![i32::MAX, i32::MAX];
        let sum = e1404_good_use_larger_type(&values);
        assert_eq!(sum, 2 * i32::MAX as i64);
    }
}
