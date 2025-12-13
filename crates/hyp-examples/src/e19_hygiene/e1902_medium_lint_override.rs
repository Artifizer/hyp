/// E1902: Medium Lint Override Detection
/// Severity: MEDIUM
/// LLM confusion: 3 (MEDIUM)
///
/// Description: Detects when code uses `#[allow(...)]` attributes to override
/// medium-severity Clippy lints related to performance, type safety, and complexity.
///
/// ## Why This Matters
///
/// 1. **Performance issues**: Hiding inefficiencies leads to slower code
/// 2. **Type safety**: Integer overflows and lossy casts cause bugs
/// 3. **Code complexity**: Cognitive complexity warnings exist for a reason
/// 4. **Maintenance burden**: Complex code is harder to understand and modify
///
/// ## The Right Solutions
///
/// ### Option 1: Simplify complex functions
/// ```rust
/// // Instead of suppressing cognitive_complexity:
/// // #[allow(clippy::cognitive_complexity)]
/// // fn complex() { /* 50 nested ifs */ }
///
/// // Refactor into smaller functions
/// fn process_step_1() { }
/// fn process_step_2() { }
/// ```
///
/// ### Option 2: Use checked arithmetic
/// ```rust
/// fn safe_add(a: i32, b: i32) -> Option<i32> {
///     a.checked_add(b)
/// }
/// ```

// ============================================================================
// PROBLEMATIC PATTERNS
// ============================================================================

/// PROBLEM E1902: Suppressing cognitive_complexity warning
#[allow(clippy::cognitive_complexity)]
pub fn e1902_bad_complexity_override(x: i32, y: i32, z: i32) -> i32 {
    // This function is too complex but the warning is hidden
    if x > 0 {
        if y > 0 {
            if z > 0 {
                x + y + z
            } else if z < -10 {
                x + y - z
            } else {
                x + y
            }
        } else if y < -5 {
            if z > 0 {
                x - y + z
            } else {
                x - y
            }
        } else {
            x
        }
    } else {
        0
    }
}

/// PROBLEM E1902: Suppressing as_conversions warning
#[allow(clippy::as_conversions)]
pub fn e1902_bad_as_conversions(x: i64) -> i32 {
    // Potentially lossy conversion hidden
    x as i32
}

/// PROBLEM E1902: Suppressing clone_on_copy warning
#[allow(clippy::clone_on_copy)]
pub fn e1902_bad_clone_on_copy() -> i32 {
    let x: i32 = 42;
    x.clone() // Unnecessary clone of Copy type
}

/// PROBLEM E1902: Suppressing too_many_arguments warning
#[allow(clippy::too_many_arguments)]
pub fn e1902_bad_too_many_args(
    a: i32,
    b: i32,
    c: i32,
    d: i32,
    e: i32,
    f: i32,
    g: i32,
    h: i32,
) -> i32 {
    a + b + c + d + e + f + g + h
}

/// Entry point for problem demonstration
pub fn e1902_entry() -> Result<(), Box<dyn std::error::Error>> {
    let _ = e1902_bad_complexity_override(1, 2, 3);
    let _ = e1902_bad_as_conversions(100);
    let _ = e1902_bad_clone_on_copy();
    let _ = e1902_bad_too_many_args(1, 2, 3, 4, 5, 6, 7, 8);
    Ok(())
}

// ============================================================================
// GOOD ALTERNATIVES
// ============================================================================

/// GOOD: Simplified function with clear logic
pub fn e1902_good_simple_logic(x: i32, y: i32, z: i32) -> i32 {
    let base = x.max(0);
    let adjustment = if y > 0 { y } else { 0 };
    let modifier = if z > 0 { z } else { 0 };
    base + adjustment + modifier
}

/// GOOD: Safe numeric conversion
pub fn e1902_good_safe_conversion(x: i64) -> Result<i32, &'static str> {
    i32::try_from(x).map_err(|_| "value out of range")
}

/// GOOD: No unnecessary clone
pub fn e1902_good_no_clone() -> i32 {
    let x: i32 = 42;
    x // Copy types don't need clone
}

/// GOOD: Use a builder or config struct instead of many arguments
pub struct ProcessConfig {
    pub a: i32,
    pub b: i32,
    pub c: i32,
    pub d: i32,
}

pub fn e1902_good_config_struct(config: ProcessConfig) -> i32 {
    config.a + config.b + config.c + config.d
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_good_simple_logic() {
        assert_eq!(e1902_good_simple_logic(1, 2, 3), 6);
        assert_eq!(e1902_good_simple_logic(-1, -2, -3), 0);
    }

    #[test]
    fn test_good_safe_conversion() {
        assert!(e1902_good_safe_conversion(100).is_ok());
        assert!(e1902_good_safe_conversion(i64::MAX).is_err());
    }

    #[test]
    fn test_good_config_struct() {
        let config = ProcessConfig {
            a: 1,
            b: 2,
            c: 3,
            d: 4,
        };
        assert_eq!(e1902_good_config_struct(config), 10);
    }
}
