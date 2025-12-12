/// E1410: Float equality comparison with ==
/// Severity: MEDIUM
/// LLM confusion: 2 (LOW)
///
/// Description: Comparing floats directly with == or != is unreliable due to
/// floating-point precision issues. The classic example: 0.1 + 0.2 != 0.3 in
/// floating point! This leads to subtle bugs that are hard to track down.
///
/// Mitigation: Use epsilon comparison: (a - b).abs() < f64::EPSILON, or use
/// the `approx` crate for more sophisticated comparisons.

/// PROBLEM E1410: Direct float comparison - WILL FAIL!
pub fn e1410_bad_sum_check() -> bool {
    let a = 0.1;
    let b = 0.2;
    let sum = a + b;

    // This is FALSE! 0.1 + 0.2 != 0.3 in floating point
    sum == 0.3
}

/// PROBLEM E1410: Float comparison in validation
pub fn e1410_bad_validate_percentage(value: f64) -> bool {
    // May fail for values like 99.99999999999999
    value == 100.0 || value == 0.0
}

/// PROBLEM E1410: Float comparison in physics/game logic
pub struct Position {
    pub x: f64,
    pub y: f64,
}

impl Position {
    /// PROBLEM E1410: May fail due to accumulated floating point errors
    pub fn e1410_is_at_origin_bad(&self) -> bool {
        self.x == 0.0 && self.y == 0.0
    }
}

/// PROBLEM E1410: Checking for exact float values in search
pub fn e1410_find_exact_value(values: &[f64], target: f64) -> Option<usize> {
    values.iter().position(|&v| v == target) // Unreliable!
}

pub fn e1410_entry() -> Result<(), Box<dyn std::error::Error>> {
    // Demonstrate the problem
    let bad_result = e1410_bad_sum_check();
    println!("0.1 + 0.2 == 0.3 using ==: {}", bad_result); // false!

    Ok(())
}

// ============================================================================
// GOOD EXAMPLES - Proper float comparison
// ============================================================================

/// GOOD: Epsilon comparison for general floats
pub fn e1410_good_sum_check() -> bool {
    let a: f64 = 0.1;
    let b: f64 = 0.2;
    let sum: f64 = a + b;

    (sum - 0.3).abs() < f64::EPSILON
}

/// GOOD: Relative epsilon for larger values
pub fn e1410_good_approx_equal(a: f64, b: f64) -> bool {
    // Use a larger multiplier for practical tolerance
    let epsilon = 1e-10 * a.abs().max(b.abs()).max(1.0);
    (a - b).abs() < epsilon
}

/// GOOD: Use relative and absolute epsilon
pub fn e1410_good_nearly_equal(a: f64, b: f64, abs_eps: f64, rel_eps: f64) -> bool {
    let diff = (a - b).abs();
    diff <= abs_eps || diff <= rel_eps * a.abs().max(b.abs())
}

impl Position {
    /// GOOD: Epsilon-based origin check
    pub fn e1410_good_is_at_origin(&self) -> bool {
        self.x.abs() < f64::EPSILON && self.y.abs() < f64::EPSILON
    }

    /// GOOD: Distance-based proximity check
    pub fn e1410_good_is_near(&self, other: &Position, tolerance: f64) -> bool {
        let dx = self.x - other.x;
        let dy = self.y - other.y;
        (dx * dx + dy * dy).sqrt() < tolerance
    }
}

/// GOOD: Use a tolerance when searching
pub fn e1410_good_find_approximate_value(values: &[f64], target: f64, tolerance: f64) -> Option<usize> {
    values
        .iter()
        .position(|&v| (v - target).abs() < tolerance)
}

// ============================================================================
// GOOD EXAMPLES unit tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn e1410_good_bad_comparison_fails() {
        // This demonstrates the problem!
        assert!(!e1410_bad_sum_check());
    }

    #[test]
    fn e1410_good_sum_check_matches_epsilon() {
        assert!(e1410_good_sum_check());
    }

    #[test]
    fn e1410_good_position_epsilon() {
        let pos = Position { x: 1e-20, y: 1e-20 };
        assert!(pos.e1410_good_is_at_origin());
    }

    #[test]
    fn e1410_good_approx_equal_with_small_delta() {
        assert!(e1410_good_approx_equal(1.0, 1.0 + 1e-15));
        assert!(!e1410_good_approx_equal(1.0, 1.1));
    }
}
