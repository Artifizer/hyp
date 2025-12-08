/// E1401: Integer overflow/underflow (8/16/32-bit types + usize/isize)
/// Severity: HIGH
/// LLM confusion: 3 (MED)
///
/// Description: Integer overflow happens when arithmetic produces a number too large for the
/// variable type. In release builds, the number wraps around (like an odometer going past its max),
/// causing subtle bugs. This checker covers 8/16/32-bit types plus usize/isize. For example,
/// adding 1 to u8::MAX (255) gives you 0. It's like a calculator that shows 0 when you go past 999.
///
/// **Why usize/isize are HIGH severity:** Even though they're 64-bit on modern systems, they're
/// used for array indices, lengths, and memory sizes where overflow is a critical security issue
/// (buffer overflows, incorrect memory allocation). Treat size/index arithmetic conservatively!
///
/// Note: Explicit i64/u64 used for large arbitrary numbers are covered by E1413 with lower severity.
///
/// ## The Wrapping Problem
///
/// ```text
/// let x: u8 = 255;
/// let y = x + 1;  // In debug: panic! In release: y = 0 (wrapped!)
///
/// let z: u8 = 200;
/// let w = z + 100; // In debug: panic! In release: w = 44 (wrapped!)
/// ```
///
/// ## Why This Matters
///
/// 1. **Silent corruption**: Release builds wrap silently, debug builds panic
/// 2. **Security vulnerabilities**: Buffer size calculations can wrap to small values
/// 3. **Financial errors**: Money calculations can produce wrong amounts
/// 4. **Logic bugs**: Loop counters can wrap causing infinite loops
///
/// ## The Right Solutions
///
/// ### Option 1: Use checked_* methods
/// ```rust
/// fn safe_add(x: u8, y: u8) -> Option<u8> {
///     x.checked_add(y)
/// }
/// ```
///
/// ### Option 2: Use saturating_* methods
/// ```rust
/// fn capped_add(x: u8, y: u8) -> u8 {
///     x.saturating_add(y)  // Returns 255 if it would overflow
/// }
/// ```
///
/// ### Option 3: Use wrapping_* when wrapping is intentional
/// ```rust
/// fn hash_combine(a: u64, b: u64) -> u64 {
///     a.wrapping_add(b).wrapping_mul(31)  // Intentional wrapping
/// }
/// ```
///
/// Mitigation: Use `#![warn(clippy::integer_arithmetic)]` to catch unchecked arithmetic. Use
/// `checked_add()`, `saturating_add()`, or `wrapping_add()` to make overflow behavior explicit.
/// Enable overflow checks in release builds with `overflow-checks = true` in Cargo.toml.

// ============================================================================
// DANGEROUS PATTERNS - NEVER DO THIS
// ============================================================================

/// PROBLEM E1401: u8 addition overflow (very common)
pub fn e1401_bad_u8_overflow(x: u8) -> u8 {
    // u8 max is 255, adding 100 to values > 155 will overflow
    x + 100
}

/// PROBLEM E1401: u16 multiplication overflow
pub fn e1401_bad_u16_multiply(x: u16, y: u16) -> u16 {
    // u16 max is 65,535 - easy to overflow with multiplication
    x * y
}

/// PROBLEM E1401: u32 underflow with subtraction
pub fn e1401_bad_u32_underflow(x: u32, y: u32) -> u32 {
    // Underflows if y > x, wraps to large value
    x - y
}

/// PROBLEM E1401: i32 addition overflow
pub fn e1401_bad_i32_overflow(x: i32) -> i32 {
    // i32 max is 2,147,483,647
    x + 1_000_000_000
}

/// PROBLEM E1401: u8 loop counter overflow
pub fn e1401_bad_loop_overflow() -> u8 {
    let mut sum: u8 = 0;
    for i in 0..200u8 {
        sum += i; // Will overflow! Sum of 0..200 is 19,900
    }
    sum
}

/// PROBLEM E1401: usize buffer size overflow (SECURITY CRITICAL!)
pub fn e1401_bad_usize_buffer_size(count: usize) -> usize {
    // This is a classic security vulnerability - buffer size calculation overflow
    count * 1024usize // Can wrap to small value, leading to buffer overflow!
}

/// PROBLEM E1401: isize offset calculation overflow
pub fn e1401_bad_isize_offset(base: isize) -> isize {
    // Pointer arithmetic with isize can overflow
    base + 10000isize
}

/// Entry point for problem demonstration
pub fn e1401_entry() -> Result<(), Box<dyn std::error::Error>> {
    Ok(())
}

// ============================================================================
// GOOD ALTERNATIVES
// ============================================================================

/// GOOD: Use checked_add for safe addition
pub fn e1401_good_checked_add(x: u8, y: u8) -> Option<u8> {
    x.checked_add(y)
}

/// GOOD: Use saturating_add to cap at max value
pub fn e1401_good_saturating_add(x: u8, y: u8) -> u8 {
    x.saturating_add(y) // Returns 255 if would overflow
}

