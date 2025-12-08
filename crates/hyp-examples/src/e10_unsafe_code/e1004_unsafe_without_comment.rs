/// E1004: Unsafe without comments
/// Severity: HIGH
/// LLM confusion: 4 (HIGH)
///
/// Description: This code uses unsafe operations without documenting the safety requirements. Unsafe
/// code can cause undefined behavior (crashes, data corruption, security vulnerabilities) if the
/// safety requirements aren't met. Every unsafe block needs a SAFETY comment explaining what makes
/// it safe - what conditions must be true, what the caller must guarantee. Without documentation,
/// future maintainers can't verify the code is correct.
///
/// Mitigation: Add `// SAFETY:` comments before all unsafe blocks. Document preconditions,
/// invariants, and why the operation is safe. Use `#![forbid(unsafe_op_in_unsafe_fn)]` to require
/// explicit unsafe blocks even in unsafe functions. Minimize and isolate unsafe code.

#[allow(clippy::useless_vec)]
pub fn e1004_bad_unsafe_no_comments() {
    let data = vec![1, 2, 3, 4, 5];
    let ptr = data.as_ptr();

    // PROBLEM E1003: Direct use of unsafe code
    unsafe {
        // PROBLEM E1004: No safety documentation
        let _first = *ptr;
    }
}

pub fn e1004_entry() -> Result<(), Box<dyn std::error::Error>> {
    e1004_bad_unsafe_no_comments();
    Ok(())
}

// ============================================================================
// GOOD EXAMPLES - Proper alternatives
// ============================================================================

/// GOOD: Use safe indexing instead of unsafe pointer access
pub fn e1004_good_safe_indexing() {
    let data = [1, 2, 3, 4, 5];
    let _first = data[0]; // Bounds-checked access
}

/// GOOD: Use get() for fallible access
pub fn e1004_good_checked_access() -> Option<i32> {
    let data = [1, 2, 3, 4, 5];
    data.first().copied() // Returns None if empty
}

/// GOOD: If unsafe is required, document thoroughly
// #[allow(clippy::useless_vec)]
// pub fn e1004_good_documented_unsafe() {
//    let data = vec![1, 2, 3, 4, 5];
//    let ptr = data.as_ptr();
//
//    // SAFETY: We verified the vector is non-empty (len=5), so ptr points
//    // to valid memory. The data vector is not modified or dropped while
//    // we hold the pointer. The pointer is properly aligned for i32.
//    unsafe {
//        let _first = *ptr;
//    }
//}

// ============================================================================
// GOOD EXAMPLES unit tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn e1004_good_checked_access_returns_first() {
        let value = e1004_good_checked_access();
        assert_eq!(value, Some(1));
    }

    #[test]
    fn e1004_good_safe_indexing_reads_first() {
        e1004_good_safe_indexing();
    }
}
