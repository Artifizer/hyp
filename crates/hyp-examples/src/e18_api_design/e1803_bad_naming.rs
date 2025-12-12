/// E1803: Bad naming
/// Severity: LOW
/// LLM confusion: 1 (LOW)
///
/// Description: Function names should clearly describe what the function does. This function
/// multiplies two numbers and adds 42, but the name gives no indication of this behavior. Poor
/// naming makes code hard to understand and maintain. Fix by using descriptive names that explain
/// the function's purpose.
///
/// ## The Unclear Name Problem
///
/// ```text
/// fn process(x: i32, y: i32) -> i32 {
///     x * y + 42
/// }
///
/// let result = process(3, 4);  // What does this do?
/// ```
///
/// ## Why This Matters
///
/// 1. **Readability**: Names are documentation
/// 2. **Maintenance**: Unclear names lead to bugs
/// 3. **Onboarding**: New developers struggle with unclear code
/// 4. **Refactoring**: Hard to know if changes break expectations
///
/// ## The Right Solutions
///
/// ### Option 1: Descriptive function names
/// ```rust
/// fn calculate_total_with_tax(subtotal: i32, tax_rate: i32) -> i32 {
///     subtotal + (subtotal * tax_rate / 100)
/// }
/// ```
///
/// ### Option 2: Domain-specific names
/// ```rust
/// fn apply_magic_number_adjustment(product: i32) -> i32 {
///     product + 42
/// }
/// ```
///
/// ### Option 3: Use types to clarify
/// ```rust
/// fn multiply_and_offset(a: i32, b: i32, offset: i32) -> i32 {
///     a * b + offset
/// }
/// ```
///
/// Mitigation: Use descriptive, verb-based names for functions. Follow Rust naming conventions:
/// snake_case for functions, describe what the function does. Use `#![warn(clippy::module_name_repetitions)]`
/// to catch redundant naming. Good names are documentation.

// ============================================================================
// PROBLEMATIC PATTERNS
// ============================================================================

/// PROBLEM E1803: Function name doesn't describe what it does
pub fn e1803_bad_naming(x: i32, y: i32) -> i32 {
    x * y + 42
}

/// PROBLEM E1803: Single letter names
pub fn f(a: i32, b: i32) -> i32 {
    a + b
}

/// PROBLEM E1803: Misleading name (doesn't add, it multiplies)
pub fn add_numbers(x: i32, y: i32) -> i32 {
    x * y // Actually multiplies!
}

/// PROBLEM E1803: Overly generic name
pub fn process(data: &[i32]) -> i32 {
    data.iter().sum()
}

/// PROBLEM E1803: Abbreviations that aren't clear
pub fn calc_avg_tmp(vals: &[f64]) -> f64 {
    vals.iter().sum::<f64>() / vals.len() as f64
}

/// Entry point for problem demonstration
pub fn e1803_entry() -> Result<(), Box<dyn std::error::Error>> {
    let _ = e1803_bad_naming(1, 2);
    Ok(())
}

// ============================================================================
// GOOD ALTERNATIVES
// ============================================================================

/// GOOD: Descriptive function name
pub fn multiply_and_add_magic_number(x: i32, y: i32) -> i32 {
    x * y + 42
}

/// GOOD: Clear parameter names
pub fn add_integers(first: i32, second: i32) -> i32 {
    first + second
}

/// GOOD: Accurate name matching behavior
pub fn multiply_integers(x: i32, y: i32) -> i32 {
    x * y
}

/// GOOD: Specific name describing the operation
pub fn sum_all_values(values: &[i32]) -> i32 {
    values.iter().sum()
}

/// GOOD: Full words, no confusing abbreviations
pub fn calculate_average_temperature(temperatures: &[f64]) -> f64 {
    if temperatures.is_empty() {
        return 0.0;
    }
    temperatures.iter().sum::<f64>() / temperatures.len() as f64
}

/// GOOD: Verb-based names for actions
pub fn validate_email(email: &str) -> bool {
    email.contains('@')
}

/// GOOD: Predicate functions start with is/has/can
pub fn is_valid_email(email: &str) -> bool {
    email.contains('@') && email.contains('.')
}

/// GOOD: Conversion functions use from/to/into
pub fn celsius_to_fahrenheit(celsius: f64) -> f64 {
    celsius * 9.0 / 5.0 + 32.0
}

/// GOOD: Getters don't need "get_" prefix in Rust
pub struct Temperature {
    celsius: f64,
}

impl Temperature {
    pub fn new(celsius: f64) -> Self {
        Self { celsius }
    }

    // GOOD: Just "celsius()" not "get_celsius()"
    pub fn celsius(&self) -> f64 {
        self.celsius
    }

    // GOOD: Conversion method
    pub fn as_fahrenheit(&self) -> f64 {
        celsius_to_fahrenheit(self.celsius)
    }
}

/// GOOD: Builder methods return Self
pub struct ConfigBuilder {
    timeout: u64,
    retries: u32,
}

impl ConfigBuilder {
    pub fn new() -> Self {
        Self {
            timeout: 30,
            retries: 3,
        }
    }

    // GOOD: Method named after what it sets
    pub fn timeout(mut self, seconds: u64) -> Self {
        self.timeout = seconds;
        self
    }

    // GOOD: Method named after what it sets
    pub fn retries(mut self, count: u32) -> Self {
        self.retries = count;
        self
    }
}

impl Default for ConfigBuilder {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// GOOD EXAMPLES unit tests
// ============================================================================


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_descriptive_names() {
        assert_eq!(multiply_and_add_magic_number(3, 4), 54);
        assert_eq!(add_integers(2, 3), 5);
        assert_eq!(multiply_integers(3, 4), 12);
    }

    #[test]
    fn test_sum_values() {
        assert_eq!(sum_all_values(&[1, 2, 3, 4, 5]), 15);
    }

    #[test]
    fn test_predicates() {
        assert!(is_valid_email("test@example.com"));
        assert!(!is_valid_email("invalid"));
    }

    #[test]
    fn test_conversions() {
        assert!((celsius_to_fahrenheit(0.0) - 32.0).abs() < 0.01);
        assert!((celsius_to_fahrenheit(100.0) - 212.0).abs() < 0.01);
    }
}
