/// E1101: High cyclomatic complexity
/// Severity: MED
/// LLM confusion: 3 (MED)
///
/// Description: This function has too many decision points (if/else branches), making it hard to
/// understand, test, and maintain. High cyclomatic complexity means there are many different paths
/// through the code. Fix by breaking the function into smaller, focused functions or using lookup
/// tables/pattern matching to simplify the logic.
///
/// Mitigation: Use `#![warn(clippy::cognitive_complexity)]` to detect overly complex functions.
/// Refactor by extracting nested conditions into separate helper functions. Consider using early
/// returns to reduce nesting depth.

#[allow(clippy::needless_return, clippy::collapsible_else_if)]
pub fn e1101_bad_high_cyclomatic_complexity(x: i32, y: i32, z: i32) -> i32 {
    // PROBLEM E1101: Too many branches (cyclomatic complexity > 10)
    if x > 0 {
        if y > 0 {
            if z > 0 {
                return x + y + z;
            } else if z < -10 {
                if z < -20 {
                    return x + y;
                }
            } else {
                println!("z is between -10 and 0");
            }
        } else if y < -5 {
            if x > 10 {
                return x - y;
            } else {
                return y - x;
            }
        } else {
            if z > 0 {
                return x * z;
            }
        }
    } else if x < -5 {
        if y > 0 {
            if z > 0 {
                return y + z;
            } else if z < -10 {
                return y - z;
            }
        } else {
            return x * y;
        }
    } else {
        if y > 0 && z > 0 {
            return y + z;
        } else if y < 0 && z < 0 {
            return y * z;
        }
    }

    // Default case
    x + y + z
}

pub fn e1101_entry() -> Result<(), Box<dyn std::error::Error>> {
    let _ = e1101_bad_high_cyclomatic_complexity(1, 2, 3);
    Ok(())
}

// ============================================================================
// GOOD EXAMPLES - Proper alternatives
// ============================================================================

/// GOOD: Use early returns to flatten the structure
pub fn e1101_good_early_returns(x: i32, y: i32, z: i32) -> i32 {
    // Handle edge cases first
    if x <= 0 && y <= 0 {
        return x * y;
    }

    // Handle the common positive case
    if x > 0 && y > 0 && z > 0 {
        return x + y + z;
    }

    // Default computation
    x + y + z
}

/// GOOD: Extract complex conditions into helper functions
fn e1101_good_is_positive_combination(x: i32, y: i32, z: i32) -> bool {
    x > 0 && y > 0 && z > 0
}

fn e1101_good_compute_positive_result(x: i32, y: i32, z: i32) -> i32 {
    x + y + z
}

pub fn e1101_good_helper_functions(x: i32, y: i32, z: i32) -> i32 {
    if e1101_good_is_positive_combination(x, y, z) {
        return e1101_good_compute_positive_result(x, y, z);
    }

    x + y + z
}

/// GOOD: Use match for clearer logic
pub fn e1101_good_match_expression(x: i32, y: i32, z: i32) -> i32 {
    match (x.signum(), y.signum(), z.signum()) {
        (1, 1, 1) => x + y + z,
        (1, 1, _) => x + y,
        (1, -1, _) if x > 10 => x - y,
        (-1, 1, 1) => y + z,
        (-1, 1, -1) => y - z,
        _ => x + y + z,
    }
}

/// GOOD: Use a lookup table for complex mappings
pub fn e1101_good_lookup_table(category: u8) -> &'static str {
    const CATEGORIES: [&str; 5] = ["none", "low", "medium", "high", "critical"];
    CATEGORIES.get(category as usize).unwrap_or(&"unknown")
}

// ============================================================================
// GOOD EXAMPLES unit tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn e1101_good_early_returns_handles_positive() {
        assert_eq!(e1101_good_early_returns(5, 3, 2), 10);
    }

    #[test]
    fn e1101_good_helper_functions_detects_positive_combo() {
        assert_eq!(e1101_good_helper_functions(1, 1, 1), 3);
    }

    #[test]
    fn e1101_good_lookup_table_returns_default() {
        assert_eq!(e1101_good_lookup_table(10), "unknown");
    }
}