/// GOOD: Use wrapping_add when wrapping is intentional
pub fn e1401_good_wrapping_add(x: u8, y: u8) -> u8 {
    x.wrapping_add(y) // Explicit that wrapping is expected
}

/// GOOD: Use overflowing_add to detect overflow
pub fn e1401_good_overflowing_add(x: u8, y: u8) -> (u8, bool) {
    x.overflowing_add(y) // Returns (result, did_overflow)
}

/// GOOD: Use larger type when overflow is possible (u8 -> u16)
pub fn e1401_good_wider_type(x: u8, y: u8) -> u16 {
    (x as u16) + (y as u16) // Cannot overflow u16
}

/// GOOD: Checked multiplication for u16
pub fn e1401_good_checked_multiply(x: u16, y: u16) -> Option<u16> {
    x.checked_mul(y)
}

/// GOOD: Checked subtraction for u32
pub fn e1401_good_checked_sub(x: u32, y: u32) -> Option<u32> {
    x.checked_sub(y)
}

/// GOOD: Checked addition for i32
pub fn e1401_good_checked_i32(x: i32, y: i32) -> Option<i32> {
    x.checked_add(y)
}

/// GOOD: Safe u8 loop with wider accumulator
pub fn e1401_good_safe_loop() -> u16 {
    let mut sum: u16 = 0;
    for i in 0..200u8 {
        sum += i as u16; // Use u16 to avoid overflow
    }
    sum
}

/// GOOD: Safe u8 loop with checked arithmetic
pub fn e1401_good_safe_loop_checked() -> Option<u16> {
    let mut sum: u16 = 0;
    for i in 0..200u16 {
        sum = sum.checked_add(i)?;
    }
    Some(sum)
}

/// GOOD: Checked usize buffer size calculation
pub fn e1401_good_checked_buffer_size(count: usize, item_size: usize) -> Option<usize> {
    count.checked_mul(item_size) // Returns None on overflow - prevents buffer overflow!
}

/// GOOD: Saturating usize for array indexing
pub fn e1401_good_saturating_usize(index: usize, offset: usize, max: usize) -> usize {
    index.saturating_add(offset).min(max) // Safe index calculation
}

/// GOOD: Checked isize offset
pub fn e1401_good_checked_isize_offset(base: isize, offset: isize) -> Option<isize> {
    base.checked_add(offset)
}

// ============================================================================
// GOOD EXAMPLES unit tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_checked_add_success() {
        assert_eq!(e1401_good_checked_add(100, 50), Some(150));
    }

    #[test]
    fn test_checked_add_overflow() {
        assert_eq!(e1401_good_checked_add(200, 100), None);
    }

    #[test]
    fn test_saturating_add_caps() {
        assert_eq!(e1401_good_saturating_add(200, 100), 255);
    }

    #[test]
    fn test_wrapping_add_wraps() {
        assert_eq!(e1401_good_wrapping_add(200, 100), 44); // 300 % 256 = 44
    }

    #[test]
    fn test_overflowing_add_detects() {
        assert_eq!(e1401_good_overflowing_add(200, 100), (44, true));
        assert_eq!(e1401_good_overflowing_add(100, 50), (150, false));
    }

    #[test]
    fn test_wider_type_no_overflow() {
        assert_eq!(e1401_good_wider_type(200, 200), 400);
    }

    #[test]
    fn test_checked_sub_underflow() {
        assert_eq!(e1401_good_checked_sub(5, 10), None);
        assert_eq!(e1401_good_checked_sub(10, 5), Some(5));
    }

    #[test]
    fn test_checked_i32_success() {
        assert_eq!(e1401_good_checked_i32(100, 200), Some(300));
    }

    #[test]
    fn test_checked_i32_overflow() {
        assert_eq!(e1401_good_checked_i32(i32::MAX, 1), None);
    }

    #[test]
    fn test_safe_loop() {
        let result = e1401_good_safe_loop();
        assert_eq!(result, 19900); // Sum of 0..200
    }

    #[test]
    fn test_safe_loop_checked() {
        let result = e1401_good_safe_loop_checked();
        assert!(result.is_some());
        assert_eq!(result.unwrap(), 19900); // Sum of 0..200
    }

    #[test]
    fn test_checked_buffer_size_success() {
        assert_eq!(e1401_good_checked_buffer_size(100, 8), Some(800));
    }

    #[test]
    fn test_checked_buffer_size_overflow() {
        assert_eq!(e1401_good_checked_buffer_size(usize::MAX, 2), None);
    }

    #[test]
    fn test_saturating_usize() {
        assert_eq!(e1401_good_saturating_usize(100, 50, 200), 150);
        assert_eq!(e1401_good_saturating_usize(190, 50, 200), 200); // Capped
    }

    #[test]
    fn test_checked_isize_offset() {
        assert_eq!(e1401_good_checked_isize_offset(100, 50), Some(150));
        assert_eq!(e1401_good_checked_isize_offset(isize::MAX, 1), None);
    }
}
