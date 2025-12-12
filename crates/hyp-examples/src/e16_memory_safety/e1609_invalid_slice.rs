/// E1609: Invalid slice creation
/// Severity: HIGH
/// LLM confusion: 4 (HIGH)
///
/// Description: Creating a slice from raw parts requires the pointer and length to be valid.
/// This code creates a slice claiming to have 100 elements when the actual data only has 3.
/// Accessing elements beyond the actual data causes undefined behavior. Fix by ensuring the
/// length matches the actual allocated size.
///
/// ## The Invalid Slice Problem
///
/// ```text
/// let data = [1, 2, 3];
/// let ptr = data.as_ptr();
///
/// unsafe {
///     let slice = std::slice::from_raw_parts(ptr, 100);  // Only 3 elements!
///     slice[50];  // Reading garbage or crashing
/// }
/// ```
///
/// ## Why This Matters
///
/// 1. **Undefined behavior**: Reading past allocated memory
/// 2. **Security vulnerability**: Information disclosure
/// 3. **Crashes**: May segfault on access
/// 4. **Silent corruption**: May read garbage data
///
/// ## The Right Solutions
///
/// ### Option 1: Use correct length
/// ```rust
/// let data = [1, 2, 3];
/// let ptr = data.as_ptr();
/// let len = data.len();
///
/// unsafe {
///     let slice = std::slice::from_raw_parts(ptr, len);  // Correct!
/// }
/// ```
///
/// ### Option 2: Use safe slice creation
/// ```rust
/// let data = [1, 2, 3];
/// let slice: &[i32] = &data;  // Safe!
/// ```
///
/// ### Option 3: Validate length before creation
/// ```rust
/// fn safe_slice(ptr: *const i32, len: usize, max_len: usize) -> Option<&[i32]> {
///     if len <= max_len {
///         Some(unsafe { std::slice::from_raw_parts(ptr, len) })
///     } else {
///         None
///     }
/// }
/// ```
///
/// Mitigation: Only use `from_raw_parts` when you can guarantee the pointer and length are valid.
/// Document the safety invariants. Prefer safe Rust slicing when possible. Use Miri to detect
/// invalid slice creation.

// ============================================================================
// DANGEROUS PATTERNS - NEVER DO THIS
// ============================================================================

/// PROBLEM E1609: Creating slice with length beyond actual data
pub fn e1609_bad_invalid_slice() {
    let data = [1, 2, 3];
    let ptr = data.as_ptr();

    // PROBLEM E1003: Direct use of unsafe code
    unsafe {
        // PROBLEM E1004: No safety documentation
        // PROBLEM E1609: Creating slice with length beyond actual data
        let _slice = std::slice::from_raw_parts(ptr, 100);
    }
}

/// PROBLEM E1609: Null pointer slice
#[allow(invalid_null_arguments)]
pub fn e1609_bad_null_slice() {
    unsafe {
        // PROBLEM E1609: Null pointer is never valid for slices
        let _slice = std::slice::from_raw_parts(std::ptr::null::<i32>(), 0);
    }
}

/// Entry point for problem demonstration
pub fn e1609_entry() -> Result<(), Box<dyn std::error::Error>> {
    Ok(())
}

// ============================================================================
// GOOD ALTERNATIVES
// ============================================================================

/// GOOD: Use correct length
pub fn e1609_good_correct_length() {
    let data = [1, 2, 3];
    let ptr = data.as_ptr();
    let len = data.len();

    // SAFETY: ptr is valid for len elements, data is not moved
    unsafe {
        let slice = std::slice::from_raw_parts(ptr, len);
        assert_eq!(slice, &[1, 2, 3]);
    }
}

/// GOOD: Use safe slice creation
pub fn e1609_good_safe_slice() {
    let data = [1, 2, 3];
    let slice: &[i32] = &data; // Safe!
    assert_eq!(slice.len(), 3);
}

/// GOOD: Validate length before creation
pub fn e1609_good_validated(ptr: *const i32, len: usize, max_len: usize) -> Option<&'static [i32]> {
    if ptr.is_null() {
        return None;
    }
    if len > max_len {
        return None;
    }

    // SAFETY: We verified ptr is non-null and len <= max_len
    // Caller must ensure ptr is valid for len elements
    Some(unsafe { std::slice::from_raw_parts(ptr, len) })
}

/// GOOD: Use NonNull for non-null guarantee
pub fn e1609_good_nonnull() {
    use std::ptr::NonNull;

    let mut data = [1, 2, 3];
    let ptr = NonNull::new(data.as_mut_ptr()).expect("Array pointer is never null");

    // SAFETY: NonNull guarantees non-null, and we know the length
    unsafe {
        let slice = std::slice::from_raw_parts(ptr.as_ptr(), data.len());
        assert_eq!(slice, &[1, 2, 3]);
    }
}

/// GOOD: Use empty slice for zero length
pub fn e1609_good_empty_slice() -> &'static [i32] {
    // NonNull::dangling() is valid for zero-length slices
    unsafe { std::slice::from_raw_parts(std::ptr::NonNull::dangling().as_ptr(), 0) }
}

/// GOOD: Wrapper that tracks valid length
pub struct ValidSlice<'a, T> {
    data: &'a [T],
}

impl<'a, T> ValidSlice<'a, T> {
    pub fn new(data: &'a [T]) -> Self {
        Self { data }
    }

    pub fn get(&self, index: usize) -> Option<&T> {
        self.data.get(index)
    }

    pub fn as_slice(&self) -> &[T] {
        self.data
    }
}

// ============================================================================
// GOOD EXAMPLES unit tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_correct_length() {
        e1609_good_correct_length();
    }

    #[test]
    fn test_safe_slice() {
        e1609_good_safe_slice();
    }

    #[test]
    fn test_empty_slice() {
        let slice = e1609_good_empty_slice();
        assert!(slice.is_empty());
    }

    #[test]
    fn test_valid_slice_wrapper() {
        let data = [1, 2, 3];
        let valid = ValidSlice::new(&data);
        assert_eq!(valid.get(1), Some(&2));
        assert_eq!(valid.get(10), None);
    }
}
