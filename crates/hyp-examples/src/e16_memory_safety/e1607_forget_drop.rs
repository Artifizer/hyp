/// E1607: Forgetting to drop
/// Severity: MED
/// LLM confusion: 3 (MED)
///
/// Description: `mem::forget` prevents a value's destructor from running, which means its
/// resources are never cleaned up. For a Vec, this leaks the heap-allocated memory. This is
/// safe (won't corrupt memory) but wastes resources. Fix by letting values drop normally, or
/// use `ManuallyDrop` if you truly need to prevent cleanup.
///
/// ## The Forget Problem
///
/// ```text
/// let data = vec![1, 2, 3];  // Allocates heap memory
/// std::mem::forget(data);     // Destructor never runs
/// // Memory is leaked forever
/// ```
///
/// ## Why This Matters
///
/// 1. **Memory leak**: Heap allocations never freed
/// 2. **Resource leak**: File handles, sockets, etc. not closed
/// 3. **Accumulating waste**: Long-running programs grow forever
/// 4. **Safe but wrong**: Rust allows it but it's usually a bug
///
/// ## The Right Solutions
///
/// ### Option 1: Let values drop naturally
/// ```rust
/// {
///     let data = vec![1, 2, 3];
///     // Use data...
/// }  // data dropped here automatically
/// ```
///
/// ### Option 2: Use ManuallyDrop for explicit control
/// ```rust
/// use std::mem::ManuallyDrop;
///
/// let data = ManuallyDrop::new(vec![1, 2, 3]);
/// // Use data...
/// // Explicitly drop when ready:
/// // unsafe { ManuallyDrop::drop(&mut data); }
/// ```
///
/// ### Option 3: Use into_raw for FFI (with from_raw later)
/// ```rust
/// let data = vec![1, 2, 3];
/// let ptr = Box::into_raw(data.into_boxed_slice());
/// // Pass ptr to C code...
/// // Later: let _ = unsafe { Box::from_raw(ptr) };
/// ```
///
/// Mitigation: Avoid `mem::forget` unless you have a very specific reason. Use `ManuallyDrop`
/// for explicit control over when destructors run. Understand that forgetting is safe but can
/// leak resources. Use `#![warn(clippy::mem_forget)]` to detect forget calls.

use std::mem::ManuallyDrop;

// ============================================================================
// PROBLEMATIC PATTERNS
// ============================================================================

/// PROBLEM E1607: Explicitly forgetting to drop, leaks memory
pub fn e1607_bad_forget() {
    let data = vec![1, 2, 3];
    // PROBLEM E1607: Explicitly forgetting to drop, leaks memory
    std::mem::forget(data);
}

/// PROBLEM E1607: Forgetting file handle leaks resource
pub fn e1607_bad_forget_file() {
    use std::fs::File;

    if let Ok(file) = File::open("test.txt") {
        std::mem::forget(file); // File handle leaked!
    }
}

/// Entry point for problem demonstration
pub fn e1607_entry() -> Result<(), Box<dyn std::error::Error>> {
    Ok(())
}

// ============================================================================
// GOOD ALTERNATIVES
// ============================================================================

/// GOOD: Let values drop naturally
pub fn e1607_good_natural_drop() {
    let data = vec![1, 2, 3];
    // Use data...
    println!("{:?}", data);
} // data dropped here automatically

/// GOOD: Use ManuallyDrop for explicit control
pub fn e1607_good_manually_drop() {
    let mut data = ManuallyDrop::new(vec![1, 2, 3]);

    // Use data...
    println!("{:?}", *data);

    // Explicitly drop when ready
    unsafe {
        ManuallyDrop::drop(&mut data);
    }
}

/// GOOD: Use into_raw/from_raw for FFI
pub fn e1607_good_ffi_pattern() {
    let data = vec![1, 2, 3];
    let ptr = Box::into_raw(data.into_boxed_slice());

    // Simulate passing to C code and getting back
    // In real code, C would return the pointer

    // Reclaim ownership and drop
    let _reclaimed = unsafe { Box::from_raw(ptr) };
    // Automatically dropped here
}

/// GOOD: Use ManuallyDrop::take to extract value
pub fn e1607_good_take() -> Vec<i32> {
    let mut data = ManuallyDrop::new(vec![1, 2, 3]);

    // Take ownership out of ManuallyDrop
    unsafe { ManuallyDrop::take(&mut data) }
    // Now the returned Vec will be dropped normally
}

/// GOOD: Explicit drop for clarity
pub fn e1607_good_explicit_drop() {
    let data = vec![1, 2, 3];
    println!("{:?}", data);
    drop(data); // Explicit but same as letting it go out of scope
}

/// GOOD: Use scope to control lifetime
pub fn e1607_good_scoped() {
    let result = {
        let temp = [1, 2, 3];
        temp.iter().sum::<i32>()
    }; // temp dropped here

    println!("Sum: {}", result);
}

// ============================================================================
// GOOD EXAMPLES unit tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_natural_drop() {
        e1607_good_natural_drop();
    }

    #[test]
    fn test_manually_drop() {
        e1607_good_manually_drop();
    }

    #[test]
    fn test_take() {
        let data = e1607_good_take();
        assert_eq!(data, vec![1, 2, 3]);
    }
}
