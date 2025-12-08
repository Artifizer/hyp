/// E1601: Aliasing violation
/// Severity: HIGH
/// LLM confusion: 5 (HIGHEST)
///
/// Description: Aliasing violations occur when you have multiple mutable pointers to the same
/// memory location, which breaks Rust's safety guarantees. Modifying memory through one pointer
/// while another exists can cause undefined behavior because the compiler assumes mutable
/// references are exclusive. Fix by ensuring only one mutable reference exists at a time, or
/// use proper synchronization.
///
/// ## The Aliasing Problem
///
/// ```text
/// let mut data = vec![1, 2, 3];
/// let ptr1 = data.as_mut_ptr();
/// let ptr2 = data.as_mut_ptr();  // Two mutable pointers!
///
/// unsafe {
///     *ptr1 = 10;  // Modify through ptr1
///     *ptr2 = 20;  // Modify through ptr2 - UB!
/// }
/// // Compiler assumed ptr1 was exclusive, optimizations may be wrong
/// ```
///
/// ## Why This Matters
///
/// 1. **Undefined behavior**: Compiler optimizations may break
/// 2. **Data corruption**: Values may be wrong in unpredictable ways
/// 3. **Security vulnerabilities**: Memory safety guarantees lost
/// 4. **Hard to debug**: Bugs may appear/disappear with optimization levels
///
/// ## The Right Solutions
///
/// ### Option 1: Use safe Rust borrowing
/// ```rust
/// let mut data = vec![1, 2, 3];
/// data[0] = 10;  // Safe - compiler tracks the borrow
/// data[1] = 20;  // Safe - still the same mutable borrow
/// ```
///
/// ### Option 2: Use indices instead of pointers
/// ```rust
/// let mut data = vec![1, 2, 3];
/// let i = 0;
/// let j = 1;
/// data[i] = 10;
/// data[j] = 20;
/// ```
///
/// ### Option 3: Use UnsafeCell for interior mutability
/// ```rust
/// use std::cell::UnsafeCell;
///
/// let data = UnsafeCell::new(42);
/// // UnsafeCell signals to compiler that aliasing may occur
/// ```
///
/// Mitigation: Avoid creating multiple mutable raw pointers to the same data. Use safe Rust
/// borrowing rules instead of raw pointers. If raw pointers are necessary, carefully document
/// aliasing assumptions and ensure they don't overlap in time. Consider using `UnsafeCell` for
/// interior mutability.

// ============================================================================
// DANGEROUS PATTERNS - NEVER DO THIS
// ============================================================================

/// PROBLEM E1601: Multiple mutable aliases to same memory
pub fn e1601_bad_aliasing() {
    let mut data = vec![1, 2, 3];
    let ptr1 = data.as_mut_ptr();
    let ptr2 = data.as_mut_ptr();

    // PROBLEM E1003: Direct use of unsafe code
    unsafe {
        // PROBLEM E1004: No safety documentation
        // PROBLEM E1601: Multiple mutable aliases to same memory
        *ptr1 = 10;
        *ptr2 = 20;
    }
}

/// PROBLEM E1601: Aliasing through reference and pointer
pub fn e1601_bad_ref_and_ptr() {
    let mut value = 42;
    let ptr = &mut value as *mut i32;
    let reference = &mut value;

    // PROBLEM E1601: ptr and reference alias the same memory
    *reference = 10;
    unsafe {
        *ptr = 20; // UB - reference still exists
    }
}

/// Entry point for problem demonstration
pub fn e1601_entry() -> Result<(), Box<dyn std::error::Error>> {
    Ok(())
}

// ============================================================================
// GOOD ALTERNATIVES
// ============================================================================

use crate::test_constants::MAGIC_I32;

/// GOOD: Use safe borrowing
pub fn e1601_good_safe_borrow() {
    let mut data = [1, 2, 3];
    data[0] = MAGIC_I32;
    data[1] = MAGIC_I32;
    // Compiler ensures no aliasing issues
    let _sum = data[0] + data[1]; // Use the values
}

/// GOOD: Use indices
pub fn e1601_good_indices() {
    let mut data = [1, 2, 3];
    let indices = [0, 1, 2];

    for &i in &indices {
        data[i] = i as i32 * 10;
    }
}

/// GOOD: Sequential pointer use (not overlapping)
pub fn e1601_good_sequential() {
    let mut data = vec![1, 2, 3];

    unsafe {
        let ptr = data.as_mut_ptr();
        *ptr = 10;
    }

    // Later, get a new pointer
    unsafe {
        let ptr = data.as_mut_ptr().add(1);
        *ptr = 20;
    }
}

/// GOOD: Use split_at_mut for non-overlapping slices
pub fn e1601_good_split() {
    let mut data = [1, 2, 3, 4];
    let (left, right) = data.split_at_mut(2);

    // left and right are guaranteed non-overlapping
    left[0] = 10;
    right[0] = 30;
}

/// GOOD: Use Cell for interior mutability
pub fn e1601_good_cell() {
    use std::cell::Cell;

    let data = Cell::new(42);
    data.set(10);
    data.set(20);
    // No aliasing issues - Cell handles it
}

// ============================================================================
// GOOD EXAMPLES unit tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_safe_borrow() {
        e1601_good_safe_borrow();
    }

    #[test]
    fn test_indices() {
        e1601_good_indices();
    }

    #[test]
    fn test_split() {
        e1601_good_split();
    }
}
