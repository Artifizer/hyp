/// E1406: Signed/unsigned mismatch
/// Severity: MED
/// LLM confusion: 3 (MED)
///
/// Description: Comparing signed numbers (can be negative) with unsigned numbers (always positive)
/// can give wrong results. Negative signed values become huge when treated as unsigned (like -1
/// becoming 4294967295). It's like comparing temperatures in Celsius with raw binary values - the
/// comparison doesn't make sense. Fix by ensuring both values have the same signedness (both signed
/// or both unsigned) before comparing.
///
/// ## The Sign Confusion Problem
///
/// ```text
/// let signed: i32 = -1;
/// let unsigned: u32 = 1;
///
/// // Comparing directly:
/// if (signed as u32) > unsigned {  // -1 becomes 4294967295!
///     // This branch is taken! -1 appears "greater" than 1
/// }
/// ```
///
/// ## Why This Matters
///
/// 1. **Wrong comparisons**: -1 > 1 when cast to unsigned
/// 2. **Security bugs**: Negative sizes bypass bounds checks
/// 3. **Logic errors**: Conditions evaluate incorrectly
/// 4. **Silent corruption**: No warning, just wrong behavior
///
/// ## The Right Solutions
///
/// ### Option 1: Keep values in same signedness domain
/// ```rust
/// fn compare_properly(signed: i32, unsigned: u32) -> bool {
///     if signed < 0 {
///         false  // Negative is always less than unsigned
///     } else {
///         (signed as u32) > unsigned
///     }
/// }
/// ```
///
/// ### Option 2: Use i64 to hold both ranges
/// ```rust
/// fn compare_via_wider(signed: i32, unsigned: u32) -> bool {
///     (signed as i64) > (unsigned as i64)
/// }
/// ```
///
/// ### Option 3: Use TryFrom for safe conversion
/// ```rust
/// fn safe_compare(signed: i32, unsigned: u32) -> Option<bool> {
///     let u_as_i: i32 = unsigned.try_into().ok()?;
///     Some(signed > u_as_i)
/// }
/// ```
///
/// Mitigation: Use `#![warn(clippy::cast_sign_loss)]` to catch signed-to-unsigned casts. Keep
/// values in the same signedness domain. Use explicit range checks instead of casting for
/// comparisons.

// ============================================================================
// DANGEROUS PATTERNS - NEVER DO THIS
// ============================================================================

/// PROBLEM E1406: Comparing signed and unsigned can be problematic
pub fn e1406_bad_mismatch(signed: i32, unsigned: u32) -> bool {
    // PROBLEM E1406: Comparing signed and unsigned can be problematic
    signed as u32 > unsigned
}

/// PROBLEM E1406: Negative becomes huge positive
pub fn e1406_bad_negative_cast(value: i32) -> u32 {
    value as u32 // -1 becomes 4294967295!
}

/// PROBLEM E1406: Array index from signed value
pub fn e1406_bad_signed_index(arr: &[i32], index: i32) -> i32 {
    arr[index as usize] // Negative index becomes huge!
}

/// PROBLEM E1406: Size comparison with signed
pub fn e1406_bad_size_check(size: i32, max: usize) -> bool {
    (size as usize) <= max // Negative size passes check!
}

/// Entry point for problem demonstration
pub fn e1406_entry() -> Result<(), Box<dyn std::error::Error>> {
    Ok(())
}

// ============================================================================
// GOOD ALTERNATIVES
// ============================================================================

/// GOOD: Check sign before comparing
pub fn e1406_good_check_sign(signed: i32, unsigned: u32) -> bool {
    if signed < 0 {
        false // Negative is always less than any unsigned
    } else {
        (signed as u32) > unsigned
    }
}

/// GOOD: Use wider type that can hold both ranges
pub fn e1406_good_wider_type(signed: i32, unsigned: u32) -> bool {
    (signed as i64) > (unsigned as i64)
}

/// GOOD: Use TryFrom for safe conversion
pub fn e1406_good_try_from(signed: i32, unsigned: u32) -> Option<bool> {
    // Try to convert unsigned to signed for comparison
    let u_as_signed: i32 = unsigned.try_into().ok()?;
    Some(signed > u_as_signed)
}

/// GOOD: Safe array indexing with signed value
pub fn e1406_good_safe_index(arr: &[i32], index: i32) -> Option<i32> {
    if index < 0 {
        None
    } else {
        arr.get(index as usize).copied()
    }
}

/// GOOD: Safe size check
pub fn e1406_good_size_check(size: i32, max: usize) -> bool {
    size >= 0 && (size as usize) <= max
}

/// GOOD: Convert unsigned to signed with bounds check
pub fn e1406_good_to_signed(unsigned: u32) -> Option<i32> {
    if unsigned <= i32::MAX as u32 {
        Some(unsigned as i32)
    } else {
        None
    }
}

/// GOOD: Ordering comparison with proper handling
pub fn e1406_good_compare_ordering(signed: i32, unsigned: u32) -> std::cmp::Ordering {
    use std::cmp::Ordering;
    if signed < 0 {
        Ordering::Less
    } else if unsigned > i32::MAX as u32 {
        Ordering::Less // signed can't be that big
    } else {
        signed.cmp(&(unsigned as i32))
    }
}

/// GOOD: Explicit about what happens with negative
pub fn e1406_good_saturating_to_unsigned(signed: i32) -> u32 {
    if signed < 0 {
        0 // Clamp negative to 0
    } else {
        signed as u32
    }
}

// ============================================================================
// GOOD EXAMPLES unit tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use std::cmp::Ordering;

    #[test]
    fn test_bad_gives_wrong_result() {
        // -1 as u32 = 4294967295, which is > 1
        assert!(e1406_bad_mismatch(-1, 1)); // Wrong! -1 should not be > 1
    }

    #[test]
    fn test_good_check_sign() {
        assert!(!e1406_good_check_sign(-1, 1)); // Correct: -1 is not > 1
        assert!(e1406_good_check_sign(5, 3));   // Correct: 5 > 3
    }

    #[test]
    fn test_good_wider_type() {
        assert!(!e1406_good_wider_type(-1, 1)); // Correct
        assert!(e1406_good_wider_type(5, 3));   // Correct
    }

    #[test]
    fn test_good_try_from() {
        assert_eq!(e1406_good_try_from(-1, 1), Some(false));
        assert_eq!(e1406_good_try_from(5, 3), Some(true));
        // Large unsigned that doesn't fit in i32
        assert_eq!(e1406_good_try_from(0, u32::MAX), None);
    }

    #[test]
    fn test_good_safe_index() {
        let arr = [10, 20, 30];
        assert_eq!(e1406_good_safe_index(&arr, 1), Some(20));
        assert_eq!(e1406_good_safe_index(&arr, -1), None);
        assert_eq!(e1406_good_safe_index(&arr, 10), None);
    }

    #[test]
    fn test_good_size_check() {
        assert!(e1406_good_size_check(5, 10));
        assert!(!e1406_good_size_check(-1, 10)); // Negative fails
        assert!(!e1406_good_size_check(15, 10)); // Too large fails
    }

    #[test]
    fn test_good_compare_ordering() {
        assert_eq!(e1406_good_compare_ordering(-1, 1), Ordering::Less);
        assert_eq!(e1406_good_compare_ordering(5, 3), Ordering::Greater);
        assert_eq!(e1406_good_compare_ordering(3, 3), Ordering::Equal);
    }
}
