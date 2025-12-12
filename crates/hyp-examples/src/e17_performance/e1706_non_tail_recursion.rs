/// E1706: Non-tail recursion
/// Severity: MED
/// LLM confusion: 3 (MED)
///
/// Description: Tail recursion is when a function's last operation is calling itself. Some
/// languages optimize this to avoid stack growth, but Rust doesn't guarantee tail call
/// optimization. This recursive factorial is NOT tail-recursive because it multiplies after
/// the recursive call. Fix by converting to iteration or using an accumulator for tail recursion.
///
/// ## The Stack Growth Problem
///
/// ```text
/// fn factorial(n: u64) -> u64 {
///     if n == 0 { 1 }
///     else { n * factorial(n - 1) }  // Multiply AFTER recursive call
/// }
///
/// factorial(10000);  // Stack overflow!
/// // Each call adds a stack frame: n=10000, n=9999, n=9998, ...
/// ```
///
/// ## Why This Matters
///
/// 1. **Stack overflow**: Deep recursion crashes
/// 2. **No TCO guarantee**: Rust doesn't optimize tail calls
/// 3. **Memory usage**: Each frame uses stack space
/// 4. **Performance**: Function call overhead per level
///
/// ## The Right Solutions
///
/// ### Option 1: Convert to iteration
/// ```rust
/// fn factorial(n: u64) -> u64 {
///     (1..=n).product()
/// }
/// ```
///
/// ### Option 2: Use accumulator (tail-recursive style)
/// ```rust
/// fn factorial(n: u64) -> u64 {
///     fn helper(n: u64, acc: u64) -> u64 {
///         if n == 0 { acc }
///         else { helper(n - 1, n * acc) }  // Tail call
///     }
///     helper(n, 1)
/// }
/// ```
///
/// ### Option 3: Use explicit stack
/// ```rust
/// fn factorial(mut n: u64) -> u64 {
///     let mut result = 1;
///     while n > 0 {
///         result *= n;
///         n -= 1;
///     }
///     result
/// }
/// ```
///
/// Mitigation: Convert recursive functions to iterative loops when possible. Use an accumulator
/// parameter to make recursion tail-recursive. Be aware that Rust doesn't guarantee TCO (tail
/// call optimization). For deep recursion, use iteration or trampolining.

// ============================================================================
// PROBLEMATIC PATTERNS
// ============================================================================

/// PROBLEM E1706: Not tail-recursive, can overflow stack
pub fn e1706_bad_factorial(n: u64) -> u64 {
    // PROBLEM E1706: Not tail-recursive, can overflow stack
    if n == 0 {
        1
    } else {
        n * e1706_bad_factorial(n - 1)
    }
}

/// PROBLEM E1706: Non-tail recursive Fibonacci
pub fn e1706_bad_fibonacci(n: u64) -> u64 {
    if n <= 1 {
        n
    } else {
        e1706_bad_fibonacci(n - 1) + e1706_bad_fibonacci(n - 2)
    }
}

/// PROBLEM E1706: Non-tail recursive sum
pub fn e1706_bad_sum(list: &[i32]) -> i32 {
    if list.is_empty() {
        0
    } else {
        list[0] + e1706_bad_sum(&list[1..])
    }
}

/// Entry point for problem demonstration
pub fn e1706_entry() -> Result<(), Box<dyn std::error::Error>> {
    Ok(())
}

// ============================================================================
// GOOD ALTERNATIVES
// ============================================================================

/// GOOD: Iterative factorial
pub fn e1706_good_factorial_iter(n: u64) -> u64 {
    (1..=n).product()
}

/// GOOD: Tail-recursive factorial with accumulator
pub fn e1706_good_factorial_tail(n: u64) -> u64 {
    fn helper(n: u64, acc: u64) -> u64 {
        if n == 0 {
            acc
        } else {
            helper(n - 1, n * acc)
        }
    }
    helper(n, 1)
}

/// GOOD: Iterative Fibonacci
pub fn e1706_good_fibonacci_iter(n: u64) -> u64 {
    if n <= 1 {
        return n;
    }

    let mut a = 0u64;
    let mut b = 1u64;

    for _ in 2..=n {
        let temp = a + b;
        a = b;
        b = temp;
    }

    b
}

/// GOOD: Memoized Fibonacci
pub fn e1706_good_fibonacci_memo(n: u64) -> u64 {
    fn helper(n: u64, memo: &mut std::collections::HashMap<u64, u64>) -> u64 {
        if n <= 1 {
            return n;
        }
        if let Some(&result) = memo.get(&n) {
            return result;
        }
        let result = helper(n - 1, memo) + helper(n - 2, memo);
        memo.insert(n, result);
        result
    }

    let mut memo = std::collections::HashMap::new();
    helper(n, &mut memo)
}

/// GOOD: Iterative sum
pub fn e1706_good_sum_iter(list: &[i32]) -> i32 {
    list.iter().sum()
}

/// GOOD: Tail-recursive sum with accumulator
pub fn e1706_good_sum_tail(list: &[i32]) -> i32 {
    fn helper(list: &[i32], acc: i32) -> i32 {
        if list.is_empty() {
            acc
        } else {
            helper(&list[1..], acc + list[0])
        }
    }
    helper(list, 0)
}

/// GOOD: Use fold for accumulation
pub fn e1706_good_sum_fold(list: &[i32]) -> i32 {
    list.iter().sum::<i32>()
}

// ============================================================================
// GOOD EXAMPLES unit tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_factorial_iter() {
        assert_eq!(e1706_good_factorial_iter(5), 120);
        assert_eq!(e1706_good_factorial_iter(0), 1);
    }

    #[test]
    fn test_factorial_tail() {
        assert_eq!(e1706_good_factorial_tail(5), 120);
        assert_eq!(e1706_good_factorial_tail(0), 1);
    }

    #[test]
    fn test_fibonacci_iter() {
        assert_eq!(e1706_good_fibonacci_iter(10), 55);
        assert_eq!(e1706_good_fibonacci_iter(0), 0);
        assert_eq!(e1706_good_fibonacci_iter(1), 1);
    }

    #[test]
    fn test_fibonacci_memo() {
        assert_eq!(e1706_good_fibonacci_memo(10), 55);
    }

    #[test]
    fn test_sum_iter() {
        assert_eq!(e1706_good_sum_iter(&[1, 2, 3, 4, 5]), 15);
    }

    #[test]
    fn test_sum_tail() {
        assert_eq!(e1706_good_sum_tail(&[1, 2, 3, 4, 5]), 15);
    }

    #[test]
    fn test_sum_fold() {
        assert_eq!(e1706_good_sum_fold(&[1, 2, 3, 4, 5]), 15);
    }
}
