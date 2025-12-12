/// E1707: Unbounded recursion
/// Severity: HIGH
/// LLM confusion: 3 (MED)
///
/// Description: Unbounded recursion is recursion without a proper base case for all inputs. This
/// function has a base case for n=0, but if called with a negative number, it recurses forever
/// (n-1 makes negative numbers more negative). This will overflow the stack and crash. Fix by
/// adding proper base cases for all possible inputs.
///
/// ## The Missing Base Case Problem
///
/// ```text
/// fn count_down(n: i32) -> i32 {
///     if n == 0 { 0 }
///     else { 1 + count_down(n - 1) }
/// }
///
/// count_down(-1);  // -1 → -2 → -3 → ... → stack overflow!
/// ```
///
/// ## Why This Matters
///
/// 1. **Stack overflow**: Infinite recursion crashes
/// 2. **Hard to debug**: May work for some inputs, crash for others
/// 3. **Security risk**: Malicious input can crash server
/// 4. **Resource exhaustion**: Consumes all stack space
///
/// ## The Right Solutions
///
/// ### Option 1: Handle all cases
/// ```rust
/// fn count_down(n: i32) -> i32 {
///     if n <= 0 { 0 }  // Handle negative AND zero
///     else { 1 + count_down(n - 1) }
/// }
/// ```
///
/// ### Option 2: Use unsigned types
/// ```rust
/// fn count_down(n: u32) -> u32 {
///     if n == 0 { 0 }
///     else { 1 + count_down(n - 1) }
/// }
/// // Can't be negative by construction
/// ```
///
/// ### Option 3: Add depth limit
/// ```rust
/// fn recurse(n: i32, depth: usize) -> Result<i32, &'static str> {
///     if depth > 1000 {
///         return Err("Recursion too deep");
///     }
///     // ... recursive call with depth + 1
/// }
/// ```
///
/// Mitigation: Ensure all recursive functions have base cases covering all inputs. Use unsigned
/// types (u32) when values should never be negative. Add assertions or validation at function
/// entry. Convert to iteration when recursion depth is unbounded.

// ============================================================================
// DANGEROUS PATTERNS
// ============================================================================

/// PROBLEM E1707: No base case for negative numbers
pub fn e1707_bad_no_negative_case(n: i32) -> i32 {
    // PROBLEM E1707: No base case for negative numbers
    if n == 0 {
        0
    } else {
        1 + e1707_bad_no_negative_case(n - 1)
    }
}

/// PROBLEM E1707: Incorrect base case
pub fn e1707_bad_wrong_base(n: i32) -> i32 {
    if n == 1 {
        // What if n starts at 0 or negative?
        1
    } else {
        n * e1707_bad_wrong_base(n - 1)
    }
}

/// PROBLEM E1707: No termination for some paths
pub fn e1707_bad_conditional_recurse(n: i32, flag: bool) -> i32 {
    if n == 0 && flag {
        0
    } else {
        // If flag is false, this never terminates!
        1 + e1707_bad_conditional_recurse(n - 1, flag)
    }
}

/// Entry point for problem demonstration
pub fn e1707_entry() -> Result<(), Box<dyn std::error::Error>> {
    Ok(())
}

// ============================================================================
// GOOD ALTERNATIVES
// ============================================================================

/// GOOD: Handle all cases including negative
pub fn e1707_good_all_cases(n: i32) -> i32 {
    if n <= 0 {
        0
    } else {
        1 + e1707_good_all_cases(n - 1)
    }
}

/// GOOD: Use unsigned type
pub fn e1707_good_unsigned(n: u32) -> u32 {
    if n == 0 {
        0
    } else {
        1 + e1707_good_unsigned(n - 1)
    }
}

/// GOOD: Add depth limit
pub fn e1707_good_depth_limit(n: i32) -> Result<i32, &'static str> {
    fn helper(n: i32, depth: usize) -> Result<i32, &'static str> {
        if depth > 1000 {
            return Err("Recursion too deep");
        }
        if n <= 0 {
            Ok(0)
        } else {
            Ok(1 + helper(n - 1, depth + 1)?)
        }
    }
    helper(n, 0)
}

/// GOOD: Validate input at entry
pub fn e1707_good_validate(n: i32) -> Option<i32> {
    if n < 0 {
        return None;
    }

    fn helper(n: i32) -> i32 {
        if n == 0 {
            0
        } else {
            1 + helper(n - 1)
        }
    }

    Some(helper(n))
}

/// GOOD: Convert to iteration
pub fn e1707_good_iterative(n: i32) -> i32 {
    if n <= 0 {
        return 0;
    }
    n // Just return n for counting down to 0
}

/// GOOD: Use saturating arithmetic
pub fn e1707_good_saturating(n: i32) -> i32 {
    if n <= 0 {
        0
    } else {
        1 + e1707_good_saturating(n.saturating_sub(1))
    }
}

/// GOOD: Explicit termination check
pub fn e1707_good_explicit_check(n: i32) -> Result<i32, &'static str> {
    if n < 0 {
        return Err("Negative input not allowed");
    }
    if n > 10000 {
        return Err("Input too large");
    }

    fn helper(n: i32) -> i32 {
        if n == 0 {
            0
        } else {
            1 + helper(n - 1)
        }
    }

    Ok(helper(n))
}

// ============================================================================
// GOOD EXAMPLES unit tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_all_cases() {
        assert_eq!(e1707_good_all_cases(5), 5);
        assert_eq!(e1707_good_all_cases(0), 0);
        assert_eq!(e1707_good_all_cases(-5), 0);
    }

    #[test]
    fn test_unsigned() {
        assert_eq!(e1707_good_unsigned(5), 5);
        assert_eq!(e1707_good_unsigned(0), 0);
    }

    #[test]
    fn test_depth_limit() {
        assert_eq!(e1707_good_depth_limit(100), Ok(100));
        // Would error for very deep recursion
    }

    #[test]
    fn test_validate() {
        assert_eq!(e1707_good_validate(5), Some(5));
        assert_eq!(e1707_good_validate(-5), None);
    }

    #[test]
    fn test_explicit_check() {
        assert_eq!(e1707_good_explicit_check(5), Ok(5));
        assert!(e1707_good_explicit_check(-5).is_err());
        assert!(e1707_good_explicit_check(100000).is_err());
    }
}
