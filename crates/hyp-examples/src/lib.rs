//! Problem Examples Library
//!
//! This library contains compilable examples of common Rust code problems
//! organized by category and severity.

// Allow only the bare minimum to make code compile
// We WANT clippy to report issues - that's the whole point!
#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unused_mut)]

// NOTE: We intentionally DO NOT suppress clippy warnings globally.
// This allows `make clippy` and `make kani` to catch and report issues,
// demonstrating the value of these tools.
// Individual examples use #[allow(...)] only when necessary to compile.

// Test constants for consistent example values
pub mod test_constants;

// E10* - Unsafe Code
pub mod e10_unsafe_code;

// E11* - Code Surface Complexity
pub mod e11_code_surface_complexity;

// E12* - Code Pattern Complexity
pub mod e12_code_pattern_complexity;

// E13* - Error Handling
pub mod e13_error_handling;

// E14* - Type Safety
pub mod e14_type_safety;

// E15* - Concurrency
pub mod e15_concurrency;

// E16* - Memory Safety
pub mod e16_memory_safety;

// E17* - Performance
pub mod e17_performance;

// E18* - API Design
pub mod e18_api_design;

// E19* - Code Hygiene
pub mod e19_hygiene;

// Note: Individual problem modules are available via their full paths
