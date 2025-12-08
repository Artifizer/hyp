/// E1408: Unchecked array indexing
/// Severity: HIGH
/// LLM confusion: 2 (LOW)
///
/// Description: Accessing an array element without checking if the index is valid will crash the
/// program if the index is out of bounds. While this is memory-safe (won't corrupt memory), it will
/// panic and crash. It's like trying to access the 100th element of a 10-element array - it crashes
/// instead of returning garbage. Fix by using .get() which returns None for invalid indices, or
/// validate indices before use.
///
/// ## The Panic Problem
///
/// ```text
/// let arr = [1, 2, 3];
/// let x = arr[5];  // PANIC! Index 5 is out of bounds
/// ```
///
/// ## Why This Matters
///
/// 1. **Program crash**: Panic terminates the program
/// 2. **User data loss**: Unsaved work is lost
/// 3. **No recovery**: Unlike Option/Result, panics are hard to recover from
/// 4. **Silent in happy path**: Only crashes with certain inputs
///
/// ## The Right Solutions
///
/// ### Option 1: Use .get() for safe access
/// ```rust
/// fn safe_get(arr: &[i32], idx: usize) -> Option<i32> {
///     arr.get(idx).copied()
/// }
/// ```
///
/// ### Option 2: Validate index before access
/// ```rust
/// fn checked_access(arr: &[i32], idx: usize) -> Result<i32, &'static str> {
///     if idx < arr.len() {
///         Ok(arr[idx])
///     } else {
///         Err("Index out of bounds")
///     }
/// }
/// ```
///
/// ### Option 3: Use iterators instead of indexing
/// ```rust
/// fn sum_first_n(arr: &[i32], n: usize) -> i32 {
///     arr.iter().take(n).sum()  // Never panics
/// }
/// ```
///
/// Mitigation: Use `.get(idx)` which returns `Option<&T>` instead of panicking. Validate indices
/// before use. Use iterators instead of manual indexing when possible. Consider using
/// `get_unchecked()` only in performance-critical code with proven bounds.

// ============================================================================
// DANGEROUS PATTERNS - NEVER DO THIS
// ============================================================================

/// PROBLEM E1408: Can panic if idx is out of bounds
pub fn e1408_bad_indexing(data: &[i32], idx: usize) -> i32 {
    // PROBLEM E1408: Can panic if idx is out of bounds
    data[idx]
}

/// PROBLEM E1408: Index from user input
pub fn e1408_bad_user_index(data: &[i32], user_idx: usize) -> i32 {
    data[user_idx] // User can crash the program!
}

/// PROBLEM E1408: Loop with potential overflow
pub fn e1408_bad_loop_index(data: &[i32], start: usize, count: usize) -> i32 {
    let mut sum = 0;
    for i in 0..count {
        sum += data[start + i]; // Can panic if start + i >= len
    }
    sum
}

/// Entry point for problem demonstration
pub fn e1408_entry() -> Result<(), Box<dyn std::error::Error>> {
    Ok(())
}

// ============================================================================
// GOOD ALTERNATIVES
// ============================================================================

/// GOOD: Use .get() for safe access
pub fn e1408_good_get(data: &[i32], idx: usize) -> Option<i32> {
    data.get(idx).copied()
}

/// GOOD: Validate index before access
pub fn e1408_good_validate(data: &[i32], idx: usize) -> Result<i32, &'static str> {
    if idx < data.len() {
        Ok(data[idx])
    } else {
        Err("Index out of bounds")
    }
}

/// GOOD: Use .get() with default value
pub fn e1408_good_get_or_default(data: &[i32], idx: usize, default: i32) -> i32 {
    data.get(idx).copied().unwrap_or(default)
}

/// GOOD: Use iterators instead of indexing
pub fn e1408_good_iterator(data: &[i32], start: usize, count: usize) -> i32 {
    data.iter().skip(start).take(count).sum()
}

/// GOOD: Safe range access with slicing
pub fn e1408_good_slice(data: &[i32], start: usize, end: usize) -> Option<&[i32]> {
    data.get(start..end)
}

/// GOOD: First and last with Option
pub fn e1408_good_first_last(data: &[i32]) -> (Option<i32>, Option<i32>) {
    (data.first().copied(), data.last().copied())
}

/// GOOD: Safe mutable access
pub fn e1408_good_get_mut(data: &mut [i32], idx: usize, new_value: i32) -> bool {
    if let Some(elem) = data.get_mut(idx) {
        *elem = new_value;
        true
    } else {
        false
    }
}

/// GOOD: Use split_at safely
pub fn e1408_good_split(data: &[i32], mid: usize) -> Option<(&[i32], &[i32])> {
    if mid <= data.len() {
        Some(data.split_at(mid))
    } else {
        None
    }
}

/// GOOD: Checked indexing with assertion for internal use
pub fn e1408_good_debug_checked(data: &[i32], idx: usize) -> i32 {
    debug_assert!(idx < data.len(), "Index {} out of bounds for len {}", idx, data.len());
    // In release, this is unchecked for performance
    // Only use when you've proven the bounds elsewhere
    data[idx]
}

// ============================================================================
// GOOD EXAMPLES unit tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_valid() {
        let data = [1, 2, 3];
        assert_eq!(e1408_good_get(&data, 1), Some(2));
    }

    #[test]
    fn test_get_invalid() {
        let data = [1, 2, 3];
        assert_eq!(e1408_good_get(&data, 10), None);
    }

    #[test]
    fn test_validate_valid() {
        let data = [1, 2, 3];
        assert_eq!(e1408_good_validate(&data, 1), Ok(2));
    }

    #[test]
    fn test_validate_invalid() {
        let data = [1, 2, 3];
        assert!(e1408_good_validate(&data, 10).is_err());
    }

    #[test]
    fn test_get_or_default() {
        let data = [1, 2, 3];
        assert_eq!(e1408_good_get_or_default(&data, 1, 0), 2);
        assert_eq!(e1408_good_get_or_default(&data, 10, 0), 0);
    }

    #[test]
    fn test_iterator_safe() {
        let data = [1, 2, 3, 4, 5];
        // Even with out-of-bounds parameters, doesn't panic
        assert_eq!(e1408_good_iterator(&data, 10, 5), 0);
        assert_eq!(e1408_good_iterator(&data, 1, 3), 9); // 2 + 3 + 4
    }

    #[test]
    fn test_slice() {
        let data = [1, 2, 3, 4, 5];
        assert_eq!(e1408_good_slice(&data, 1, 4), Some(&[2, 3, 4][..]));
        assert_eq!(e1408_good_slice(&data, 1, 10), None);
    }

    #[test]
    fn test_first_last() {
        assert_eq!(e1408_good_first_last(&[1, 2, 3]), (Some(1), Some(3)));
        assert_eq!(e1408_good_first_last(&[]), (None, None));
    }

    #[test]
    fn test_get_mut() {
        let mut data = [1, 2, 3];
        assert!(e1408_good_get_mut(&mut data, 1, 10));
        assert_eq!(data[1], 10);
        assert!(!e1408_good_get_mut(&mut data, 10, 20));
    }
}
