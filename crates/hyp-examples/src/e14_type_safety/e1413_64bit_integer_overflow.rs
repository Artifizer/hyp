/// E1413: 64-bit integer overflow/underflow (i64/u64 only)
/// Severity: LOW
/// LLM confusion: 2 (LOW)
///
/// Description: While 64-bit integer overflow is less common than smaller types due to their
/// enormous range (0 to 18,446,744,073,709,551,615 for u64), it can still occur in specific
/// scenarios: large data processing, timestamps (especially nanosecond precision), file sizes,
/// cryptographic operations, or accumulating counters in high-throughput systems.
///
/// **Note:** This checker covers only explicit i64/u64 types used for large arbitrary numbers.
/// usize/isize are covered by E1401 (HIGH severity) because they're used for security-critical
/// size/index calculations where overflow can lead to buffer overflows.
///
/// ## When 64-bit Overflow Matters
///
/// ```text
/// // Nanosecond timestamps can overflow
/// let nanos: u64 = u64::MAX;
/// let more_nanos = nanos + 1_000_000_000;  // Wraps to small value!
///
/// // Large file size calculations
/// let size: u64 = 10_000_000_000_000;  // 10 TB
/// let total = size * 2;  // Could overflow with large multipliers
///
/// // Cryptographic operations with large numbers
/// let hash1: u64 = 0xFFFFFFFFFFFFFF00;
/// let hash2 = hash1 + 0x200;  // Wraps around
/// ```
///
/// ## Why This Matters (But Less Critical)
///
/// 1. **Rare but possible**: Takes extreme values to overflow, but can happen
/// 2. **Specific domains**: Timestamps, file systems, crypto, high-frequency counters
/// 3. **Silent wrapping**: Same wrapping behavior as smaller types in release mode
/// 4. **Use u128 when needed**: For truly large calculations, consider 128-bit types
///
/// ## The Right Solutions
///
/// ### Option 1: Use checked_* methods
/// ```rust
/// fn safe_timestamp_add(ts: u64, delta: u64) -> Option<u64> {
///     ts.checked_add(delta)
/// }
/// ```
///
/// ### Option 2: Use saturating_* methods
/// ```rust
/// fn capped_accumulator(count: u64, increment: u64) -> u64 {
///     count.saturating_add(increment)  // Caps at u64::MAX
/// }
/// ```
///
/// ### Option 3: Use u128 for very large calculations
/// ```rust
/// fn large_multiply(a: u64, b: u64) -> u128 {
///     (a as u128) * (b as u128)  // Cannot overflow u128
/// }
/// ```
///
/// ### Option 4: Use wrapping_* when wrapping is intentional
/// ```rust
/// fn hash_combine(a: u64, b: u64) -> u64 {
///     a.wrapping_add(b).wrapping_mul(0x9e3779b97f4a7c15)  // Intentional wrapping
/// }
/// ```
///
/// Mitigation: Use `#![warn(clippy::integer_arithmetic)]` to catch unchecked arithmetic.
/// For critical applications, use checked_* methods. Consider u128 for calculations that
/// might exceed u64 range.

// ============================================================================
// PROBLEMATIC PATTERNS
// ============================================================================

/// PROBLEM E1413: Timestamp overflow with nanoseconds
pub fn e1413_bad_timestamp_overflow(timestamp_nanos: u64) -> u64 {
    // Adding large nanosecond values can overflow
    timestamp_nanos + 1_000_000_000_000u64  // 1 trillion nanoseconds
}

/// PROBLEM E1413: File size calculation overflow
pub fn e1413_bad_file_size_multiply(size_bytes: u64, count: u64) -> u64 {
    // Multiplying large file sizes can overflow
    size_bytes * count
}

/// PROBLEM E1413: Accumulator overflow in high-throughput system
pub fn e1413_bad_counter_accumulate(counter: u64, increment: u64) -> u64 {
    // High-frequency counter can eventually overflow
    counter + increment
}

/// PROBLEM E1413: Large number subtraction underflow
pub fn e1413_bad_large_subtraction(a: u64, b: u64) -> u64 {
    // Can underflow if b > a
    a - b
}

/// Entry point for problem demonstration
pub fn e1413_entry() -> Result<(), Box<dyn std::error::Error>> {
    // Safe calls with small values
    let _ = e1413_bad_timestamp_overflow(1000);
    let _ = e1413_bad_file_size_multiply(100, 10);
    let _ = e1413_bad_counter_accumulate(500, 50);
    let _ = e1413_bad_large_subtraction(1000, 100);
    Ok(())
}

