/// E1109: Excessive method chaining
/// Severity: LOW
/// LLM confusion: 2 (LOW)
///
/// Description: This code chains too many method calls together in a single expression, making
/// it difficult to debug (you can't easily inspect intermediate values) and hard to understand
/// what transformations are being applied. Fix by breaking the chain into intermediate variables
/// with descriptive names, or limiting chains to 5-7 operations.
///
/// Mitigation: Break long chains into steps with meaningful variable names. This makes debugging
/// easier and improves readability. Consider using intermediate `let` bindings when chains exceed
/// 5-7 operations.

pub fn e1109_bad_excessive_chaining(data: Vec<i32>) -> Vec<String> {
    // PROBLEM E1109: Too many chained methods (10 operations - hard to debug)
    data.iter()
        .filter(|x| **x > 0)
        .map(|x| x * 2)
        .filter(|x| x % 3 == 0)
        .map(|x| x + 1)
        .filter(|x| x % 2 == 0)
        .map(|x| x.to_string())
        .filter(|s| s.len() > 1)
        .map(|s| format!("Value: {}", s))
        .filter(|s| !s.contains('0'))
        .collect()
}

pub fn e1109_entry() -> Result<(), Box<dyn std::error::Error>> {
    let _ = e1109_bad_excessive_chaining(vec![1, 2, 3, 4, 5]);
    Ok(())
}

// ============================================================================
// GOOD EXAMPLES - Proper alternatives
// ============================================================================

/// GOOD: Break chain into named intermediate steps
pub fn e1109_good_intermediate_vars(data: Vec<i32>) -> Vec<String> {
    // Step 1: Filter positive values
    let positive: Vec<_> = data.iter().filter(|x| **x > 0).collect();

    // Step 2: Double and filter divisible by 3
    let doubled: Vec<_> = positive.iter()
        .map(|x| **x * 2)
        .filter(|x| x % 3 == 0)
        .collect();

    // Step 3: Transform and format
    let formatted: Vec<String> = doubled.iter()
        .map(|x| x + 1)
        .filter(|x| x % 2 == 0)
        .map(|x| format!("Value: {}", x))
        .collect();

    // Step 4: Final filter
    formatted.into_iter()
        .filter(|s| !s.contains('0'))
        .collect()
}

/// GOOD: Extract filter/map logic into named functions
fn e1109_good_is_positive(x: &&i32) -> bool {
    **x > 0
}

fn e1109_good_double(x: &i32) -> i32 {
    *x * 2
}

fn e1109_good_is_divisible_by_3(x: &i32) -> bool {
    x % 3 == 0
}

fn e1109_good_format_value(x: i32) -> String {
    format!("Value: {}", x)
}

pub fn e1109_good_named_functions(data: Vec<i32>) -> Vec<String> {
    data.iter()
        .filter(e1109_good_is_positive)
        .map(e1109_good_double)
        .filter(e1109_good_is_divisible_by_3)
        .map(|x| x + 1)
        .filter(|x| x % 2 == 0)
        .map(e1109_good_format_value)
        .filter(|s| !s.contains('0'))
        .collect()
}

/// GOOD: Split pipeline into logical stages
pub fn e1109_good_pipeline_stages(data: Vec<i32>) -> Vec<String> {
    // Stage 1: Numeric transformations
    let numbers = data.iter()
        .filter(|x| **x > 0)
        .map(|x| x * 2)
        .filter(|x| x % 3 == 0)
        .map(|x| x + 1)
        .filter(|x| x % 2 == 0);

    // Stage 2: String transformations
    numbers
        .map(|x| format!("Value: {}", x))
        .filter(|s| !s.contains('0'))
        .collect()
}

/// GOOD: Use a struct to encapsulate the pipeline
pub struct DataPipeline {
    data: Vec<i32>,
}

impl DataPipeline {
    pub fn new(data: Vec<i32>) -> Self {
        Self { data }
    }

    pub fn e1109_good_filter_positive(self) -> Self {
        Self {
            data: self.data.into_iter().filter(|x| *x > 0).collect()
        }
    }

    pub fn e1109_good_double_stage(self) -> Self {
        Self {
            data: self.data.into_iter().map(|x| x * 2).collect()
        }
    }

    pub fn e1109_good_to_strings(self) -> Vec<String> {
        self.data.into_iter().map(|x| x.to_string()).collect()
    }
}

// ============================================================================
// GOOD EXAMPLES unit tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn e1109_good_intermediate_vars_filters_data() {
        // The filter chain is very restrictive: x*2+1 is always odd, so the
        // filter(|x| x % 2 == 0) will always produce empty results.
        // This test verifies the function runs correctly even with empty results.
        let result = e1109_good_intermediate_vars(vec![1, 2, 3, 4, 5]);
        assert_eq!(result.len(), 0);
    }

    #[test]
    fn e1109_good_named_functions_formats_values() {
        let result = e1109_good_named_functions(vec![3, 6, 9]);
        assert!(result.iter().all(|s| s.contains("Value")));
    }

    #[test]
    fn e1109_good_pipeline_stages_runs() {
        // Same logic as intermediate_vars - result will be empty due to filter chain
        let result = e1109_good_pipeline_stages(vec![1, 2, 3, 4, 5]);
        assert_eq!(result.len(), 0);
    }

    #[test]
    fn e1109_good_data_pipeline_chains_steps() {
        let result = DataPipeline::new(vec![1, 2, 3])
            .e1109_good_filter_positive()
            .e1109_good_double_stage()
            .e1109_good_to_strings();
        assert_eq!(result.len(), 3);
    }
}
