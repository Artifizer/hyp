/// E1304: Using unwrap() in error paths
/// Severity: HIGH
/// LLM confusion: 3 (MED)
///
/// Description: This function returns a Result type (indicating it can handle errors gracefully),
/// but internally uses `unwrap()` which crashes the program if an error occurs. This defeats the
/// entire purpose of returning Result. It's like a function that promises to handle errors but
/// actually just crashes when things go wrong. Fix by using the `?` operator to pass errors up
/// to the caller instead of crashing.
///
/// ## The Broken Promise Problem
///
/// ```text
/// Function signature: fn process() -> Result<i32, String>
///     ↓
/// Caller expects: "I can handle any error this returns"
///     ↓
/// Reality: .parse().unwrap() inside the function
///     ↓
/// Invalid input → PANIC! Caller never gets a chance to handle it
/// ```
///
/// ## Why This Matters
///
/// 1. **Broken contract**: Signature promises Result but delivers panics
/// 2. **Caller confusion**: Callers wrap in try/catch expecting graceful errors
/// 3. **Cascading failures**: One bad input crashes entire application
/// 4. **Untestable error paths**: Can't test error handling if it panics
///
/// ## The Right Solutions
///
/// ### Option 1: Use ? operator throughout
/// ```rust
/// fn process(data: &str) -> Result<i32, Box<dyn std::error::Error>> {
///     let content = std::fs::read_to_string(data)?;
///     let num: i32 = content.trim().parse()?;
///     Ok(num)
/// }
/// ```
///
/// ### Option 2: Use map_err for error conversion
/// ```rust
/// fn process(data: &str) -> Result<i32, String> {
///     let content = std::fs::read_to_string(data)
///         .map_err(|e| format!("Read failed: {}", e))?;
///     let num: i32 = content.trim().parse()
///         .map_err(|e| format!("Parse failed: {}", e))?;
///     Ok(num)
/// }
/// ```
///
/// Mitigation: Use `#![warn(clippy::unwrap_in_result)]` to catch unwrap/expect in functions
/// returning Result. Use the `?` operator to propagate errors, or use `map_err()` to convert
/// error types when needed.

// ============================================================================
// DANGEROUS PATTERNS - NEVER DO THIS
// ============================================================================

/// PROBLEM E1304: unwrap() in a function that returns Result
pub fn e1304_bad_unwrap_in_result() -> Result<i32, String> {
    let data = std::fs::read_to_string("data.txt").map_err(|e| e.to_string())?;

    // PROBLEM E1304: unwrap() in a function that returns Result
    // PROBLEM E1002: direct unwrap/expect
    let num: i32 = data.trim().parse().unwrap();
    Ok(num)
}

/// PROBLEM E1304: Multiple unwraps in error-returning function
pub fn e1304_bad_multiple_unwraps(input: &str) -> Result<String, std::io::Error> {
    let path = std::path::Path::new(input);
    let content = std::fs::read_to_string(path)?;

    // PROBLEM E1304: These unwraps defeat the purpose of returning Result
    let first_line = content.lines().next().unwrap();
    let trimmed = first_line.split(':').next().unwrap();

    Ok(trimmed.to_string())
}

/// PROBLEM E1304: expect() is just as bad
pub fn e1304_bad_expect_in_result() -> Result<Vec<i32>, String> {
    let data = "1,2,3,invalid";

    // PROBLEM E1304: expect() still panics
    let numbers: Vec<i32> = data
        .split(',')
        .map(|s| s.parse().expect("should be a number"))
        .collect();

    Ok(numbers)
}

/// Entry point for problem demonstration
pub fn e1304_entry() -> Result<(), Box<dyn std::error::Error>> {
    let _ = e1304_bad_unwrap_in_result();
    Ok(())
}

// ============================================================================
// GOOD ALTERNATIVES
// ============================================================================

/// GOOD: Use ? operator consistently
pub fn e1304_good_question_mark() -> Result<i32, Box<dyn std::error::Error>> {
    let data = std::fs::read_to_string("data.txt")?;
    let num: i32 = data.trim().parse()?;
    Ok(num)
}

/// GOOD: Use map_err for custom error messages
pub fn e1304_good_map_err(input: &str) -> Result<String, String> {
    let path = std::path::Path::new(input);
    let content = std::fs::read_to_string(path).map_err(|e| format!("Read error: {}", e))?;

    let first_line = content
        .lines()
        .next()
        .ok_or_else(|| "File is empty".to_string())?;

    let trimmed = first_line
        .split(':')
        .next()
        .ok_or_else(|| "No colon found in first line".to_string())?;

    Ok(trimmed.to_string())
}

/// GOOD: Use ok_or/ok_or_else for Option to Result conversion
pub fn e1304_good_option_to_result(data: &str) -> Result<i32, String> {
    let first = data.lines().next().ok_or("Empty input")?;
    let num: i32 = first.parse().map_err(|e| format!("Parse error: {}", e))?;
    Ok(num)
}

/// GOOD: Collect Results properly
pub fn e1304_good_collect_results(data: &str) -> Result<Vec<i32>, String> {
    data.split(',')
        .map(|s| {
            s.trim()
                .parse::<i32>()
                .map_err(|e| format!("Failed to parse '{}': {}", s, e))
        })
        .collect()
}

/// GOOD: Use and_then for chained fallible operations
pub fn e1304_good_and_then(path: &str) -> Result<i32, Box<dyn std::error::Error>> {
    std::fs::read_to_string(path)
        .map_err(|e| e.into())
        .and_then(|content| {
            content
                .trim()
                .parse::<i32>()
                .map_err(|e| e.into())
        })
}

// ============================================================================
// GOOD EXAMPLES unit tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_good_option_to_result_success() {
        let result = e1304_good_option_to_result("42\nmore lines");
        assert_eq!(result, Ok(42));
    }

    #[test]
    fn test_good_option_to_result_empty() {
        let result = e1304_good_option_to_result("");
        assert_eq!(result, Err("Empty input".to_string()));
    }

    #[test]
    fn test_good_option_to_result_invalid() {
        let result = e1304_good_option_to_result("not_a_number");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Parse error"));
    }

    #[test]
    fn test_good_collect_results_success() {
        let result = e1304_good_collect_results("1, 2, 3, 4");
        assert_eq!(result, Ok(vec![1, 2, 3, 4]));
    }

    #[test]
    fn test_good_collect_results_failure() {
        let result = e1304_good_collect_results("1, 2, invalid, 4");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("invalid"));
    }
}
