/// E1110: Deeply nested callbacks/closures
/// Severity: MED
/// LLM confusion: 5 (HIGHEST)
///
/// Description: This code has multiple layers of nested closures (anonymous functions), making it
/// extremely difficult to follow the control flow and understand what data is being captured at
/// each level. Each closure can access variables from outer scopes, creating a complex web of
/// dependencies. It's like having functions within functions within functions, where each inner
/// function can use variables from all the outer functions. This "callback hell" or "pyramid of doom"
/// makes code nearly impossible to debug and maintain. Fix by flattening the structure using
/// async/await, extracting closures into named functions, or using combinator methods.
///
/// Mitigation: Limit closure nesting to 2-3 levels maximum. Extract nested closures into named
/// functions with clear signatures. Use async/await instead of nested callbacks for asynchronous
/// code. Consider using the `?` operator and early returns to reduce nesting. Use combinator
/// methods like `and_then`, `map`, `flat_map` to express intent more clearly.

pub fn e1110_bad_nested_callbacks() {
    let data = [1, 2, 3, 4, 5];

    // PROBLEM E1110: Deeply nested closures (5 levels deep)
    data.iter().for_each(|x| {
        if *x > 0 {
            let result =
                (0..*x)
                    .map(|y| {
                        if y % 2 == 0 {
                            Some(
                                (0..y)
                                    .filter_map(|z| {
                                        if z % 3 == 0 {
                                            Some((0..z).fold(0, |acc, w| {
                                                if w % 2 == 0 {
                                                    acc + w
                                                } else {
                                                    acc
                                                }
                                            }))
                                        } else {
                                            None
                                        }
                                    })
                                    .sum::<i32>(),
                            )
                        } else {
                            None
                        }
                    })
                    .collect::<Vec<_>>();

            let _sum: i32 = result.iter().filter_map(|x| *x).sum();
        }
    });
}

// PROBLEM E1110: Nested closures with complex variable capture
pub fn e1110_bad_callback_hell() {
    let multiplier = 2;
    let threshold = 10;

    let process = |data: Vec<i32>| {
        data.into_iter()
            .filter(|x| *x > threshold) // Captures threshold
            .map(|x| {
                let doubled = x * multiplier; // Captures multiplier
                (0..doubled)
                    .filter_map(|y| {
                        if y % 2 == 0 {
                            Some(
                                (0..y)
                                    .map(|z| {
                                        z * multiplier // Captures multiplier from outer scope
                                    })
                                    .sum::<i32>(),
                            )
                        } else {
                            None
                        }
                    })
                    .collect::<Vec<_>>()
            })
            .flatten()
            .collect::<Vec<_>>()
    };

    let _result = process(vec![5, 10, 15, 20]);
}

pub fn e1110_entry() -> Result<(), Box<dyn std::error::Error>> {
    e1110_bad_nested_callbacks();
    e1110_bad_callback_hell();
    Ok(())
}

// ============================================================================
// GOOD EXAMPLES - Proper alternatives
// ============================================================================

/// GOOD: Extract inner logic into named functions
fn e1110_good_sum_even_numbers_in_range(n: i32) -> i32 {
    (0..n).filter(|w| w % 2 == 0).sum()
}

fn e1110_good_process_divisible_by_three(y: i32) -> i32 {
    (0..y)
        .filter(|z| z % 3 == 0)
        .map(e1110_good_sum_even_numbers_in_range)
        .sum()
}

fn e1110_good_process_even_numbers(x: i32) -> Vec<Option<i32>> {
    (0..x)
        .map(|y| {
            if y % 2 == 0 {
                Some(e1110_good_process_divisible_by_three(y))
            } else {
                None
            }
        })
        .collect()
}

pub fn e1110_good_named_functions() {
    let data = [1, 2, 3, 4, 5];

    for x in data.iter().filter(|&&x| x > 0) {
        let result = e1110_good_process_even_numbers(*x);
        let _sum: i32 = result.iter().filter_map(|x| *x).sum();
    }
}

/// GOOD: Use flat_map to flatten nested iterations
pub fn e1110_good_flat_map() {
    let data = [1, 2, 3, 4, 5];

    let _results: Vec<i32> = data.iter()
        .filter(|&&x| x > 0)
        .flat_map(|&x| {
            (0..x)
                .filter(|y| y % 2 == 0)
                .flat_map(|y| {
                    (0..y)
                        .filter(|z| z % 3 == 0)
                        .map(e1110_good_sum_even_numbers_in_range)
                })
        })
        .collect();
}

/// GOOD: Use for loops for complex nested logic
pub fn e1110_good_for_loops() {
    let data = [1, 2, 3, 4, 5];

    for &x in &data {
        if x <= 0 {
            continue;
        }

        let mut results = Vec::new();

        for y in 0..x {
            if y % 2 != 0 {
                continue;
            }

            let mut sum = 0;
            for z in (0..y).filter(|z| z % 3 == 0) {
                sum += e1110_good_sum_even_numbers_in_range(z);
            }
            results.push(sum);
        }

        let _total: i32 = results.iter().sum();
    }
}

/// GOOD: Move captured variables into function parameters
fn e1110_good_process_data_with_params(
    data: Vec<i32>,
    multiplier: i32,
    threshold: i32,
) -> Vec<i32> {
    data.into_iter()
        .filter(|x| *x > threshold)
        .flat_map(|x| {
            let doubled = x * multiplier;
            (0..doubled)
                .filter(|y| y % 2 == 0)
                .map(move |y| (0..y).map(|z| z * multiplier).sum::<i32>())
        })
        .collect()
}

pub fn e1110_good_explicit_params() {
    let _result = e1110_good_process_data_with_params(vec![5, 10, 15, 20], 2, 10);
}

// ============================================================================
// GOOD EXAMPLES unit tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn e1110_good_named_functions_runs() {
        e1110_good_named_functions();
    }

    #[test]
    fn e1110_good_flat_map_produces_results() {
        e1110_good_flat_map();
    }

    #[test]
    fn e1110_good_process_data_with_params_filters_threshold() {
        let result = e1110_good_process_data_with_params(vec![5, 10, 15, 20], 2, 10);
        assert!(!result.is_empty());
        assert!(result.iter().all(|v| *v >= 0));
    }
}
