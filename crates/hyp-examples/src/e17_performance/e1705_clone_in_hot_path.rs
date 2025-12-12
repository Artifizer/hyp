/// E1705: Clone in hot path
/// Severity: LOW
/// LLM confusion: 2 (LOW)
///
/// Description: Cloning large data structures in performance-critical code (hot paths) is
/// expensive. This code clones a Vec 1000 times in a loop, copying all the data each time. Fix
/// by using references instead of clones, or restructure the algorithm to avoid repeated cloning.
///
/// ## The Hot Path Problem
///
/// ```text
/// for _ in 0..1000 {
///     let copy = big_data.clone();  // 1000 clones!
///     process(copy);
/// }
/// // If big_data is 1MB, this copies 1GB total
/// ```
///
/// ## Why This Matters
///
/// 1. **Multiplied cost**: Clone cost × iteration count
/// 2. **Memory bandwidth**: Saturates memory bus
/// 3. **Cache thrashing**: Evicts useful data from cache
/// 4. **Allocation pressure**: Many allocations in tight loop
///
/// ## The Right Solutions
///
/// ### Option 1: Use references
/// ```rust
/// for _ in 0..1000 {
///     process(&big_data);  // No clone
/// }
/// ```
///
/// ### Option 2: Clone once outside loop
/// ```rust
/// let working_copy = big_data.clone();  // Clone once
/// for _ in 0..1000 {
///     process(&working_copy);
/// }
/// ```
///
/// ### Option 3: Use Cow for conditional cloning
/// ```rust
/// use std::borrow::Cow;
///
/// for _ in 0..1000 {
///     let data: Cow<[i32]> = if needs_modification {
///         Cow::Owned(big_data.clone())
///     } else {
///         Cow::Borrowed(&big_data)
///     };
/// }
/// ```
///
/// Mitigation: Use `#![warn(clippy::clone_on_ref_ptr)]` to catch suspicious clones. Profile to
/// identify expensive clones. Use references or `Cow` (Clone on Write) to avoid clones. Consider
/// if you really need owned data or if borrowing would work.

// ============================================================================
// WASTEFUL PATTERNS
// ============================================================================

/// PROBLEM E1705: Cloning large data structure repeatedly
#[allow(clippy::ptr_arg)]
pub fn e1705_bad_clone_in_loop(data: &Vec<i32>) -> Vec<i32> {
    let mut result = Vec::new();
    for _ in 0..1000 {
        // PROBLEM E1705: Cloning large data structure repeatedly
        let copy = data.clone();
        result.extend(copy);
    }
    result
}

/// PROBLEM E1705: Clone in inner loop
pub fn e1705_bad_nested_clone(outer: &[Vec<i32>]) -> i32 {
    let mut sum = 0;
    for vec in outer {
        for _ in 0..100 {
            let copy = vec.clone(); // Clone in inner loop!
            sum += copy.iter().sum::<i32>();
        }
    }
    sum
}

/// Entry point for problem demonstration
pub fn e1705_entry() -> Result<(), Box<dyn std::error::Error>> {
    Ok(())
}

// ============================================================================
// GOOD ALTERNATIVES
// ============================================================================

/// GOOD: Use reference instead of clone
pub fn e1705_good_reference(data: &[i32]) -> Vec<i32> {
    let mut result = Vec::with_capacity(data.len() * 1000);
    for _ in 0..1000 {
        result.extend_from_slice(data); // No clone, just copy elements
    }
    result
}

/// GOOD: Clone once outside loop
pub fn e1705_good_clone_once(data: &[i32]) -> i32 {
    let working_copy = data.to_vec(); // Clone once
    let mut sum = 0;
    for _ in 0..1000 {
        sum += working_copy.iter().sum::<i32>(); // Use reference
    }
    sum
}

/// GOOD: Use Cow for conditional cloning
pub fn e1705_good_cow(data: &[i32], should_modify: bool) -> i32 {
    use std::borrow::Cow;

    let mut sum = 0;
    for i in 0..1000 {
        let cow: Cow<[i32]> = if should_modify && i % 100 == 0 {
            // Only clone when we need to modify
            let mut v = data.to_vec();
            v[0] += 1;
            Cow::Owned(v)
        } else {
            Cow::Borrowed(data) // No clone!
        };
        sum += cow.iter().sum::<i32>();
    }
    sum
}

/// GOOD: Use Arc for shared ownership
pub fn e1705_good_arc(data: std::sync::Arc<Vec<i32>>) -> i32 {
    let mut sum = 0;
    for _ in 0..1000 {
        let shared = std::sync::Arc::clone(&data); // Cheap clone (atomic increment)
        sum += shared.iter().sum::<i32>();
    }
    sum
}

/// GOOD: Restructure to avoid clone need
pub fn e1705_good_restructure(data: &[i32]) -> i32 {
    // Instead of cloning 1000 times, compute directly
    data.iter().sum::<i32>() * 1000
}

/// GOOD: Use indices instead of cloning
pub fn e1705_good_indices(data: &[Vec<i32>]) -> i32 {
    let mut sum = 0;
    for vec in data {
        for _ in 0..100 {
            sum += vec.iter().sum::<i32>(); // Reference, no clone
        }
    }
    sum
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
        let result = e1705_good_reference(&data);
        assert_eq!(result.len(), 3000);
    }

    #[test]
    fn test_clone_once() {
        let data = vec![1, 2, 3];
        let result = e1705_good_clone_once(&data);
        assert_eq!(result, 6000); // 6 * 1000
    }

    #[test]
    fn test_cow() {
        let data = vec![1, 2, 3];
        let result = e1705_good_cow(&data, false);
        assert_eq!(result, 6000);
    }

    #[test]
    fn test_arc() {
        let data = Arc::new(vec![1, 2, 3]);
        let result = e1705_good_arc(data);
        assert_eq!(result, 6000);
    }

    #[test]
    fn test_restructure() {
        let data = vec![1, 2, 3];
        let result = e1705_good_restructure(&data);
        assert_eq!(result, 6000);
    }
}
