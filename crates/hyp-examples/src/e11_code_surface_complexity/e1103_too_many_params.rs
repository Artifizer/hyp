/// E1103: Too many function parameters
/// Severity: LOW
/// LLM confusion: 2 (LOW)
///
/// Description: This function takes too many parameters (more than 7), making it difficult to
/// call correctly and hard to remember the parameter order. Functions with many parameters are
/// error-prone because it's easy to pass arguments in the wrong order. Fix by grouping related
/// parameters into a struct or using a builder pattern for complex configurations.
///
/// Mitigation: Use `#![warn(clippy::too_many_arguments)]` to catch functions with too many
/// parameters. Group related parameters into a configuration struct. Consider using the builder
/// pattern for functions that need many optional parameters.

#[allow(clippy::too_many_arguments)]
pub fn e1103_bad_too_many_params(
    param_a: i32,
    param_b: i32,
    param_c: i32,
    param_d: i32,
    param_e: i32,
    param_f: i32,
    param_g: i32,
    param_h: i32,
) -> i32 {
    param_a + param_b + param_c + param_d + param_e + param_f + param_g + param_h
}

pub fn e1103_entry() -> Result<(), Box<dyn std::error::Error>> {
    let _ = e1103_bad_too_many_params(1, 2, 3, 4, 5, 6, 7, 8);
    Ok(())
}

// ============================================================================
// GOOD EXAMPLES - Proper alternatives
// ============================================================================

/// GOOD: Group related parameters into a struct
#[derive(Default)]
pub struct ComputationParams {
    pub a: i32,
    pub b: i32,
    pub c: i32,
    pub d: i32,
    pub e: i32,
    pub f: i32,
    pub g: i32,
    pub h: i32,
}

pub fn e1103_good_params_struct(params: ComputationParams) -> i32 {
    params.a + params.b + params.c + params.d + params.e + params.f + params.g + params.h
}

/// GOOD: Use builder pattern for complex construction
pub struct ComputationBuilder {
    params: ComputationParams,
}

impl ComputationBuilder {
    pub fn new() -> Self {
        Self {
            params: ComputationParams::default(),
        }
    }

    pub fn e1103_good_a(mut self, value: i32) -> Self {
        self.params.a = value;
        self
    }

    pub fn e1103_good_b(mut self, value: i32) -> Self {
        self.params.b = value;
        self
    }

    // ... other setters

    pub fn e1103_good_compute(self) -> i32 {
        e1103_good_params_struct(self.params)
    }
}

impl Default for ComputationBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// GOOD: Use arrays/slices for homogeneous parameters
pub fn e1103_good_slice_params(values: &[i32]) -> i32 {
    values.iter().sum()
}

/// GOOD: Split into multiple focused functions
pub fn e1103_good_split_functions(
    primary: (i32, i32, i32),
    secondary: (i32, i32, i32),
) -> i32 {
    let (a, b, c) = primary;
    let (d, e, f) = secondary;
    a + b + c + d + e + f
}

// ============================================================================
// GOOD EXAMPLES unit tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn e1103_good_params_struct_sums_values() {
        let params = ComputationParams { a:1,b:2,c:3,d:4,e:5,f:6,g:7,h:8 };
        assert_eq!(e1103_good_params_struct(params), 36);
    }

    #[test]
    fn e1103_good_builder_sets_fields() {
        let total = ComputationBuilder::new()
            .e1103_good_a(1)
            .e1103_good_b(2)
            .e1103_good_compute();
        assert_eq!(total, 3);
    }

    #[test]
    fn e1103_good_slice_params_sums_slice() {
        assert_eq!(e1103_good_slice_params(&[1,2,3]), 6);
    }
}
