/// E1712: Expensive operations inside loops
/// Severity: MEDIUM
/// LLM confusion: 2 (LOW)
///
/// Description: Expensive initialization like Regex compilation, file operations, or
/// creating synchronization primitives inside loops wastes resources. These operations
/// should be hoisted outside the loop and reused.
///
/// Mitigation: Move expensive initialization (Regex::new, File::open, Mutex::new, etc.)
/// outside the loop. Use lazy_static or once_cell for truly static patterns.

use std::fs::File;
use std::io::{BufRead, BufReader};
use std::sync::Mutex;

/// PROBLEM E1712: Compiling regex on every iteration
pub fn e1712_bad_regex_in_loop(lines: &[String], pattern: &str) -> usize {
    let mut count = 0;
    for line in lines {
        // Regex compilation is VERY expensive!
        let re = regex::Regex::new(pattern).unwrap();
        if re.is_match(line) {
            count += 1;
        }
    }
    count
}

/// PROBLEM E1712: Opening file in loop
pub fn e1712_bad_file_open_in_loop(keys: &[&str]) -> Vec<String> {
    let mut values = Vec::new();
    for key in keys {
        // Opens file N times!
        if let Ok(file) = File::open("config.txt") {
            let reader = BufReader::new(file);
            for line in reader.lines().map_while(Result::ok) {
                if line.starts_with(key) {
                    values.push(line);
                }
            }
        }
    }
    values
}

/// PROBLEM E1712: Creating Mutex inside loop
#[allow(clippy::vec_init_then_push)]
pub fn e1712_bad_mutex_in_loop(data: Vec<i32>) -> Vec<Mutex<i32>> {
    let mut mutexes = Vec::new();
    for item in data {
        // Creates new Mutex for each item - usually wrong!
        let mutex = Mutex::new(item);
        mutexes.push(mutex);
    }
    mutexes
}

/// PROBLEM E1712: String allocation in tight loop
pub fn e1712_bad_string_alloc_in_loop(numbers: &[i32]) -> Vec<String> {
    let mut results = Vec::new();
    for n in numbers {
        // Creates new format string each time
        let prefix = String::from("Number: ");
        results.push(format!("{}{}", prefix, n));
    }
    results
}

pub fn e1712_entry() -> Result<(), Box<dyn std::error::Error>> {
    let lines = vec!["a@b.com".to_string()];
    let _ = e1712_bad_regex_in_loop(&lines, "@");
    let _ = e1712_bad_string_alloc_in_loop(&[1, 2, 3]);
    Ok(())
}

// ============================================================================
// GOOD EXAMPLES - Hoist expensive operations
// ============================================================================

/// GOOD: Compile regex once outside loop
pub fn e1712_good_regex(lines: &[String], pattern: &str) -> usize {
    let re = regex::Regex::new(pattern).unwrap(); // Compile once!
    let mut count = 0;
    for line in lines {
        if re.is_match(line) {
            count += 1;
        }
    }
    count
}

/// GOOD: Read file once, iterate keys
pub fn e1712_good_file_read(keys: &[&str]) -> Vec<String> {
    // Read file once
    let file = match File::open("config.txt") {
        Ok(f) => f,
        Err(_) => return vec![],
    };
    let reader = BufReader::new(file);
    let lines: Vec<String> = reader.lines().map_while(Result::ok).collect();

    // Then iterate
    let mut values = Vec::new();
    for key in keys {
        for line in &lines {
            if line.starts_with(key) {
                values.push(line.clone());
            }
        }
    }
    values
}

/// GOOD: Pre-allocate and use const strings
pub fn e1712_good_format(numbers: &[i32]) -> Vec<String> {
    const PREFIX: &str = "Number: ";
    let mut results = Vec::with_capacity(numbers.len()); // Pre-allocate!

    for n in numbers {
        results.push(format!("{}{}", PREFIX, n));
    }
    results
}

/// GOOD: Use iterator with map for clarity
pub fn e1712_good_idiomatic(numbers: &[i32]) -> Vec<String> {
    numbers
        .iter()
        .map(|n| format!("Number: {}", n))
        .collect()
}

/// GOOD: Single shared Mutex when actually sharing data
pub fn e1712_good_shared_mutex() -> Mutex<i32> {
    // Create ONE mutex to protect shared state
    Mutex::new(0)
}

// ============================================================================
// GOOD EXAMPLES unit tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_regex_good() {
        let lines = vec![
            "hello@example.com".to_string(),
            "not an email".to_string(),
            "another@test.org".to_string(),
        ];
        let count = e1712_good_regex(&lines, r"@.*\.");
        assert_eq!(count, 2);
    }

    #[test]
    fn test_format_good() {
        let numbers = vec![1, 2, 3];
        let results = e1712_good_format(&numbers);
        assert_eq!(results[0], "Number: 1");
    }
}
