/// E1102: Deeply nested logic in loops and conditions
/// Severity: MED
/// LLM confusion: 4 (HIGH)
///
/// Description: This code has deeply nested loops and conditions that make it extremely difficult
/// to understand what it does. Cognitive complexity measures how hard code is to understand, not
/// just how many branches exist. Fix by flattening nested structures, extracting inner loops into
/// separate functions, and using iterator methods to express intent more clearly.
///
/// Mitigation: Enable `#![warn(clippy::cognitive_complexity)]`. Break down nested loops into
/// separate functions with descriptive names. Use iterator combinators like `filter_map` and
/// `flat_map` to reduce nesting.

pub fn e1102_bad_deep_nested_logic_in_loops(items: Vec<i32>) -> Vec<i32> {
    // PROBLEM E1102: Deeply nested logic that's hard to understand
    let mut result = Vec::new();
    for item in items {
        if item > 0 {
            for i in 0..item {
                if i % 2 == 0 {
                    for j in 0..i {
                        if j % 3 == 0 {
                            if i == 10 {
                                result.push(j);
                            }
                        }
                    }
                }
            }
        }
    }
    result
}

// Public entry function that prepares arguments and calls the actual problem function
pub fn e1102_entry() -> Result<(), Box<dyn std::error::Error>> {
    println!("Running E1102: Deeply nested logic in loops and conditions");
    let _ = e1102_bad_deep_nested_logic_in_loops(vec![1, 2, 3]);
    Ok(())
}

// ============================================================================
// GOOD EXAMPLES - Proper alternatives
// ============================================================================

/// GOOD: Use iterator combinators to flatten the logic
pub fn e1102_good_iterator_combinators(items: Vec<i32>) -> Vec<i32> {
    items
        .into_iter()
        .filter(|&item| item > 0)
        .flat_map(|item| (0..item).filter(|i| i % 2 == 0))
        .flat_map(|i| (0..i).filter(|j| j % 3 == 0))
        .collect()
}

/// GOOD: Extract inner logic into helper functions
fn e1102_good_process_even_numbers(item: i32) -> Vec<i32> {
    (0..item)
        .filter(|i| i % 2 == 0)
        .flat_map(|i| (0..i).filter(|j| j % 3 == 0))
        .collect()
}

pub fn e1102_good_helper_functions(items: Vec<i32>) -> Vec<i32> {
    items
        .into_iter()
        .filter(|&item| item > 0)
        .flat_map(e1102_good_process_even_numbers)
        .collect()
}

/// GOOD: Use early continue to reduce nesting
pub fn e1102_good_early_continue(items: Vec<i32>) -> Vec<i32> {
    let mut result = Vec::new();

    for item in items {
        if item <= 0 {
            continue;
        }

        for i in (0..item).filter(|i| i % 2 == 0) {
            for j in (0..i).filter(|j| j % 3 == 0) {
                result.push(j);
            }
        }
    }

    result
}

/// GOOD: Break into named steps
pub fn e1102_good_named_steps(items: Vec<i32>) -> Vec<i32> {
    let positive_items = items.into_iter().filter(|&x| x > 0);
    let even_indices = positive_items.flat_map(|item| {
        (0..item).filter(|i| i % 2 == 0)
    });
    let divisible_by_three = even_indices.flat_map(|i| {
        (0..i).filter(|j| j % 3 == 0)
    });

    divisible_by_three.collect()
}

// ============================================================================
// GOOD EXAMPLES unit tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn e1102_good_iterator_combinators_collects_expected() {
        let out = e1102_good_iterator_combinators(vec![4]);
        // With input [4]: i=2 produces j=0, so result is [0]
        assert!(out.contains(&0));
        assert_eq!(out.len(), 1);
    }

    #[test]
    fn e1102_good_helper_functions_matches_iterator() {
        let input = vec![5];
        assert_eq!(
            e1102_good_helper_functions(input.clone()),
            e1102_good_iterator_combinators(input)
        );
    }
}
