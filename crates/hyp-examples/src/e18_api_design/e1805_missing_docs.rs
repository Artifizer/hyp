/// E1805: Missing documentation
/// Severity: LOW
/// LLM confusion: 1 (LOW)
///
/// Description: Public API functions should have documentation comments explaining what they do,
/// their parameters, return values, and any important behavior. This public function has no
/// documentation, making it hard for users to understand how to use it. Fix by adding doc comments
/// with `///` describing the function.
///
/// ## The Undocumented API Problem
///
/// ```text
/// pub fn process(x: i32) -> i32 {
///     x * 2
/// }
/// // What does this do? What are valid inputs? What are edge cases?
/// ```
///
/// ## Why This Matters
///
/// 1. **Usability**: Users can't use what they don't understand
/// 2. **Maintenance**: Future developers (including you) need context
/// 3. **Testing**: Doc examples are tested by cargo test
/// 4. **IDE support**: Doc comments appear in hover/autocomplete
///
/// ## The Right Solutions
///
/// ### Option 1: Basic doc comment
/// ```rust
/// /// Doubles the input value.
/// pub fn double(x: i32) -> i32 {
///     x * 2
/// }
/// ```
///
/// ### Option 2: Full documentation
/// ```rust
/// /// Calculates the factorial of a non-negative integer.
/// ///
/// /// # Arguments
/// ///
/// /// * `n` - A non-negative integer
/// ///
/// /// # Returns
/// ///
/// /// The factorial of `n`
/// ///
/// /// # Panics
/// ///
/// /// Panics if `n` is greater than 20 (overflow)
/// ///
/// /// # Examples
/// ///
/// /// ```
/// /// let result = factorial(5);
/// /// assert_eq!(result, 120);
/// /// ```
/// pub fn factorial(n: u64) -> u64 { ... }
/// ```
///
/// Mitigation: Use `#![warn(missing_docs)]` to require documentation on all public items. Write
/// doc comments with `///` for functions, `//!` for modules. Include examples in doc comments -
/// they're tested by `cargo test`. Good documentation is part of a good API.

// ============================================================================
// PROBLEMATIC PATTERNS
// ============================================================================

/// PROBLEM E1805: No doc comment on public function
pub fn e1805_missing_docs(x: i32) -> i32 {
    x * 2
}

// PROBLEM E1805: No docs on public struct
pub struct UndocumentedStruct {
    pub field: i32,
}

// PROBLEM E1805: No docs on public enum
pub enum UndocumentedEnum {
    VariantA,
    VariantB,
}

/// Entry point for problem demonstration
pub fn e1805_entry() -> Result<(), Box<dyn std::error::Error>> {
    let _ = e1805_missing_docs(42);
    Ok(())
}

// ============================================================================
// GOOD ALTERNATIVES
// ============================================================================

/// Doubles the input value.
///
/// This function takes an integer and returns twice its value.
/// It handles negative numbers correctly.
///
/// # Arguments
///
/// * `x` - The value to double
///
/// # Returns
///
/// The input value multiplied by 2
///
/// # Examples
///
/// ```
/// use problem_examples::e18_api_design::e1805_missing_docs::double;
///
/// assert_eq!(double(5), 10);
/// assert_eq!(double(-3), -6);
/// ```
pub fn double(x: i32) -> i32 {
    x * 2
}

/// A point in 2D space.
///
/// Represents a coordinate with x and y values.
/// Both coordinates are stored as 64-bit floating point numbers.
///
/// # Examples
///
/// ```
/// use problem_examples::e18_api_design::e1805_missing_docs::Point;
///
/// let origin = Point::new(0.0, 0.0);
/// let point = Point::new(3.0, 4.0);
/// assert!((point.distance_from(&origin) - 5.0).abs() < 0.001);
/// ```
#[derive(Debug, Clone, Copy)]
pub struct Point {
    /// The x-coordinate
    pub x: f64,
    /// The y-coordinate
    pub y: f64,
}

impl Point {
    /// Creates a new point at the given coordinates.
    ///
    /// # Arguments
    ///
    /// * `x` - The x-coordinate
    /// * `y` - The y-coordinate
    pub fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }

    /// Calculates the Euclidean distance from this point to another.
    ///
    /// # Arguments
    ///
    /// * `other` - The point to measure distance to
    ///
    /// # Returns
    ///
    /// The distance between the two points
    pub fn distance_from(&self, other: &Point) -> f64 {
        let dx = self.x - other.x;
        let dy = self.y - other.y;
        (dx * dx + dy * dy).sqrt()
    }
}

/// The result of a comparison operation.
///
/// Used when comparing two values that may be less than,
/// equal to, or greater than each other.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompareResult {
    /// The first value is less than the second
    Less,
    /// The values are equal
    Equal,
    /// The first value is greater than the second
    Greater,
}

/// Compares two integers.
///
/// # Arguments
///
/// * `a` - The first value
/// * `b` - The second value
///
/// # Returns
///
/// A [`CompareResult`] indicating the relationship between `a` and `b`
///
/// # Examples
///
/// ```
/// use problem_examples::e18_api_design::e1805_missing_docs::{compare, CompareResult};
///
/// assert_eq!(compare(1, 2), CompareResult::Less);
/// assert_eq!(compare(2, 2), CompareResult::Equal);
/// assert_eq!(compare(3, 2), CompareResult::Greater);
/// ```
pub fn compare(a: i32, b: i32) -> CompareResult {
    use std::cmp::Ordering;
    match a.cmp(&b) {
        Ordering::Less => CompareResult::Less,
        Ordering::Equal => CompareResult::Equal,
        Ordering::Greater => CompareResult::Greater,
    }
}

/// Parses a string as an integer.
///
/// # Arguments
///
/// * `s` - A string slice containing the integer to parse
///
/// # Returns
///
/// * `Ok(i32)` - The parsed integer
/// * `Err(&str)` - An error message if parsing failed
///
/// # Errors
///
/// Returns an error if:
/// - The string is empty
/// - The string contains non-numeric characters
/// - The number is out of range for i32
///
/// # Examples
///
/// ```
/// use problem_examples::e18_api_design::e1805_missing_docs::parse_int;
///
/// assert_eq!(parse_int("42"), Ok(42));
/// assert!(parse_int("not a number").is_err());
/// ```
pub fn parse_int(s: &str) -> Result<i32, &'static str> {
    s.parse().map_err(|_| "Invalid integer")
}

// ============================================================================
// GOOD EXAMPLES unit tests
// ============================================================================


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_double() {
        assert_eq!(double(5), 10);
        assert_eq!(double(-3), -6);
        assert_eq!(double(0), 0);
    }

    #[test]
    fn test_point_distance() {
        let origin = Point::new(0.0, 0.0);
        let point = Point::new(3.0, 4.0);
        assert!((point.distance_from(&origin) - 5.0).abs() < 0.001);
    }

    #[test]
    fn test_compare() {
        assert_eq!(compare(1, 2), CompareResult::Less);
        assert_eq!(compare(2, 2), CompareResult::Equal);
        assert_eq!(compare(3, 2), CompareResult::Greater);
    }

    #[test]
    fn test_parse_int() {
        assert_eq!(parse_int("42"), Ok(42));
        assert!(parse_int("invalid").is_err());
    }
}
