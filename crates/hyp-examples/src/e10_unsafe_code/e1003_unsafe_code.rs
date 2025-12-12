/// E1003: Direct use of unsafe code
/// Severity: HIGH
/// LLM confusion: 4 (HIGH)
///
/// Description: Unsafe code bypasses Rust's compile-time safety guarantees, allowing direct memory
/// manipulation and other operations that can lead to undefined behavior, crashes, or memory
/// corruption if used incorrectly. In a production environment, such issues can be critical.
/// While `unsafe` is sometimes necessary for performance or FFI, its use must be rigorously
/// justified and carefully audited. Every `unsafe` block requires a `// SAFETY:` comment
/// explaining precisely why the code is safe, detailing all invariants maintained and
/// assumptions made. Without this documentation, verifying correctness and safely maintaining
/// the code becomes extremely difficult, increasing the risk of severe bugs.
///
/// Mitigation: Try to avoid unsafe code in release builds or move it to isolated modules
/// to make it easier to audit.

pub fn e1003_bad_unsafe_code() {
    let x = 42;
    let ptr = &x as *const i32;

    // PROBLEM E1003: Direct use of usafe code
    unsafe {
        // PROBLEM E1904: No safety comment explaining why this is safe
        let _value = *ptr;
    }
}

pub fn e1003_entry() -> Result<(), Box<dyn std::error::Error>> {
    e1003_bad_unsafe_code();
    Ok(())
}

// ============================================================================
// GOOD EXAMPLES - Proper alternatives
// ============================================================================

use crate::test_constants::MAGIC_I32;

/// GOOD: Use safe Rust abstractions instead of raw pointers
pub fn e1003_good_use_references() {
    let x = MAGIC_I32;
    let reference = &x; // Safe reference, no unsafe needed
    let _value = *reference;
}

/// GOOD: Use safe references for value access
pub fn e1003_good_safe_dereference() {
    let x = MAGIC_I32;
    // Completely safe - no raw pointers, no unsafe blocks
    let reference: &i32 = &x;
    let _value: i32 = *reference;  // Safe dereference through reference
}

/// GOOD: Encapsulate operations in safe abstractions
pub fn e1003_good_safe_wrapper(data: &[i32]) -> Option<i32> {
    // Use safe checked access instead of unsafe pointer arithmetic
    data.first().copied()
}

// ============================================================================
// GOOD EXAMPLES unit tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn e1003_good_safe_wrapper_returns_first() {
        const FIRST_ELEMENT: i32 = 1;
        const SECOND_ELEMENT: i32 = 2;
        const THIRD_ELEMENT: i32 = 3;
        let data = [FIRST_ELEMENT, SECOND_ELEMENT, THIRD_ELEMENT];
        assert_eq!(e1003_good_safe_wrapper(&data), Some(FIRST_ELEMENT));
    }

    #[test]
    fn e1003_good_safe_dereference_is_safe() {
        e1003_good_safe_dereference();
    }

    #[test]
    fn e1003_good_use_references_reads_reference() {
        e1003_good_use_references();
    }
}
