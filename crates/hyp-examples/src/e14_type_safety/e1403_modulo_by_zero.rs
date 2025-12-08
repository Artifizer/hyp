/// E1403: Modulo by zero
/// Severity: HIGH
/// LLM confusion: 2 (LOW)
///
/// Description: The modulo operation (remainder after division) also crashes when the divisor is
/// zero, just like regular division. This is the same problem as division by zero but for the
/// remainder operation. It's like asking 'what's the remainder when dividing by nothing?' - it
/// doesn't make sense and crashes. Fix by checking for zero before the modulo operation, or using
/// checked_rem() which returns None for zero divisors.
///
/// ## The Crash Problem
///
/// ```text
/// fn wrap_index(index: usize, size: usize) -> usize {
///     index % size  // If size is 0 → PANIC!
/// }
///
/// wrap_index(5, 0);  // Program crashes immediately
/// ```
///
/// ## Why This Matters
///
/// 1. **Immediate crash**: Same severity as division by zero
/// 2. **Common in indexing**: Wrapping indices is a common pattern
/// 3. **Hash tables**: Modulo is used in hash bucket selection
/// 4. **Cyclic operations**: Round-robin, circular buffers, etc.
///
/// ## The Right Solutions
///
/// ### Option 1: Use checked_rem
/// ```rust
/// fn safe_modulo(x: i32, y: i32) -> Option<i32> {
///     x.checked_rem(y)  // Returns None for modulo by zero
/// }
/// ```
///
/// ### Option 2: Validate before modulo
/// ```rust
/// fn modulo(x: i32, y: i32) -> Result<i32, &'static str> {
///     if y == 0 {
///         Err("Modulo by zero")
///     } else {
///         Ok(x % y)
///     }
/// }
/// ```
///
/// ### Option 3: Use NonZero types
/// ```rust
/// use std::num::NonZeroUsize;
///
/// fn wrap_index(index: usize, size: NonZeroUsize) -> usize {
///     index % size.get()  // Cannot be zero by construction
/// }
/// ```
///
/// Mitigation: Use `checked_rem()` which returns `None` for modulo by zero. Add validation to
/// ensure divisors are non-zero. Consider using `rem_euclid()` for consistent behavior with
/// negative numbers.

// ============================================================================
// DANGEROUS PATTERNS - NEVER DO THIS
// ============================================================================

/// PROBLEM E1403: No check for zero divisor
pub fn e1403_bad_modulo(x: i32, y: i32) -> i32 {
    // PROBLEM E1403: No check for zero divisor
    x % y
}

/// PROBLEM E1403: Wrap index without size check
pub fn e1403_bad_wrap_index(index: usize, size: usize) -> usize {
    index % size // Crashes if size is 0!
}

/// PROBLEM E1403: Cyclic selection
pub fn e1403_bad_round_robin<'a>(items: &'a [&'a str], iteration: usize) -> &'a str {
    items[iteration % items.len()] // Crashes if items is empty!
}

/// Entry point for problem demonstration
pub fn e1403_entry() -> Result<(), Box<dyn std::error::Error>> {
    Ok(())
}

// ============================================================================
// GOOD ALTERNATIVES
// ============================================================================

/// GOOD: Use checked_rem
pub fn e1403_good_checked_rem(x: i32, y: i32) -> Option<i32> {
    x.checked_rem(y)
}

/// GOOD: Validate before modulo
pub fn e1403_good_validate(x: i32, y: i32) -> Result<i32, &'static str> {
    if y == 0 {
        Err("Modulo by zero")
    } else {
        Ok(x % y)
    }
}

/// GOOD: Use NonZero type
pub fn e1403_good_nonzero(x: usize, y: std::num::NonZeroUsize) -> usize {
    x % y.get()
}

/// GOOD: Safe wrap index
pub fn e1403_good_wrap_index(index: usize, size: usize) -> Option<usize> {
    if size == 0 {
        None
    } else {
        Some(index % size)
    }
}

/// GOOD: Safe round robin with Result
pub fn e1403_good_round_robin<'a>(items: &'a [&str], iteration: usize) -> Option<&'a str> {
    if items.is_empty() {
        None
    } else {
        Some(items[iteration % items.len()])
    }
}

/// GOOD: Use rem_euclid for consistent behavior with negatives
pub fn e1403_good_rem_euclid(x: i32, y: i32) -> Option<i32> {
    if y == 0 {
        None
    } else {
        Some(x.rem_euclid(y)) // Always returns non-negative result
    }
}

/// GOOD: Return default on modulo by zero
pub fn e1403_good_with_default(x: i32, y: i32, default: i32) -> i32 {
    x.checked_rem(y).unwrap_or(default)
}

// ============================================================================
// GOOD EXAMPLES unit tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use std::num::NonZeroUsize;

    #[test]
    fn test_checked_rem_success() {
        assert_eq!(e1403_good_checked_rem(10, 3), Some(1));
    }

    #[test]
    fn test_checked_rem_zero() {
        assert_eq!(e1403_good_checked_rem(10, 0), None);
    }

    #[test]
    fn test_validate_success() {
        assert_eq!(e1403_good_validate(10, 3), Ok(1));
    }

    #[test]
    fn test_validate_zero() {
        assert_eq!(e1403_good_validate(10, 0), Err("Modulo by zero"));
    }

    #[test]
    fn test_nonzero() {
        let y = NonZeroUsize::new(3).unwrap();
        assert_eq!(e1403_good_nonzero(10, y), 1);
    }

    #[test]
    fn test_wrap_index_empty() {
        assert_eq!(e1403_good_wrap_index(5, 0), None);
    }

    #[test]
    fn test_wrap_index_valid() {
        assert_eq!(e1403_good_wrap_index(7, 3), Some(1));
    }

    #[test]
    fn test_round_robin_empty() {
        let items: &[&str] = &[];
        assert_eq!(e1403_good_round_robin(items, 0), None);
    }

    #[test]
    fn test_round_robin_valid() {
        let items = &["a", "b", "c"];
        assert_eq!(e1403_good_round_robin(items, 4), Some("b")); // 4 % 3 = 1
    }

    #[test]
    fn test_rem_euclid_negative() {
        // rem_euclid always returns non-negative
        assert_eq!(e1403_good_rem_euclid(-7, 4), Some(1)); // Not -3!
        assert_eq!(e1403_good_rem_euclid(7, 4), Some(3));
    }
}
