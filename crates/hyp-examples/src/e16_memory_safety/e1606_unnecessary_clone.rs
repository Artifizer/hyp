/// E1606: Unnecessary clone
/// Severity: LOW
/// LLM confusion: 2 (LOW)
///
/// Description: Cloning large data structures is expensive because it copies all the data.
/// This function clones a vector when it could just return a reference or restructure to avoid
/// the clone. Unnecessary clones waste memory and CPU time. Fix by using references, borrowing,
/// or restructuring ownership to avoid clones.
///
/// ## The Clone Cost
///
/// ```text
/// let big_vec = vec![0u8; 1_000_000];  // 1MB
/// let copy = big_vec.clone();           // Another 1MB allocated and copied!
/// ```
///
/// ## Why This Matters
///
/// 1. **Memory usage**: Doubles memory for cloned data
/// 2. **CPU time**: Copying takes time proportional to size
/// 3. **Allocation pressure**: More allocations = more GC-like overhead
/// 4. **Cache pollution**: Large copies evict useful data from cache
///
/// ## The Right Solutions
///
/// ### Option 1: Return reference instead of clone
/// ```rust
/// fn get_data(data: &Vec<i32>) -> &[i32] {
///     data.as_slice()  // No copy
/// }
/// ```
///
/// ### Option 2: Use Cow for conditional cloning
/// ```rust
/// use std::borrow::Cow;
///
/// fn maybe_modify(data: &[i32], modify: bool) -> Cow<[i32]> {
///     if modify {
///         Cow::Owned(data.iter().map(|x| x * 2).collect())
///     } else {
///         Cow::Borrowed(data)  // No clone!
///     }
/// }
/// ```
///
/// ### Option 3: Take ownership when needed
/// ```rust
/// fn consume(data: Vec<i32>) -> Vec<i32> {
///     data  // Take ownership, no clone
/// }
/// ```
///
/// Mitigation: Use `#![warn(clippy::clone_on_copy)]` and `#![warn(clippy::unnecessary_clone)]`
/// to detect unnecessary clones. Return references instead of clones when possible. Use `Cow`
/// (Clone on Write) for conditional cloning. Profile to identify expensive clones.

// ============================================================================
// WASTEFUL PATTERNS
// ============================================================================

/// PROBLEM E1606: Unnecessary clone, could return reference
#[allow(clippy::ptr_arg)]
pub fn e1606_bad_clone(data: &Vec<i32>) -> Vec<i32> {
    // PROBLEM E1606: Unnecessary clone, could return reference
    data.clone()
}

/// PROBLEM E1606: Clone in loop
pub fn e1606_bad_clone_in_loop(data: &[String]) -> Vec<String> {
    let mut result = Vec::new();
    for item in data {
        result.push(item.clone()); // Clone on each iteration
    }
    result
}

/// PROBLEM E1606: Clone when moving would work
pub fn e1606_bad_clone_instead_of_move(data: Vec<i32>) -> Vec<i32> {
     // Unnecessary - could just use data
    data.clone()
}

/// Entry point for problem demonstration
pub fn e1606_entry() -> Result<(), Box<dyn std::error::Error>> {
    Ok(())
}

// ============================================================================
// GOOD ALTERNATIVES
// ============================================================================

/// GOOD: Return reference instead of clone
pub fn e1606_good_reference(data: &[i32]) -> &[i32] {
    data // No copy
}

/// GOOD: Use Cow for conditional cloning
pub fn e1606_good_cow(data: &[i32], should_modify: bool) -> std::borrow::Cow<'_, [i32]> {
    use std::borrow::Cow;

    if should_modify {
        Cow::Owned(data.iter().map(|x| x * 2).collect())
    } else {
        Cow::Borrowed(data) // No clone!
    }
}

/// GOOD: Take ownership instead of cloning
pub fn e1606_good_take_ownership(data: Vec<i32>) -> Vec<i32> {
    data // Just move it
}

/// GOOD: Use iterators to avoid intermediate collections
pub fn e1606_good_iterator(data: &[i32]) -> i32 {
    data.iter().sum() // No clone needed
}

/// GOOD: Clone only what's needed
pub fn e1606_good_partial_clone(data: &[i32]) -> Vec<i32> {
    data.iter().filter(|&&x| x > 0).copied().collect()
}

/// GOOD: Use Arc for shared ownership
pub fn e1606_good_arc(data: std::sync::Arc<Vec<i32>>) -> std::sync::Arc<Vec<i32>> {
    data // Just clone the Arc (cheap), not the Vec
}

/// GOOD: Use into_iter to consume and transform
pub fn e1606_good_into_iter(data: Vec<String>) -> Vec<String> {
    data.into_iter()
        .map(|s| s.to_uppercase())
        .collect()
}

/// GOOD: Borrow checker guides correct usage
pub fn e1606_good_borrow_check(data: &mut Vec<i32>) {
    // Modify in place instead of clone + modify
    for item in data.iter_mut() {
        *item *= 2;
    }
}

// ============================================================================
// GOOD EXAMPLES unit tests
// ============================================================================


#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    #[test]
    fn test_reference() {
        let data = vec![1, 2, 3];
        let result = e1606_good_reference(&data);
        assert_eq!(result, &[1, 2, 3]);
    }

    #[test]
    fn test_cow_borrowed() {
        let data = vec![1, 2, 3];
        let result = e1606_good_cow(&data, false);
        assert!(matches!(result, std::borrow::Cow::Borrowed(_)));
    }

    #[test]
    fn test_cow_owned() {
        let data = vec![1, 2, 3];
        let result = e1606_good_cow(&data, true);
        assert!(matches!(result, std::borrow::Cow::Owned(_)));
        assert_eq!(&*result, &[2, 4, 6]);
    }

    #[test]
    fn test_arc() {
        let data = Arc::new(vec![1, 2, 3]);
        let data2 = e1606_good_arc(data.clone());
        assert!(Arc::ptr_eq(&data, &data2));
    }

    #[test]
    fn test_into_iter() {
        let data = vec!["hello".to_string(), "world".to_string()];
        let result = e1606_good_into_iter(data);
        assert_eq!(result, vec!["HELLO", "WORLD"]);
    }
}
