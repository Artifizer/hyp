/// E1405: Integer division rounding errors
/// Severity: LOW
/// LLM confusion: 2 (LOW)
///
/// Description: Integer division truncates (cuts off) the decimal part, which can cause precision
/// loss. For example, 7 divided by 2 gives 3 (not 3.5), losing the 0.5 remainder. This might not
/// be what you want for calculations needing precision. It's like a calculator that only shows
/// whole numbers. Fix by using floating-point types when precision matters, or document that
/// truncation is intentional.
///
/// ## The Truncation Problem
///
/// ```text
/// let average = (1 + 2 + 3) / 3;  // = 2, not 2.0
/// let half = 5 / 2;               // = 2, not 2.5
/// let percentage = 1 / 3 * 100;   // = 0! (1/3 = 0 first)
/// ```
///
/// ## Why This Matters
///
/// 1. **Lost precision**: Fractional parts are discarded
/// 2. **Order matters**: (1/3)*100 ≠ (1*100)/3 in integer math
/// 3. **Accumulating errors**: Multiple divisions compound the loss
/// 4. **Unexpected zeros**: Small/large divisions can produce 0
///
/// ## The Right Solutions
///
/// ### Option 1: Use floating-point when precision matters
/// ```rust
/// fn average(values: &[i32]) -> f64 {
///     values.iter().sum::<i32>() as f64 / values.len() as f64
/// }
/// ```
///
/// ### Option 2: Multiply before dividing
/// ```rust
/// fn percentage(part: i32, total: i32) -> i32 {
///     (part * 100) / total  // Multiply first to preserve precision
/// }
/// ```
///
/// ### Option 3: Use div_ceil/div_floor for explicit rounding
/// ```rust
/// fn round_up_divide(x: i32, y: i32) -> i32 {
///     (x + y - 1) / y  // Rounds up instead of truncating
/// }
/// ```
///
/// Mitigation: Use floating-point types (f32/f64) when precision is needed. If integer division
/// is intentional, add a comment explaining the truncation behavior. Consider rounding behavior
/// carefully (toward zero, up, down, or nearest).

// ============================================================================
// POTENTIALLY PROBLEMATIC PATTERNS
// ============================================================================

/// PROBLEM E1405: Integer division loses precision
pub fn e1405_bad_division(total: i32, count: i32) -> i32 {
    // PROBLEM E1405: Integer division loses precision
    total / count // Should consider using floating point
}

/// PROBLEM E1405: Percentage calculation loses precision
pub fn e1405_bad_percentage(part: i32, total: i32) -> i32 {
    part / total * 100 // Wrong order! 1/3*100 = 0
}

/// PROBLEM E1405: Average loses fractional part
pub fn e1405_bad_average(values: &[i32]) -> i32 {
    if values.is_empty() {
        return 0;
    }
    values.iter().sum::<i32>() / values.len() as i32
}

/// Entry point for problem demonstration
pub fn e1405_entry() -> Result<(), Box<dyn std::error::Error>> {
    Ok(())
}

// ============================================================================
// GOOD ALTERNATIVES
// ============================================================================

/// GOOD: Use floating-point for precise division
pub fn e1405_good_float_division(total: i32, count: i32) -> f64 {
    total as f64 / count as f64
}

/// GOOD: Correct percentage order (multiply first)
pub fn e1405_good_percentage(part: i32, total: i32) -> i32 {
    if total == 0 {
        return 0;
    }
    (part * 100) / total // Multiply before divide
}

/// GOOD: Floating-point percentage
pub fn e1405_good_percentage_float(part: i32, total: i32) -> f64 {
    if total == 0 {
        return 0.0;
    }
    (part as f64 / total as f64) * 100.0
}

/// GOOD: Precise average with floating-point
pub fn e1405_good_average(values: &[i32]) -> f64 {
    if values.is_empty() {
        return 0.0;
    }
    values.iter().sum::<i32>() as f64 / values.len() as f64
}

/// GOOD: Ceiling division (rounds up)
pub fn e1405_good_div_ceil(x: i32, y: i32) -> i32 {
    if y == 0 {
        return 0;
    }
    (x + y - 1) / y // Rounds up
}

/// GOOD: Explicit truncating division with documentation
pub fn e1405_good_explicit_truncate(total_cents: i32, num_items: i32) -> i32 {
    // Intentionally truncates - we can't split cents
    total_cents / num_items
}

/// GOOD: Rounding division (rounds to nearest)
pub fn e1405_good_div_round(x: i32, y: i32) -> i32 {
    if y == 0 {
        return 0;
    }
    (x + y / 2) / y // Rounds to nearest
}

/// GOOD: Division with remainder
pub fn e1405_good_div_rem(x: i32, y: i32) -> (i32, i32) {
    if y == 0 {
        return (0, 0);
    }
    (x / y, x % y) // Returns both quotient and remainder
}

/// GOOD: Fixed-point arithmetic for money
pub fn e1405_good_money_division(cents: i64, divisor: i64) -> (i64, i64) {
    if divisor == 0 {
        return (0, 0);
    }
    let quotient = cents / divisor;
    let remainder = cents % divisor;
    (quotient, remainder) // Track remainder for proper accounting
}

// ============================================================================
// GOOD EXAMPLES unit tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_float_division() {
        assert!((e1405_good_float_division(7, 2) - 3.5).abs() < f64::EPSILON);
    }

    #[test]
    fn test_percentage_correct_order() {
        // 1/3 as percentage should be ~33, not 0
        assert_eq!(e1405_good_percentage(1, 3), 33);
    }

    #[test]
    fn test_percentage_float() {
        let pct = e1405_good_percentage_float(1, 3);
        assert!((pct - 33.333333).abs() < 0.001);
    }

    #[test]
    fn test_average_precise() {
        let avg = e1405_good_average(&[1, 2, 3, 4]);
        assert!((avg - 2.5).abs() < f64::EPSILON);
    }

    #[test]
    fn test_div_ceil() {
        assert_eq!(e1405_good_div_ceil(7, 2), 4); // Rounds up
        assert_eq!(e1405_good_div_ceil(6, 2), 3); // Exact
    }

    #[test]
    fn test_div_round() {
        assert_eq!(e1405_good_div_round(7, 2), 4);  // 3.5 rounds to 4
        assert_eq!(e1405_good_div_round(5, 2), 3);  // 2.5 rounds to 3
        assert_eq!(e1405_good_div_round(4, 2), 2);  // Exact
    }

    #[test]
    fn test_div_rem() {
        assert_eq!(e1405_good_div_rem(7, 2), (3, 1));
        assert_eq!(e1405_good_div_rem(6, 2), (3, 0));
    }

    #[test]
    fn test_money_division() {
        // Split $10.00 (1000 cents) among 3 people
        let (each, remainder) = e1405_good_money_division(1000, 3);
        assert_eq!(each, 333);      // Each gets $3.33
        assert_eq!(remainder, 1);   // 1 cent left over
    }
}
