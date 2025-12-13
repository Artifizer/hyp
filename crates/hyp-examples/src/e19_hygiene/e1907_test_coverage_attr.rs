/// E1907: Test modules missing coverage attribute
/// Severity: HIGH
/// LLM confusion: 2 (LOW)
///
/// Description: Detects test modules that have `#[cfg(test)]` but are missing
/// the `#[cfg_attr(coverage_nightly, coverage(off))]` attribute for excluding
/// tests from code coverage instrumentation.
///
/// ## Why This Matters
///
/// 1. **Accurate coverage metrics**: Including test code in coverage reports inflates metrics
/// 2. **Focus on production code**: Coverage should measure how well production code is tested
/// 3. **Consistent configuration**: Ensures all test modules follow the same coverage exclusion pattern
/// 4. **Configurable feature**: The coverage feature name is configurable (default: "coverage_nightly")
///
/// ## The Right Solutions
///
/// ### Option 1: Add both attributes together
/// ```rust
/// #[cfg(test)]
/// #[cfg_attr(coverage_nightly, coverage(off))]
/// mod tests {
///     #[test]
///     fn test_example() {
///         assert!(true);
///     }
/// }
/// ```
///
/// ### Option 2: Use custom coverage feature name
/// Configure in Hyp.toml:
/// ```toml
/// [checkers.e1907_test_coverage_attr]
/// enabled = true
/// coverage_feature = "my_coverage_feature"
/// ```
///
/// Then use:
/// ```rust
/// #[cfg(test)]
/// #[cfg_attr(my_coverage_feature, coverage(off))]
/// mod tests {
///     // tests here
/// }
/// ```
///
/// Mitigation: Configure E1907 in Hyp.toml to use your project's coverage feature name.

// ============================================================================
// PROBLEMATIC PATTERNS
// ============================================================================

/// PROBLEM E1907: Test module without coverage attribute
/// This module will be included in coverage reports unnecessarily.
#[cfg(test)]
mod e1907_bad_missing_coverage_attr {
    #[test]
    fn test_without_coverage_exclusion() {
        assert!(true);
    }
}

/// PROBLEM E1907: Nested test module without coverage attribute
mod e1907_outer_module {
    #[cfg(test)]
    mod e1907_bad_nested_tests {
        #[test]
        fn nested_test() {
            assert!(1 + 1 == 2);
        }
    }
}

// ============================================================================
// GOOD ALTERNATIVES
// ============================================================================

/// GOOD: Test module with proper coverage attribute
#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod e1907_good_with_coverage_attr {
    #[test]
    fn test_with_coverage_exclusion() {
        assert!(true);
    }
}

/// GOOD: Non-test module (doesn't need coverage attribute)
mod e1907_good_regular_module {
    pub fn helper_function() -> i32 {
        42
    }
}

/// GOOD: Test module with custom coverage feature
#[cfg(test)]
#[cfg_attr(custom_coverage, coverage(off))]
mod e1907_good_custom_feature {
    // This is OK if the project uses "custom_coverage" as the coverage feature
    #[test]
    fn test_custom_feature() {
        assert!(true);
    }
}

// ============================================================================
// Entry point for problem demonstration
// ============================================================================

pub fn e1907_entry() -> Result<(), Box<dyn std::error::Error>> {
    // The problems are in the test modules above which have #[cfg(test)]
    // but no #[cfg_attr(coverage_nightly, coverage(off))]
    Ok(())
}

// ============================================================================
// GOOD EXAMPLES unit tests
// ============================================================================

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use super::*;

    #[test]
    fn test_entry() {
        assert!(e1907_entry().is_ok());
    }

    #[test]
    fn test_regular_module() {
        assert_eq!(e1907_good_regular_module::helper_function(), 42);
    }
}
