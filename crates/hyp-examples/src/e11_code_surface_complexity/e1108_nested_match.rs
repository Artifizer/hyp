/// E1108: Deeply nested match expressions
/// Severity: MED
/// LLM confusion: 3 (MED)
///
/// Description: This code has deeply nested pattern matching (like nested switch statements),
/// making it hard to follow the logic flow. Each level of nesting increases cognitive load.
/// Fix by flattening the match expressions using early returns, combining patterns, or
/// extracting nested matches into separate functions.
///
/// Mitigation: Use `#![warn(clippy::cognitive_complexity)]` to detect overly complex patterns.
/// Flatten nested matches using tuple patterns like `match (opt1, opt2)` or use `if let` chains.
/// Extract complex match arms into helper functions.

pub fn e1108_bad_nested_match(opt1: Option<i32>, opt2: Option<i32>) -> i32 {
    // PROBLEM E1108: Deeply nested pattern matching
    match opt1 {
        Some(x) => match opt2 {
            Some(y) => match x.cmp(&y) {
                std::cmp::Ordering::Less => x,
                std::cmp::Ordering::Equal => match x {
                    0 => 1,
                    _ => x,
                },
                std::cmp::Ordering::Greater => y,
            },
            None => x,
        },
        None => 0,
    }
}

pub fn e1108_entry() -> Result<(), Box<dyn std::error::Error>> {
    let _ = e1108_bad_nested_match(Some(1), Some(2));
    Ok(())
}

// ============================================================================
// GOOD EXAMPLES - Proper alternatives
// ============================================================================

/// GOOD: Match on tuple instead of nesting
pub fn e1108_good_tuple_match(opt1: Option<i32>, opt2: Option<i32>) -> i32 {
    match (opt1, opt2) {
        (Some(x), Some(y)) if x < y => x,
        (Some(x), Some(y)) if x == y && x == 0 => 1,
        (Some(x), Some(y)) if x == y => x,
        (Some(_), Some(y)) => y, // x > y
        (Some(x), None) => x,
        (None, _) => 0,
    }
}

/// GOOD: Use if let chains (cleaner for simple cases)
pub fn e1108_good_if_let(opt1: Option<i32>, opt2: Option<i32>) -> i32 {
    let Some(x) = opt1 else { return 0 };
    let Some(y) = opt2 else { return x };

    match x.cmp(&y) {
        std::cmp::Ordering::Less => x,
        std::cmp::Ordering::Equal => if x == 0 { 1 } else { x },
        std::cmp::Ordering::Greater => y,
    }
}

/// GOOD: Extract comparison logic into helper
fn e1108_good_compare_values(x: i32, y: i32) -> i32 {
    match x.cmp(&y) {
        std::cmp::Ordering::Less => x,
        std::cmp::Ordering::Equal => if x == 0 { 1 } else { x },
        std::cmp::Ordering::Greater => y,
    }
}

pub fn e1108_good_helper_function(opt1: Option<i32>, opt2: Option<i32>) -> i32 {
    match (opt1, opt2) {
        (Some(x), Some(y)) => e1108_good_compare_values(x, y),
        (Some(x), None) => x,
        (None, _) => 0,
    }
}

/// GOOD: Use Option combinators
pub fn e1108_good_combinators(opt1: Option<i32>, opt2: Option<i32>) -> i32 {
    opt1.map_or(0, |x| {
        opt2.map_or(x, |y| e1108_good_compare_values(x, y))
    })
}

/// GOOD: Use and_then for chaining
pub fn e1108_good_and_then(opt1: Option<i32>, opt2: Option<i32>) -> i32 {
    opt1.and_then(|x| {
        opt2.map(|y| e1108_good_compare_values(x, y))
    })
    .or(opt1)
    .unwrap_or(0)
}

// ============================================================================
// GOOD EXAMPLES unit tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn e1108_good_tuple_match_handles_equal_zero() {
        assert_eq!(e1108_good_tuple_match(Some(0), Some(0)), 1);
    }

    #[test]
    fn e1108_good_helper_function_orders_values() {
        assert_eq!(e1108_good_helper_function(Some(5), Some(3)), 3);
    }

    #[test]
    fn e1108_good_combinators_defaults_to_zero() {
        assert_eq!(e1108_good_combinators(None, Some(1)), 0);
    }
}
