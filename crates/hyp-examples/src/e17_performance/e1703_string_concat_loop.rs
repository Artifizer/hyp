/// E1703: String concatenation in loop
/// Severity: LOW
/// LLM confusion: 2 (LOW)
///
/// Description: Using `+` to concatenate strings in a loop is very inefficient because each `+`
/// creates a new String allocation. With N items, this does O(N²) work because each concatenation
/// copies all previous characters. Fix by using `push_str()` to append to the same String, or
/// use `join()` for collections.
///
/// ## The Quadratic Problem
///
/// ```text
/// for item in items {
///     result = result + item + ", ";  // O(N) copy each iteration!
/// }
/// // Total: O(N²) work for N items
/// ```
///
/// ## Why This Matters
///
/// 1. **O(N²) complexity**: 1000 items = 1,000,000 operations
/// 2. **Memory churn**: Creates and discards many Strings
/// 3. **Allocation pressure**: Many temporary allocations
/// 4. **Performance cliff**: Small inputs OK, large inputs very slow
///
/// ## The Right Solutions
///
/// ### Option 1: Use push_str
/// ```rust
/// let mut result = String::new();
/// for item in items {
///     result.push_str(item);
///     result.push_str(", ");
/// }
/// // O(N) total
/// ```
///
/// ### Option 2: Use join
/// ```rust
/// let result = items.join(", ");
/// // O(N) and very clean
/// ```
///
/// ### Option 3: Use collect
/// ```rust
/// let result: String = items.iter().collect();
/// // O(N)
/// ```
///
/// Mitigation: Use `String::push_str()` or `String::push()` to append to existing strings. Use
/// `[].join()` for joining collections. Pre-allocate with `String::with_capacity()` when you know
/// the final size. Use `#![warn(clippy::string_add_assign)]` to catch `+=` on strings.

// ============================================================================
// INEFFICIENT PATTERNS
// ============================================================================

/// PROBLEM E1703: Inefficient string concatenation with +
pub fn e1703_bad_plus_concat(items: &[&str]) -> String {
    let mut result = String::new();
    for item in items {
        // PROBLEM E1703: Inefficient string concatenation
        result = result + item + ", ";
    }
    result
}

/// PROBLEM E1703: Using format! in loop
pub fn e1703_bad_format_concat(items: &[&str]) -> String {
    let mut result = String::new();
    for item in items {
        result = format!("{}{}, ", result, item); // Creates new String each time!
    }
    result
}

/// Entry point for problem demonstration
pub fn e1703_entry() -> Result<(), Box<dyn std::error::Error>> {
    Ok(())
}

// ============================================================================
// GOOD ALTERNATIVES
// ============================================================================

/// GOOD: Use push_str
pub fn e1703_good_push_str(items: &[&str]) -> String {
    let mut result = String::new();
    for item in items {
        result.push_str(item);
        result.push_str(", ");
    }
    result
}

/// GOOD: Use push_str with capacity
pub fn e1703_good_with_capacity(items: &[&str]) -> String {
    // Estimate total length
    let total_len: usize = items.iter().map(|s| s.len() + 2).sum();
    let mut result = String::with_capacity(total_len);

    for item in items {
        result.push_str(item);
        result.push_str(", ");
    }
    result
}

/// GOOD: Use join
pub fn e1703_good_join(items: &[&str]) -> String {
    items.join(", ")
}

/// GOOD: Use collect with intersperse
pub fn e1703_good_collect(items: &[&str]) -> String {
    let mut result = String::new();
    let mut first = true;

    for item in items {
        if !first {
            result.push_str(", ");
        }
        first = false;
        result.push_str(item);
    }
    result
}

/// GOOD: Use write! macro
pub fn e1703_good_write(items: &[&str]) -> String {
    use std::fmt::Write;

    let mut result = String::new();
    for (i, item) in items.iter().enumerate() {
        if i > 0 {
            result.push_str(", ");
        }
        write!(&mut result, "{}", item).unwrap();
    }
    result
}

/// GOOD: Use iterator chain
pub fn e1703_good_iterator(items: &[&str]) -> String {
    items.to_vec()
        .join(", ")
}

/// GOOD: Use fold for complex accumulation
pub fn e1703_good_fold(items: &[&str]) -> String {
    items.iter().enumerate().fold(String::new(), |mut acc, (i, item)| {
        if i > 0 {
            acc.push_str(", ");
        }
        acc.push_str(item);
        acc
    })
}

// ============================================================================
// GOOD EXAMPLES unit tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    fn test_items() -> Vec<&'static str> {
        vec!["apple", "banana", "cherry"]
    }

    #[test]
    fn test_push_str() {
        let result = e1703_good_push_str(&test_items());
        assert_eq!(result, "apple, banana, cherry, ");
    }

    #[test]
    fn test_join() {
        let result = e1703_good_join(&test_items());
        assert_eq!(result, "apple, banana, cherry");
    }

    #[test]
    fn test_collect() {
        let result = e1703_good_collect(&test_items());
        assert_eq!(result, "apple, banana, cherry");
    }

    #[test]
    fn test_write() {
        let result = e1703_good_write(&test_items());
        assert_eq!(result, "apple, banana, cherry");
    }

    #[test]
    fn test_fold() {
        let result = e1703_good_fold(&test_items());
        assert_eq!(result, "apple, banana, cherry");
    }

    // Verify bad and good produce same result (ignoring trailing comma)
    #[test]
    fn test_equivalence() {
        let items = test_items();
        let bad = e1703_bad_plus_concat(&items);
        let good = e1703_good_push_str(&items);
        assert_eq!(bad, good);
    }
}
