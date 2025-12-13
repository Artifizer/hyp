/// E1005: Unsafe precondition violation
/// Severity: HIGH
/// LLM confusion: 4 (HIGH)
///
/// Description: This code violates the safety requirements (preconditions) of an unsafe operation.
/// Dereferencing a raw pointer is only safe if the pointer is valid, properly aligned, and points
/// to initialized memory. This code creates a pointer to a local variable, then the variable goes
/// out of scope (is destroyed), making the pointer invalid. Dereferencing it after that is undefined
/// behavior - it might crash, return garbage, or appear to work but corrupt memory.
///
/// Mitigation: Carefully read and follow the safety documentation for all unsafe operations. Use
/// tools like Miri to detect undefined behavior. Ensure pointers remain valid for their entire
/// lifetime. Prefer safe abstractions over raw pointers. Document all safety requirements.

pub fn e1005_bad_raw_pointer_deref() {
    let ptr: *const i32;
    {
        let x = 42;
        ptr = &x as *const i32;
    } // x goes out of scope here, ptr is now dangling

    // PROBLEM E1003: Direct use of unsafe code
    unsafe {
        // PROBLEM E1908: No safety documentation
        // PROBLEM E1005: Dereferencing dangling pointer (undefined behavior)
        let _value = *ptr;
    }
}

pub fn e1005_entry() -> Result<(), Box<dyn std::error::Error>> {
    e1005_bad_raw_pointer_deref();
    Ok(())
}

// ============================================================================
// GOOD EXAMPLES - Proper alternatives
// ============================================================================

use crate::test_constants::MAGIC_I32;

/// GOOD: Use safe references instead of raw pointers
pub fn e1005_good_use_references() -> i32 {
    let x = MAGIC_I32;
    let reference = &x; // Safe reference with lifetime tracking
    *reference // Compiler ensures x is still valid
}

/// GOOD: Return owned data instead of pointers
pub fn e1005_good_return_owned() -> i32 {

    MAGIC_I32 // Return by value, no pointer needed
}


// ============================================================================
// GOOD EXAMPLES unit tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn e1005_good_use_references_reads_value() {
        assert_eq!(e1005_good_use_references(), 42);
    }

    #[test]
    fn e1005_good_return_owned_returns_value() {
        assert_eq!(e1005_good_return_owned(), 42);
    }
}
