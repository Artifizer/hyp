/// E1011: Uninitialized memory
/// Severity: HIGH
/// LLM confusion: 5 (HIGHEST)
///
/// Description: Reading uninitialized memory is undefined behavior - the memory contains whatever
/// random bytes happened to be there before, which could be anything. This can cause crashes,
/// security vulnerabilities (leaking sensitive data from previous allocations), or unpredictable
/// behavior. It's like reading from an uninitialized variable in C - you get garbage. Rust prevents
/// this in safe code, but unsafe code can bypass the checks.
///
/// Mitigation: Never read uninitialized memory. Use `MaybeUninit<T>` when you need to work with
/// uninitialized data, and call `assume_init()` only after fully initializing it. Initialize all
/// memory before reading. Use safe constructors like `vec![0; size]` instead of uninitialized
/// allocations. Run code with Miri to detect uninitialized reads.

#[allow(invalid_value)]
pub fn e1011_bad_uninitialized_memory() {
    // PROBLEM E1003: Direct use of unsafe code
    unsafe {
        // PROBLEM E1004: No safety documentation
        // PROBLEM E1011: Reading uninitialized memory
        let x: i32 = std::mem::MaybeUninit::uninit().assume_init();
        let _value = x; // Reading garbage
    }
}

pub fn e1011_entry() -> Result<(), Box<dyn std::error::Error>> {
    e1011_bad_uninitialized_memory();
    Ok(())
}

// ============================================================================
// GOOD EXAMPLES - Proper alternatives
// ============================================================================

/// GOOD: Initialize with a value
pub fn e1011_good_initialize() {
    let x: i32 = 0; // Always initialized
    let _value = x;
}

/// GOOD: Use Default trait
pub fn e1011_good_use_default() {
    let x = i32::default(); // 0
    let _value = x;
}

/// GOOD: Use vec! macro for initialized buffers
pub fn e1011_good_vec_init() -> Vec<u8> {
    vec![0u8; 1024] // All zeros, fully initialized
}

/// GOOD: Use read_buf or similar for I/O
pub fn e1011_good_buffer_pattern() -> Vec<u8> {
    let mut buffer = vec![0u8; 1024]; // Pre-initialized
                                      // Could now pass &mut buffer to read()
    buffer
}

// ============================================================================
// GOOD EXAMPLES unit tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn e1011_good_vec_init_returns_initialized_buffer() {
        let buffer = e1011_good_vec_init();
        assert_eq!(buffer.len(), 1024);
        assert!(buffer.iter().all(|b| *b == 0));
    }
}
