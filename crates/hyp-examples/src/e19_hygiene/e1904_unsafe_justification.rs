/// E1904: Unsafe justification requirement
/// Severity: HIGH
/// LLM confusion: 3 (MEDIUM)
///
/// Description: Requires every unsafe block to have a justification comment
/// (e.g., `// SAFETY:`).
///
/// ## Why This Matters
///
/// 1. **Safety documentation**: Forces developers to think about and document safety invariants
/// 2. **Code review**: Makes it easier to review unsafe code when reasoning is explicit
/// 3. **Maintenance**: Future maintainers understand why unsafe was necessary
/// 4. **Audit trail**: Provides searchable documentation of all unsafe usage
///
/// ## The Right Solutions
///
/// ### Option 1: Add SAFETY comment
/// ```rust
/// // SAFETY: The pointer is valid because we just allocated it
/// // and we're within bounds of the allocation.
/// unsafe {
///     *ptr = 42;
/// }
/// ```
///
/// ### Option 2: Use safe alternatives
/// ```rust
/// // Instead of:
/// // unsafe { *ptr = 42; }
///
/// // Use safe Rust:
/// let mut value = 42;
/// ```
///
/// ### Option 3: Encapsulate unsafe in safe API
/// ```rust
/// /// Safe wrapper that maintains invariants
/// pub fn set_value(data: &mut Data, value: i32) {
///     // SAFETY: Data structure maintains invariant that
///     // ptr is always valid and aligned.
///     unsafe {
///         (*data.ptr) = value;
///     }
/// }
/// ```
///
/// Mitigation: Configure E1904 in Hyp.toml to require specific comment patterns
/// (SAFETY:, UNSAFE:) and optionally restrict unsafe blocks to specific paths.

// ============================================================================
// PROBLEMATIC PATTERNS
// ============================================================================
// Note: This file demonstrates both naming conventions:
// - Structs use PascalCase: E1904GoodSafeWrapper
// - Functions use snake_case: e1904_bad_no_comment, e1904_good_with_safety_comment
// Both conventions are supported and follow Rust naming standards.

/// PROBLEM E1904: Unsafe block without justification comment
#[allow(unused_unsafe)]
pub fn e1904_bad_no_comment() -> i32 {
    unsafe {
        // No SAFETY comment explaining why this is safe
        let x = 42;
        x
    }
}

/// PROBLEM E1904: Unsafe with insufficient explanation
#[allow(unused_unsafe)]
pub fn e1904_bad_weak_comment() -> i32 {
    // This is safe
    unsafe {
        let x = 42;
        x
    }
}

/// PROBLEM E1904: Unsafe pointer dereference without justification
pub fn e1904_bad_pointer_no_comment(ptr: *const i32) -> i32 {
    unsafe {
        *ptr
    }
}

/// PROBLEM E1904: Unsafe transmute without explanation
#[allow(unnecessary_transmutes)]
pub fn e1904_bad_transmute_no_comment(x: u32) -> i32 {
    unsafe {
        std::mem::transmute(x)
    }
}

/// Entry point for problem demonstration
pub fn e1904_entry() -> Result<(), Box<dyn std::error::Error>> {
    let _ = e1904_bad_no_comment();
    let _ = e1904_bad_weak_comment();
    let value = 42i32;
    let _ = e1904_bad_pointer_no_comment(&value as *const i32);
    let _ = e1904_bad_transmute_no_comment(42u32);
    Ok(())
}

// ============================================================================
// GOOD ALTERNATIVES
// ============================================================================

/// GOOD: Unsafe with proper SAFETY comment
#[allow(unused_unsafe)]
pub fn e1904_good_with_safety_comment() -> i32 {
    // SAFETY: This unsafe block is actually safe because we're only
    // reading a local variable, not dereferencing any raw pointers.
    // In practice, this wouldn't need unsafe at all.
    unsafe {
        let x = 42;
        x
    }
}

/// GOOD: Avoid unsafe entirely when possible
pub fn e1904_good_no_unsafe() -> i32 {
    let x = 42;
    x
}

/// GOOD: Unsafe pointer with detailed justification
pub fn e1904_good_pointer_with_comment(ptr: *const i32) -> Option<i32> {
    if ptr.is_null() {
        return None;
    }
    // SAFETY: We've checked that ptr is not null. The caller must ensure
    // that ptr points to valid, initialized memory and that the memory
    // remains valid for the duration of this function call.
    unsafe {
        Some(*ptr)
    }
}

/// GOOD: Unsafe transmute with explanation
#[allow(unnecessary_transmutes)]
pub fn e1904_good_transmute_with_comment(x: u32) -> i32 {
    // SAFETY: Transmuting u32 to i32 is safe because they have the same
    // size and alignment. The bit pattern is preserved, which is the
    // intended behavior for reinterpreting the value.
    unsafe {
        std::mem::transmute(x)
    }
}

/// GOOD: Safe wrapper around unsafe code
pub struct E1904GoodSafeWrapper {
    data: Vec<i32>,
}

impl E1904GoodSafeWrapper {
    pub fn new() -> Self {
        Self { data: vec![1, 2, 3] }
    }

    pub fn get_unchecked(&self, index: usize) -> i32 {
        // SAFETY: This is safe because we check bounds before calling
        // get_unchecked. The Vec maintains the invariant that indices
        // less than len() are valid.
        unsafe {
            if index < self.data.len() {
                *self.data.get_unchecked(index)
            } else {
                0
            }
        }
    }
}

// ============================================================================
// GOOD EXAMPLES unit tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_good_with_safety() {
        assert_eq!(e1904_good_with_safety_comment(), 42);
    }

    #[test]
    fn test_good_no_unsafe() {
        assert_eq!(e1904_good_no_unsafe(), 42);
    }

    #[test]
    fn test_good_pointer() {
        let value = 42i32;
        assert_eq!(e1904_good_pointer_with_comment(&value as *const i32), Some(42));
        assert_eq!(e1904_good_pointer_with_comment(std::ptr::null()), None);
    }

    #[test]
    fn test_good_transmute() {
        assert_eq!(e1904_good_transmute_with_comment(42u32), 42i32);
    }

    #[test]
    fn test_good_safe_wrapper() {
        let wrapper = E1904GoodSafeWrapper::new();
        assert_eq!(wrapper.get_unchecked(0), 1);
        assert_eq!(wrapper.get_unchecked(1), 2);
        assert_eq!(wrapper.get_unchecked(10), 0);
    }
}
