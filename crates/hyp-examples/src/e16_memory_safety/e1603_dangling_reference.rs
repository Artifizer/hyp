/// E1603: Dangling reference
/// Severity: HIGH
/// LLM confusion: 4 (HIGH)
///
/// Description: A dangling reference points to memory that no longer contains valid data. This
/// code tries to return a reference to a local variable, which will be destroyed when the function
/// returns. The reference would point to invalid stack memory. Rust's compiler prevents this at
/// compile time. Fix by returning owned data or using proper lifetime annotations.
///
/// ## The Dangling Reference Problem
///
/// ```text
/// fn bad() -> &i32 {
///     let x = 42;
///     &x  // ERROR: x will be dropped, reference would dangle
/// }
/// ```
///
/// ## Why This Matters
///
/// 1. **Compile error**: Rust prevents this (good!)
/// 2. **Would cause UB**: If allowed, would access invalid memory
/// 3. **Common mistake**: Especially for those from GC languages
/// 4. **Lifetime learning**: Key to understanding Rust
///
/// ## The Right Solutions
///
/// ### Option 1: Return owned data
/// ```rust
/// fn good() -> i32 {
///     let x = 42;
///     x  // Return the value, not a reference
/// }
/// ```
///
/// ### Option 2: Return reference to input
/// ```rust
/// fn good<'a>(data: &'a [i32]) -> &'a i32 {
///     &data[0]  // Reference lives as long as input
/// }
/// ```
///
/// ### Option 3: Use static lifetime for constants
/// ```rust
/// fn good() -> &'static str {
///     "hello"  // String literals are 'static
/// }
/// ```
///
/// Mitigation: Rust's borrow checker prevents dangling references at compile time. If you get
/// lifetime errors, don't fight the compiler - restructure to return owned data or use proper
/// lifetime relationships. Never try to return references to local variables.

// ============================================================================
// PATTERNS THAT RUST PREVENTS
// ============================================================================

/// PROBLEM E1603: Returning reference to local variable (won't compile)
/// Note: This returns a static reference to demonstrate - the commented code shows the problem
pub fn e1603_bad_dangling() -> &'static i32 {
    let _x = 42;
    // PROBLEM E1603: Returning reference to local variable (won't compile)
    // &x  // This would not compile!
    &0 // Placeholder - static reference is OK
}

/// Entry point for problem demonstration
pub fn e1603_entry() -> Result<(), Box<dyn std::error::Error>> {
    Ok(())
}

// ============================================================================
// GOOD ALTERNATIVES
// ============================================================================

/// GOOD: Return owned data
pub fn e1603_good_return_owned() -> i32 {
    
    42 // Return the value itself
}

/// GOOD: Return reference to input (with lifetime)
pub fn e1603_good_return_input_ref(data: &[i32]) -> Option<&i32> {
    data.first()
}

/// GOOD: Return static reference
pub fn e1603_good_static_ref() -> &'static str {
    "hello" // String literals have 'static lifetime
}

/// GOOD: Return Box for heap allocation
pub fn e1603_good_return_box() -> Box<i32> {
    let x = 42;
    Box::new(x)
}

/// GOOD: Use String instead of &str for owned strings
pub fn e1603_good_return_string() -> String {
    let local = "hello";
    local.to_string() // Convert to owned String
}

/// GOOD: Take reference as parameter and return derived reference
pub fn e1603_good_derive_reference(s: &str) -> &str {
    &s[0..1] // Reference derived from input
}

/// GOOD: Use Option when reference might not exist
pub fn e1603_good_optional_ref(data: &[i32], idx: usize) -> Option<&i32> {
    data.get(idx)
}

// ============================================================================
// GOOD EXAMPLES unit tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_return_owned() {
        assert_eq!(e1603_good_return_owned(), 42);
    }

    #[test]
    fn test_input_ref() {
        let data = vec![1, 2, 3];
        assert_eq!(e1603_good_return_input_ref(&data), Some(&1));
    }

    #[test]
    fn test_static_ref() {
        assert_eq!(e1603_good_static_ref(), "hello");
    }

    #[test]
    fn test_return_box() {
        let b = e1603_good_return_box();
        assert_eq!(*b, 42);
    }

    #[test]
    fn test_derive_reference() {
        assert_eq!(e1603_good_derive_reference("hello"), "h");
    }

    #[test]
    fn test_optional_ref() {
        let data = vec![1, 2, 3];
        assert_eq!(e1603_good_optional_ref(&data, 1), Some(&2));
        assert_eq!(e1603_good_optional_ref(&data, 10), None);
    }
}
