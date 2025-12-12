/// E1308: Not using ? operator when appropriate
/// Severity: LOW
/// LLM confusion: 2 (LOW)
///
/// Description: This code manually handles errors with verbose match/if statements instead of
/// using the `?` operator, which is a shorthand for "if this fails, return the error to my caller".
/// The manual approach is much more verbose and harder to read. It's like writing out a full
/// try/catch block when you could just use a throws declaration. Fix by replacing manual error
/// matching with the `?` operator for cleaner, more idiomatic code.
///
/// ## The Verbosity Problem
///
/// ```text
/// // Verbose manual handling:
/// let data = match std::fs::read_to_string("file.txt") {
///     Ok(d) => d,
///     Err(e) => return Err(e),
/// };
///
/// // Clean ? operator:
/// let data = std::fs::read_to_string("file.txt")?;
/// ```
///
/// ## Why This Matters
///
/// 1. **Readability**: 4 lines become 1 line
/// 2. **Consistency**: ? is idiomatic Rust
/// 3. **Maintenance**: Less code to update when error types change
/// 4. **Focus**: Code focuses on happy path, not error plumbing
///
/// ## The Right Solutions
///
/// ### Option 1: Use ? for simple propagation
/// ```rust
/// fn read_config() -> std::io::Result<String> {
///     let content = std::fs::read_to_string("config.txt")?;
///     Ok(content)
/// }
/// ```
///
/// ### Option 2: Chain with ?
/// ```rust
/// fn process() -> Result<i32, Box<dyn std::error::Error>> {
///     let content = std::fs::read_to_string("data.txt")?;
///     let num: i32 = content.trim().parse()?;
///     Ok(num * 2)
/// }
/// ```
///
/// ### Option 3: Use ? with map_err for error conversion
/// ```rust
/// fn process() -> Result<i32, String> {
///     let content = std::fs::read_to_string("data.txt")
///         .map_err(|e| format!("Read failed: {}", e))?;
///     Ok(content.len() as i32)
/// }
/// ```
///
/// Mitigation: Use `#![warn(clippy::manual_map)]` to detect patterns that could use `?`. Learn
/// the `?` operator - it's the idiomatic way to handle errors in Rust. It automatically converts
/// error types when needed and makes code much cleaner.

// ============================================================================
// VERBOSE PATTERNS - USE ? INSTEAD
// ============================================================================

/// PROBLEM E1308: Verbose error handling instead of ?
#[allow(clippy::question_mark)]
pub fn e1308_bad_verbose_match() -> Result<i32, std::io::Error> {
    // PROBLEM E1308: Verbose error handling instead of ?
    let data = match std::fs::read_to_string("file.txt") {
        Ok(d) => d,
        Err(e) => return Err(e),
    };
    Ok(data.len() as i32)
}

/// PROBLEM E1308: if let with early return
#[allow(clippy::question_mark)]
pub fn e1308_bad_if_let_return() -> Result<String, std::io::Error> {
    let result = std::fs::read_to_string("file.txt");
    if let Err(e) = result {
        return Err(e);
    }
    Ok(result.unwrap())
}

/// PROBLEM E1308: Multiple verbose matches in sequence
#[allow(clippy::question_mark)]
pub fn e1308_bad_multiple_matches() -> Result<i32, Box<dyn std::error::Error>> {
    let content = match std::fs::read_to_string("data.txt") {
        Ok(c) => c,
        Err(e) => return Err(e.into()),
    };

    let num: i32 = match content.trim().parse() {
        Ok(n) => n,
        Err(e) => return Err(e.into()),
    };

    Ok(num)
}

/// Entry point for problem demonstration
pub fn e1308_entry() -> Result<(), Box<dyn std::error::Error>> {
    let _ = e1308_bad_verbose_match();
    Ok(())
}

// ============================================================================
// GOOD ALTERNATIVES
// ============================================================================

/// GOOD: Clean ? operator
pub fn e1308_good_question_mark() -> Result<i32, std::io::Error> {
    let data = std::fs::read_to_string("file.txt")?;
    Ok(data.len() as i32)
}

/// GOOD: Chained ? operators
pub fn e1308_good_chained() -> Result<i32, Box<dyn std::error::Error>> {
    let content = std::fs::read_to_string("data.txt")?;
    let num: i32 = content.trim().parse()?;
    Ok(num)
}

/// GOOD: ? with map_err for error context
pub fn e1308_good_with_context() -> Result<i32, String> {
    let content = std::fs::read_to_string("data.txt")
        .map_err(|e| format!("Failed to read data: {}", e))?;

    let num: i32 = content
        .trim()
        .parse()
        .map_err(|e| format!("Failed to parse number: {}", e))?;

    Ok(num)
}

/// GOOD: One-liner with ?
pub fn e1308_good_one_liner() -> std::io::Result<usize> {
    Ok(std::fs::read_to_string("file.txt")?.len())
}

/// GOOD: ? in closures (requires try blocks or helper functions)
pub fn e1308_good_in_iterator() -> Result<Vec<i32>, std::num::ParseIntError> {
    let data = "1\n2\n3\n4\n5";
    data.lines().map(|line| line.parse::<i32>()).collect()
}

/// GOOD: ? with Option using ok_or
pub fn e1308_good_option_to_result(opt: Option<i32>) -> Result<i32, &'static str> {
    let value = opt.ok_or("Value was None")?;
    Ok(value * 2)
}

/// GOOD: Complex chain with ?
pub fn e1308_good_complex_chain(path: &str) -> Result<String, Box<dyn std::error::Error>> {
    let content = std::fs::read_to_string(path)?;
    let first_line = content.lines().next().ok_or("Empty file")?;
    let parts: Vec<&str> = first_line.split(':').collect();
    let value = parts.get(1).ok_or("No colon in first line")?;
    Ok(value.trim().to_string())
}

// ============================================================================
// GOOD EXAMPLES unit tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_good_question_mark_returns_error() {
        let result = e1308_good_question_mark();
        assert!(result.is_err());
    }

    #[test]
    fn test_good_in_iterator() {
        let result = e1308_good_in_iterator();
        assert_eq!(result, Ok(vec![1, 2, 3, 4, 5]));
    }

    #[test]
    fn test_good_option_to_result_some() {
        let result = e1308_good_option_to_result(Some(21));
        assert_eq!(result, Ok(42));
    }

    #[test]
    fn test_good_option_to_result_none() {
        let result = e1308_good_option_to_result(None);
        assert_eq!(result, Err("Value was None"));
    }
}
