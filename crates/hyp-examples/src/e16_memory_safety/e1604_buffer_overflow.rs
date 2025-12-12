/// E1604: Buffer overflow
/// Severity: HIGH
/// LLM confusion: 4 (HIGH)
///
/// Description: Buffer overflow occurs when you write beyond the bounds of allocated memory.
/// In safe Rust, array indexing is bounds-checked and will panic. In unsafe code with raw
/// pointers, you can write past the end of a buffer, corrupting adjacent memory. Fix by always
/// validating indices and buffer sizes before pointer arithmetic.
///
/// ## The Overflow Problem
///
/// ```text
/// let mut buffer = [0u8; 10];
/// let ptr = buffer.as_mut_ptr();
///
/// unsafe {
///     *ptr.add(15) = 42;  // Writing past end of buffer!
/// }
/// // Corrupts whatever memory is after buffer
/// ```
///
/// ## Why This Matters
///
/// 1. **Memory corruption**: Adjacent data gets overwritten
/// 2. **Security vulnerabilities**: Classic exploit vector
/// 3. **Crashes**: May cause segfault
/// 4. **Undefined behavior**: Anything can happen
///
/// ## The Right Solutions
///
/// ### Option 1: Use safe indexing (panics on out-of-bounds)
/// ```rust
/// let mut buffer = [0u8; 10];
/// buffer[5] = 42;  // Bounds-checked
/// ```
///
/// ### Option 2: Use .get_mut() for safe optional access
/// ```rust
/// let mut buffer = [0u8; 10];
/// if let Some(elem) = buffer.get_mut(5) {
///     *elem = 42;
/// }
/// ```
///
/// ### Option 3: Validate bounds before unsafe access
/// ```rust
/// let mut buffer = [0u8; 10];
/// let index = 5;
/// assert!(index < buffer.len());
/// unsafe {
///     *buffer.as_mut_ptr().add(index) = 42;
/// }
/// ```
///
/// Mitigation: Avoid pointer arithmetic when possible - use safe Rust slices. If pointer
/// arithmetic is necessary, carefully validate all offsets. Use `ptr.add()` instead of manual
/// arithmetic. Test with tools like AddressSanitizer and Miri to detect buffer overflows.

// ============================================================================
// DANGEROUS PATTERNS - NEVER DO THIS
// ============================================================================

/// PROBLEM E1604: Writing past buffer end
pub fn e1604_bad_overflow() {
    let mut buffer = [0u8; 10];
    let index = 15;

    // PROBLEM E1604: No bounds checking (will panic in safe Rust)
    // buffer[index] = 42;  // This would panic

    // PROBLEM E1003: Direct use of unsafe code
    unsafe {
        // PROBLEM E1904: No safety documentation
        // PROBLEM E1604: Actual buffer overflow in unsafe code
        let ptr = buffer.as_mut_ptr();
        *ptr.add(index) = 42;
    }
}

/// PROBLEM E1604: Off-by-one in loop
pub fn e1604_bad_off_by_one() {
    let mut buffer = [0u8; 10];

    unsafe {
        let ptr = buffer.as_mut_ptr();
        for i in 0..=10 {
            // Should be 0..10, not 0..=10!
            *ptr.add(i) = i as u8;
        }
    }
}

/// Entry point for problem demonstration
pub fn e1604_entry() -> Result<(), Box<dyn std::error::Error>> {
    Ok(())
}

// ============================================================================
// GOOD ALTERNATIVES
// ============================================================================

/// GOOD: Use safe indexing
pub fn e1604_good_safe_index() {
    let mut buffer = [0u8; 10];
    buffer[5] = 42; // Bounds-checked, panics if out of range
    let _ = buffer; // Use buffer to avoid unused warning
}

/// GOOD: Use get_mut for optional access
pub fn e1604_good_get_mut() -> bool {
    let mut buffer = [0u8; 10];
    if let Some(elem) = buffer.get_mut(5) {
        *elem = 42;
        true
    } else {
        false
    }
}

/// GOOD: Validate before unsafe access
pub fn e1604_good_validate() {
    let mut buffer = [0u8; 10];
    let index = 5;

    assert!(index < buffer.len(), "Index out of bounds");

    // SAFETY: We just verified index < buffer.len()
    unsafe {
        *buffer.as_mut_ptr().add(index) = 42;
    }
}

/// GOOD: Use iterators instead of indexing
pub fn e1604_good_iterator() {
    let mut buffer = [0u8; 10];

    for (i, elem) in buffer.iter_mut().enumerate() {
        *elem = i as u8;
    }
}

/// GOOD: Use slice methods
pub fn e1604_good_slice_methods() {
    let mut buffer = [0u8; 10];

    // Fill with value
    buffer.fill(42);

    // Copy from another slice
    let source = [1, 2, 3];
    buffer[..3].copy_from_slice(&source);
}

/// GOOD: Use Vec with push (auto-grows)
pub fn e1604_good_vec_push() -> Vec<u8> {
    let mut buffer = Vec::with_capacity(10);

    for i in 0..15 {
        buffer.push(i); // Safe - Vec grows as needed
    }

    buffer
}

// ============================================================================
// GOOD EXAMPLES unit tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_safe_index() {
        e1604_good_safe_index();
    }

    #[test]
    fn test_get_mut_valid() {
        assert!(e1604_good_get_mut());
    }

    #[test]
    fn test_iterator() {
        e1604_good_iterator();
    }

    #[test]
    fn test_vec_push() {
        let buffer = e1604_good_vec_push();
        assert_eq!(buffer.len(), 15);
    }
}
