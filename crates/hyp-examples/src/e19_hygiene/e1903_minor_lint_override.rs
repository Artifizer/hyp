/// E1903: Minor Lint Override Detection
/// Severity: LOW
/// LLM confusion: 2 (LOW)
///
/// Description: Detects when code uses `#[allow(...)]` attributes to override
/// minor/stylistic Clippy lints. While these may have legitimate overrides,
/// tracking them helps maintain code consistency.
///
/// ## Why This Matters
///
/// 1. **Code consistency**: Style lints ensure uniform codebase
/// 2. **Documentation**: Missing docs make code harder to understand
/// 3. **Maintainability**: Idiomatic code is easier to maintain
/// 4. **Review awareness**: Track where style rules are bypassed
///
/// ## When Overrides May Be Acceptable
///
/// - Test files with helper functions
/// - Generated code
/// - FFI bindings matching external naming conventions
/// - Temporary work-in-progress code (with TODO comments)

// ============================================================================
// PROBLEMATIC PATTERNS
// ============================================================================

/// PROBLEM E1903: Suppressing wildcard_imports warning
#[allow(clippy::wildcard_imports)]
pub mod e1903_bad_glob {
    pub use std::collections::*;
    // Glob imports make it unclear what's being used
}

/// PROBLEM E1903: Suppressing todo warning
#[allow(clippy::todo)]
pub fn e1903_bad_todo_override(x: i32) {
    if x == 0 {
        todo!("This TODO is hidden from linters");
    }
}

/// PROBLEM E1903: Suppressing missing_docs warning
#[allow(clippy::missing_docs_in_private_items)]
fn e1903_bad_undocumented() {
    // This function has no documentation but the warning is suppressed
}

/// PROBLEM E1903: Suppressing shadow warnings
#[allow(clippy::shadow_unrelated)]
pub fn e1903_bad_shadow_override() {
    let x = 5;
    let x = "now I'm a string"; // Confusing shadow
    let _ = x;
}

/// PROBLEM E1903: Suppressing print_stdout warning
#[allow(clippy::print_stdout)]
pub fn e1903_bad_print_override() {
    println!("Debug output hidden from linters");
}

/// Entry point for problem demonstration
pub fn e1903_entry() -> Result<(), Box<dyn std::error::Error>> {
    e1903_bad_todo_override(1);
    e1903_bad_undocumented();
    e1903_bad_shadow_override();
    e1903_bad_print_override();
    Ok(())
}

// ============================================================================
// GOOD ALTERNATIVES
// ============================================================================

/// GOOD: Explicit imports instead of glob
pub mod e1903_good_explicit {
    pub use std::collections::{HashMap, HashSet, VecDeque};
}

/// GOOD: Proper documentation
/// Calculates the sum of two numbers.
///
/// # Arguments
/// * `a` - First number
/// * `b` - Second number
///
/// # Returns
/// The sum of a and b
pub fn e1903_good_documented(a: i32, b: i32) -> i32 {
    a + b
}

/// GOOD: Clear variable names instead of shadowing
pub fn e1903_good_no_shadow() {
    let count = 5;
    let count_str = count.to_string();
    let _ = count_str;
}

/// GOOD: Use proper logging instead of println
pub fn e1903_good_logging(message: &str) {
    // In production, use a proper logging framework:
    // log::info!("{}", message);
    // For now, use eprintln for errors only
    eprintln!("LOG: {}", message);
}

/// GOOD: Use unimplemented for stub functions with proper comment
///
/// This function will be implemented in phase 2.
pub fn e1903_good_stub_with_docs() -> i32 {
    // TODO(phase-2): Implement actual logic
    0
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_good_documented() {
        assert_eq!(e1903_good_documented(2, 3), 5);
    }

    #[test]
    fn test_good_stub() {
        assert_eq!(e1903_good_stub_with_docs(), 0);
    }

    // Note: In test modules, some style lints may be legitimately overridden
    #[allow(clippy::wildcard_imports)]
    use std::collections::*;

    #[test]
    fn test_uses_hashmap() {
        let mut map: HashMap<i32, i32> = HashMap::new();
        map.insert(1, 2);
        assert_eq!(map.get(&1), Some(&2));
    }
}