// ============================================================================
// GOOD ALTERNATIVES
// ============================================================================

/// GOOD: Use checked_add for timestamp operations
pub fn e1413_good_checked_timestamp(timestamp_nanos: u64, delta: u64) -> Option<u64> {
    timestamp_nanos.checked_add(delta)
}

/// GOOD: Use checked_mul for file size calculations
pub fn e1413_good_checked_file_size(size_bytes: u64, count: u64) -> Option<u64> {
    size_bytes.checked_mul(count)
}

/// GOOD: Use saturating_add for counters that should cap
pub fn e1413_good_saturating_counter(counter: u64, increment: u64) -> u64 {
    counter.saturating_add(increment)  // Caps at u64::MAX
}

/// GOOD: Use checked_sub to detect underflow
pub fn e1413_good_checked_sub(a: u64, b: u64) -> Option<u64> {
    a.checked_sub(b)
}

/// GOOD: Use u128 for very large calculations
pub fn e1413_good_u128_multiply(a: u64, b: u64) -> u128 {
    (a as u128) * (b as u128)  // Cannot overflow u128
}

/// GOOD: Use wrapping_* when wrapping is intentional (e.g., hashing)
pub fn e1413_good_wrapping_hash(a: u64, b: u64) -> u64 {
    // Intentional wrapping for hash combination
    a.wrapping_add(b).wrapping_mul(0x9e3779b97f4a7c15)
}

/// GOOD: Use overflowing_* to detect and handle overflow
pub fn e1413_good_overflowing_add(a: u64, b: u64) -> Result<u64, String> {
    let (result, overflowed) = a.overflowing_add(b);
    if overflowed {
        Err("Overflow detected".to_string())
    } else {
        Ok(result)
    }
}

/// GOOD: Validate inputs before arithmetic
pub fn e1413_good_validated_multiply(a: u64, b: u64) -> Result<u64, String> {
    // Check if multiplication would overflow
    if b != 0 && a > u64::MAX / b {
        Err("Multiplication would overflow".to_string())
    } else {
        Ok(a * b)
    }
}

// ============================================================================
// GOOD EXAMPLES unit tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_checked_timestamp_success() {
        assert_eq!(e1413_good_checked_timestamp(1000, 500), Some(1500));
    }

    #[test]
    fn test_checked_timestamp_overflow() {
        assert_eq!(e1413_good_checked_timestamp(u64::MAX, 1), None);
    }

    #[test]
    fn test_checked_file_size_success() {
        assert_eq!(e1413_good_checked_file_size(1000, 1000), Some(1_000_000));
    }

    #[test]
    fn test_checked_file_size_overflow() {
        assert_eq!(e1413_good_checked_file_size(u64::MAX, 2), None);
    }

    #[test]
    fn test_saturating_counter_caps() {
        assert_eq!(e1413_good_saturating_counter(u64::MAX, 1000), u64::MAX);
    }

    #[test]
    fn test_checked_sub_success() {
        assert_eq!(e1413_good_checked_sub(1000, 500), Some(500));
    }

    #[test]
    fn test_checked_sub_underflow() {
        assert_eq!(e1413_good_checked_sub(500, 1000), None);
    }

    #[test]
    fn test_u128_multiply_no_overflow() {
        let result = e1413_good_u128_multiply(u64::MAX, 2);
        assert_eq!(result, (u64::MAX as u128) * 2);
    }

    #[test]
    fn test_wrapping_hash() {
        // Just verify it doesn't panic
        let result = e1413_good_wrapping_hash(u64::MAX, 1);
        assert!(result < u64::MAX);  // Wrapped around
    }

    #[test]
    fn test_overflowing_add_success() {
        assert_eq!(e1413_good_overflowing_add(1000, 500), Ok(1500));
    }

    #[test]
    fn test_overflowing_add_detects_overflow() {
        assert!(e1413_good_overflowing_add(u64::MAX, 1).is_err());
    }

    #[test]
    fn test_validated_multiply_success() {
        assert_eq!(e1413_good_validated_multiply(1000, 1000), Ok(1_000_000));
    }

    #[test]
    fn test_validated_multiply_detects_overflow() {
        assert!(e1413_good_validated_multiply(u64::MAX, 2).is_err());
    }
}
