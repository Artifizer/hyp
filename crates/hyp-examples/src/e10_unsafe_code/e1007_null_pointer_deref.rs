/// E1007: Dereferencing null pointer
/// Severity: HIGH
/// LLM confusion: 4 (HIGH)
///
/// Description: Dereferencing a null pointer (address 0) is undefined behavior and will typically
/// crash the program immediately. This is one of the most common bugs in C/C++ code. In Rust, you
/// have to explicitly use unsafe code to create and dereference null pointers, which is why this
/// is a serious error - you're bypassing safety checks to do something dangerous. Never dereference
/// a pointer without checking if it's null first.
///
/// Mitigation: Never create null pointers in safe code. If working with FFI that might return null,
/// check for null before dereferencing. Use `Option<NonNull<T>>` to make nullability explicit.
/// Use `ptr.is_null()` to check before dereferencing. Prefer safe Rust references over raw pointers.

pub fn e1007_bad_null_pointer_deref(input: i32) {
    let ptr: *const i32 = std::ptr::null();

    if input > 0 {
        // PROBLEM E1003: Direct use of unsafe code
        unsafe {
            // PROBLEM E1904: No safety documentation
            // PROBLEM E1007: Dereferencing null pointer (undefined behavior)
            let _value = *ptr;
        }
    }
}

pub fn e1007_entry() -> Result<(), Box<dyn std::error::Error>> {
    e1007_bad_null_pointer_deref(0);
    Ok(())
}

// ============================================================================
// GOOD EXAMPLES - Proper alternatives
// ============================================================================

/// GOOD: Use Option<&T> instead of nullable pointers
pub fn e1007_good_use_option(value: Option<&i32>) -> i32 {
    const DEFAULT_VALUE: i32 = 0;
    match value {
        Some(v) => *v,
        None => DEFAULT_VALUE, // Handle the None case with named constant
    }
}

/// GOOD: Use Option with unwrap_or for safe default handling
pub fn e1007_good_option_with_default(value: Option<i32>) -> i32 {
    const DEFAULT: i32 = 0;
    value.unwrap_or(DEFAULT) // Safe - no null pointers involved
}

/// GOOD: Use Result for operations that might fail
pub fn e1007_good_result_pattern(data: &[i32], index: usize) -> Result<i32, &'static str> {
    data.get(index).copied().ok_or("Index out of bounds")
}

/// GOOD: Use references which are never null in safe Rust
pub fn e1007_good_use_references(data: &[i32]) -> usize {
    data.len() // Safe operations on references
}

// ============================================================================
// GOOD EXAMPLES unit tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn e1007_good_use_option_handles_none() {
        const EXPECTED_DEFAULT: i32 = 0;
        assert_eq!(e1007_good_use_option(None), EXPECTED_DEFAULT);
    }

    #[test]
    fn e1007_good_option_with_default_uses_default() {
        const EXPECTED: i32 = 0;
        assert_eq!(e1007_good_option_with_default(None), EXPECTED);
    }

    #[test]
    fn e1007_good_result_pattern_returns_value() {
        const FIRST: i32 = 1;
        const SECOND: i32 = 2;
        const INDEX: usize = 0;
        let data = [FIRST, SECOND];
        assert_eq!(e1007_good_result_pattern(&data, INDEX), Ok(FIRST));
    }

    #[test]
    fn e1007_good_use_references_returns_length() {
        const EXPECTED_LENGTH: usize = 3;
        const FIRST: i32 = 1;
        const SECOND: i32 = 2;
        const THIRD: i32 = 3;
        let data = [FIRST, SECOND, THIRD];
        assert_eq!(e1007_good_use_references(&data), EXPECTED_LENGTH);
    }
}
