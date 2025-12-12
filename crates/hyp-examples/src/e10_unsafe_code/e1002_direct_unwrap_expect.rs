#![doc(test(attr(ignore)))]

/// E1002: Direct use of unwrap() and expect()
/// Severity: HIGH
/// LLM confusion: 2 (LOW)
///
/// Description: Using unwrap() and expect() crashes your program when values are None or Err.
/// Instead of gracefully handling errors and returning them to callers, your code terminates
/// abruptly. This is problematic because:
///
/// 1. **Caller has no control**: The caller cannot decide how to handle the error
/// 2. **No recovery possible**: The entire program/thread terminates
/// 3. **Poor user experience**: Users see crashes instead of helpful error messages
/// 4. **Panic cascades**: In concurrent code, one panic can cause others (e.g., mutex poisoning)
/// 5. **Resource leaks**: Resources may not be properly cleaned up on panic
///
/// ## Why unwrap() is dangerous - Common examples:
///
/// ### Example 1: Lock poisoning cascade
/// ```rust,no_run
/// use std::sync::{Arc, Mutex};
/// use std::thread;
///
/// let data = 0;
/// let mutex = Arc::new(Mutex::new(data));
/// thread::spawn(move || {
///     let guard = mutex.lock().unwrap(); // If another thread panicked while holding
///                                         // this lock, this will ALSO panic!
/// });
/// ```
///
/// ### Example 2: File operations
/// ```rust,no_run
/// let content = std::fs::read_to_string("config.txt").unwrap();
/// // Crashes if file doesn't exist, permissions denied, disk error, etc.
/// // Better: Return Result to let caller decide (retry? use default? ask user?)
/// ```
///
/// ### Example 3: Network operations
/// ```rust,no_run
/// use std::io::Read;
/// use std::net::TcpStream;
///
/// let mut stream = TcpStream::connect("127.0.0.1:80").unwrap();
/// let mut response = String::new();
/// stream.read_to_string(&mut response).unwrap();
/// // Crashes on network timeout, DNS failure, connection refused...
/// // In a server: one bad request crashes the whole service!
/// ```
///
/// ### Example 4: User input parsing
/// ```rust,no_run
/// let args: Vec<String> = std::env::args().collect();
/// let port: u16 = args[1].parse().unwrap();
/// // Crashes if user provides invalid input
/// // Better: Show helpful error message and usage instructions
/// ```
///
/// Mitigation: Use `#![warn(clippy::unwrap_used, clippy::expect_used)]` to catch these.
/// Prefer `?` operator, `if let`, `match`, or combinators like `unwrap_or_default()`,
/// `unwrap_or_else()`, `ok_or()`. Only use unwrap() when you can PROVE the value exists.

/// PROBLEM E1002: Direct unwrap - crashes on None
pub fn e1002_bad_option_unwrap(data: Option<i32>) -> i32 {
    data.unwrap() // What if data is None? Program crashes!
}

/// PROBLEM E1002: Direct unwrap on Result - crashes on Err
pub fn e1002_bad_result_unwrap(content_file: String) -> String {
    
    // What if file doesn't exist? Permission denied? Disk full?
    // Program crashes with unhelpful message!
    std::fs::read_to_string(content_file).unwrap()
}

/// PROBLEM E1002: expect() is just unwrap() with a message - still crashes!
pub fn e1002_bad_expect_still_crashes() -> i32 {
    let value: Result<i32, &str> = Err("something went wrong");
    value.expect("this will crash") // Crashes! expect() doesn't recover, just adds a message
}

/// PROBLEM E1002: Chain of unwraps - any one can crash
pub fn e1002_bad_chained_unwraps() {
    let nested: Option<Option<i32>> = Some(Some(42));
    let _value = nested.unwrap().unwrap(); // TWO potential crash points!
}

/// PROBLEM E1002: unwrap() in iterator - crashes mid-processing
pub fn e1002_bad_unwrap_in_iterator() {
    let items = ["1", "2", "not_a_number", "4"];
    let _numbers: Vec<i32> = items
        .iter()
        .map(|s| s.parse().unwrap()) // Crashes on "not_a_number"!
        .collect();
}

/// PROBLEM E1002: unwrap() hides the actual error
pub fn e1002_bad_unwrap_hides_error() {
    let json = r#"{"name": invalid}"#; // Invalid JSON
    let _parsed: serde_json::Value = serde_json::from_str(json).unwrap();
    // Crash! But what was wrong? Where? What input caused it?
    // With proper error handling, you could show: "Parse error at line 1, column 10"
}

/// Entry point for problem demonstration
pub fn e1002_entry() -> Result<(), Box<dyn std::error::Error>> {
    // These would crash - demonstrating the problem
    let data: Option<i32> = Some(42);
    e1002_bad_option_unwrap(data);

    // let content_file = "config.txt".to_string();
    // e1002_bad_result_unwrap(content_file);
    Ok(())
}

// ============================================================================
// GOOD EXAMPLES - How to handle errors properly
// ============================================================================

/// GOOD: Return Result to let caller decide
fn good_return_result() -> Result<i32, &'static str> {
    let data: Option<i32> = None;
    data.ok_or("value was not present")
}

/// GOOD: Use ? operator for propagation
fn good_question_mark() -> std::io::Result<String> {
    let content = std::fs::read_to_string("config.txt")?;
    // Error automatically propagated to caller with full context!
    Ok(content)
}

/// GOOD: Provide defaults for optional values
fn good_unwrap_or_default() -> i32 {
    let data: Option<i32> = None;
    data.unwrap_or(0) // Safe! Returns 0 if None
}

/// GOOD: Use if let for conditional handling
fn good_if_let() {
    let data: Option<i32> = Some(42);
    if let Some(value) = data {
        println!("Got value: {}", value);
    } else {
        println!("No value present");
    }
}

/// GOOD: Use match for exhaustive handling
fn good_match_handling() -> Result<(), String> {
    let result: Result<i32, &str> = Err("failed");
    match result {
        Ok(value) => {
            println!("Success: {}", value);
            Ok(())
        }
        Err(e) => {
            // Log, retry, or return error - YOU decide!
            Err(format!("Operation failed: {}", e))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_good_alternatives() {
        // These should work without panicking
        assert_eq!(good_unwrap_or_default(), 0);
        assert!(good_return_result().is_err());
        good_if_let();
    }

    #[test]
    #[should_panic]
    fn test_unwrap_crashes() {
        let none: Option<i32> = None;
        none.unwrap();
    }
}
