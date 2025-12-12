/// E1107: Deeply nested conditionals
/// Severity: MED
/// LLM confusion: 3 (MED)
///
/// Description: This function combines multiple layers of `if/else` and `match` in one place,
/// making the control flow hard to follow. It's similar in spirit to E1101/E1102 but focused on
/// nested boolean conditions instead of arithmetic.
///
/// Mitigation: Extract inner branches into helper functions and avoid more than a couple of
/// nested `if/else` levels.

pub fn e1107_bad_deep_nesting(flag_a: bool, flag_b: bool, flag_c: bool, n: i32) -> i32 {
    if flag_a {
        if n > 0 {
            if flag_b {
                if n % 2 == 0 {
                    n * 2
                } else if flag_c {
                    n * 3
                } else {
                    n + 1
                }
            } else if flag_c {
                if n > 10 {
                    n - 1
                } else {
                    n + 10
                }
            } else {
                0
            }
        } else if flag_b && flag_c {
            -n
        } else if flag_b {
            n - 5
        } else {
            n
        }
    } else if flag_b {
        if flag_c {
            n * n
        } else {
            n / 2
        }
    } else if flag_c {
        match n {
            0 => 0,
            1..=10 => n + 100,
            _ => n - 100,
        }
    } else {
        -1
    }
}

pub fn e1107_entry() -> Result<(), Box<dyn std::error::Error>> {
    let _ = e1107_bad_deep_nesting(true, false, true, 7);
    Ok(())
}

// ============================================================================
// GOOD EXAMPLES - Proper alternatives
// ============================================================================

/// GOOD: Use early returns to flatten structure
pub fn e1107_good_early_returns(flag_a: bool, flag_b: bool, flag_c: bool, n: i32) -> i32 {
    // Handle simple cases first
    if !flag_a && !flag_b && !flag_c {
        return -1;
    }

    if !flag_a && flag_b && flag_c {
        return n * n;
    }

    if !flag_a && flag_b {
        return n / 2;
    }

    if !flag_a && flag_c {
        return match n {
            0 => 0,
            1..=10 => n + 100,
            _ => n - 100,
        };
    }

    // Handle flag_a cases
    if flag_a && n <= 0 {
        if flag_b && flag_c {
            return -n;
        }
        if flag_b {
            return n - 5;
        }
        return n;
    }

    // flag_a and n > 0
    if flag_b && n % 2 == 0 {
        return n * 2;
    }
    if flag_b && flag_c {
        return n * 3;
    }
    if flag_b {
        return n + 1;
    }

    // flag_a, n > 0, !flag_b
    if flag_c && n > 10 {
        return n - 1;
    }
    if flag_c {
        return n + 10;
    }

    0
}

/// GOOD: Extract complex logic into helper functions
fn e1107_good_handle_positive_n(flag_b: bool, flag_c: bool, n: i32) -> i32 {
    if flag_b {
        if n % 2 == 0 { n * 2 }
        else if flag_c { n * 3 }
        else { n + 1 }
    } else if flag_c {
        if n > 10 { n - 1 } else { n + 10 }
    } else {
        0
    }
}

fn e1107_good_handle_negative_n(flag_b: bool, flag_c: bool, n: i32) -> i32 {
    if flag_b && flag_c { -n }
    else if flag_b { n - 5 }
    else { n }
}

pub fn e1107_good_helper_functions(flag_a: bool, flag_b: bool, flag_c: bool, n: i32) -> i32 {
    if flag_a {
        if n > 0 {
            e1107_good_handle_positive_n(flag_b, flag_c, n)
        } else {
            e1107_good_handle_negative_n(flag_b, flag_c, n)
        }
    } else if flag_b {
        if flag_c { n * n } else { n / 2 }
    } else if flag_c {
        match n {
            0 => 0,
            1..=10 => n + 100,
            _ => n - 100,
        }
    } else {
        -1
    }
}

/// GOOD: Use match on tuples for clearer patterns
pub fn e1107_good_tuple_match(flag_a: bool, flag_b: bool, flag_c: bool, n: i32) -> i32 {
    match (flag_a, flag_b, flag_c, n > 0, n % 2 == 0) {
        (true, true, _, true, true) => n * 2,
        (true, true, true, true, false) => n * 3,
        (true, true, false, true, false) => n + 1,
        (true, false, true, true, _) if n > 10 => n - 1,
        (true, false, true, true, _) => n + 10,
        (true, _, _, false, _) if flag_b && flag_c => -n,
        (true, true, _, false, _) => n - 5,
        (true, _, _, false, _) => n,
        (true, false, false, _, _) => 0,
        (false, true, true, _, _) => n * n,
        (false, true, false, _, _) => n / 2,
        (false, false, true, _, _) => match n {
            0 => 0,
            1..=10 => n + 100,
            _ => n - 100,
        },
        _ => -1,
    }
}

// ============================================================================
// GOOD EXAMPLES unit tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn e1107_good_early_returns_handles_simple_case() {
        assert_eq!(e1107_good_early_returns(false, false, false, 5), -1);
    }

    #[test]
    fn e1107_good_helper_functions_handles_positive() {
        assert_eq!(e1107_good_helper_functions(true, true, false, 4), 8);
    }

    #[test]
    fn e1107_good_tuple_match_handles_tuple() {
        assert_eq!(e1107_good_tuple_match(true, true, true, 5), 15);
    }
}
