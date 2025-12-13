/// E1602: Use-after-free
/// Severity: HIGH
/// LLM confusion: 5 (HIGHEST)
///
/// Description: Use-after-free occurs when you access memory after it has been deallocated.
/// This code keeps a raw pointer to a vector, then drops the vector (freeing its memory), then
/// dereferences the pointer. This accesses freed memory, causing undefined behavior - the memory
/// might be reused for something else. Fix by ensuring pointers don't outlive the data they point to.
///
/// ## The Dangling Pointer Problem
///
/// ```text
/// let data = vec![1, 2, 3];
/// let ptr = data.as_ptr();  // Pointer to vec's buffer
/// drop(data);               // Vec deallocated, buffer freed
/// unsafe { *ptr }           // BOOM! Accessing freed memory
/// ```
///
/// ## Why This Matters
///
/// 1. **Undefined behavior**: Anything can happen
/// 2. **Security vulnerabilities**: Exploitable for code execution
/// 3. **Data corruption**: Reading garbage or other data
/// 4. **Crashes**: Segfaults, access violations
///
/// ## The Right Solutions
///
/// ### Option 1: Use references with lifetimes
/// ```rust
/// fn get_first(data: &[i32]) -> Option<&i32> {
///     data.first()  // Lifetime tied to data
/// }
/// ```
///
/// ### Option 2: Clone data instead of pointing
/// ```rust
/// let data = vec![1, 2, 3];
/// let first = data[0];  // Copy the value
/// drop(data);
/// println!("{}", first);  // Safe - we own the copy
/// ```
///
/// ### Option 3: Use Rc/Arc for shared ownership
/// ```rust
/// use std::rc::Rc;
///
/// let data = Rc::new(vec![1, 2, 3]);
/// let data2 = Rc::clone(&data);
/// drop(data);  // Still alive via data2
/// println!("{:?}", data2);
/// ```
///
/// Mitigation: Avoid raw pointers when possible - use references which are lifetime-checked. If
/// raw pointers are necessary, carefully track the lifetime of pointed-to data. Use tools like
/// Miri to detect use-after-free bugs. Never dereference a pointer after its data is dropped.

// ============================================================================
// DANGEROUS PATTERNS - NEVER DO THIS
// ============================================================================

/// PROBLEM E1602: Accessing freed memory
pub fn e1602_bad_use_after_free() {
    let data = vec![1, 2, 3];
    let ptr = data.as_ptr();
    drop(data);

    // PROBLEM E1003: Direct use of unsafe code
    unsafe {
        // PROBLEM E1908: No safety documentation
        // PROBLEM E1602: Accessing freed memory
        let _value = *ptr;
    }
}

/// PROBLEM E1602: Returning pointer to local
#[allow(dangling_pointers_from_locals)]
pub fn e1602_bad_return_local_ptr() -> *const i32 {
    let local = 42;
    &local as *const i32 // Pointer to stack memory that will be freed
}

/// Entry point for problem demonstration
pub fn e1602_entry() -> Result<(), Box<dyn std::error::Error>> {
    Ok(())
}

// ============================================================================
// GOOD ALTERNATIVES
// ============================================================================

/// GOOD: Use references with proper lifetimes
pub fn e1602_good_reference(data: &[i32]) -> Option<&i32> {
    data.first()
}

/// GOOD: Clone/copy the data
pub fn e1602_good_copy() {
    let data = vec![1, 2, 3];
    let first = data[0]; // Copy the value
    drop(data);
    println!("{}", first); // Safe - we own the copy
}

/// GOOD: Use Rc for shared ownership
pub fn e1602_good_rc() {
    use std::rc::Rc;

    let data = Rc::new(vec![1, 2, 3]);
    let data2 = Rc::clone(&data);
    drop(data);
    println!("{:?}", data2); // Still alive
}

/// GOOD: Return owned data instead of pointer
pub fn e1602_good_return_owned() -> i32 {

    42 // Return the value, not a pointer
}

/// GOOD: Use Box for heap allocation with clear ownership
pub fn e1602_good_box() -> Box<i32> {
    Box::new(42) // Caller owns the Box
}

// ============================================================================
// GOOD EXAMPLES unit tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_reference() {
        let data = vec![1, 2, 3];
        assert_eq!(e1602_good_reference(&data), Some(&1));
    }

    #[test]
    fn test_return_owned() {
        assert_eq!(e1602_good_return_owned(), 42);
    }

    #[test]
    fn test_box() {
        let b = e1602_good_box();
        assert_eq!(*b, 42);
    }
}
