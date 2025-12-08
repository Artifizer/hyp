# Adding a New Checker to Hyp

This guide walks you through adding a new checker end-to-end. We'll use E1002 (Direct unwrap/expect) as a reference example.

## Overview

Adding a checker requires changes in **three crates**:
1. **`hyp-checks-generic`** - The checker implementation
2. **`hyp-examples`** - Compilable examples demonstrating the problem
3. **`hyp-examples-cli`** - Compilable CLI tool ensuring the problem examples can be compiled and executed w/o visible problems

## Step-by-Step Guide

### Step 1: Choose Your Checker Code

Checker codes follow the pattern `E{category}{number}`:
- **E10xx** - Unsafe Code (panics, unwrap, unsafe blocks, FFI)
- **E11xx** - Code Surface Complexity (long functions, many parameters)
- **E12xx** - Code Pattern Complexity (complex generics, lifetimes)
- **E13xx** - Error Handling
- **E14xx** - Type Safety
- **E15xx** - Concurrency
- **E16xx** - Memory Safety
- **E17xx** - Performance
- **E18xx** - API Design

Pick the next available number in the appropriate category.

### Step 2: Create the Checker Implementation

Create a new file in `crates/hyp-checks-generic/src/checkers/eXX/` (where XX is the category):

```bash
touch crates/hyp-checks-generic/src/checkers/e10/e1002_direct_unwrap_expect.rs
```

Use the `define_checker!` macro. Here's the structure:

```rust
//! E1002: Direct use of unwrap() and expect()
//!
//! Detects calls to unwrap() and expect() which panic on None/Err.

use crate::{checker::Checker, define_checker, violation::Violation};
use syn::{spanned::Spanned, visit::Visit};

define_checker! {
    /// Checker for E1002: Direct unwrap/expect calls
    E1002DirectUnwrapExpect,
    code = "E1002",
    name = "Direct use of unwrap/expect crashes program",
    suggestions = "Return Result to caller with ?, use if let/match, or unwrap_or_default()",
    target_items = [Function],
    config_entry_name = "e1002_direct_unwrap_expect",
    config = E1002Config {
        enabled: bool = true,
        severity: crate::config::SeverityLevel = crate::config::SeverityLevel::High,
        categories: Vec<crate::config::CheckerCategory> = vec![crate::config::CheckerCategory::Operations],
        // Add custom config fields here if needed
    },
    check_item(self, item, file_path) {
        let mut visitor = UnwrapVisitor {
            violations: Vec::new(),
            file_path,
            checker: self,
        };
        visitor.visit_item(item);
        Ok(visitor.violations)
    }
}

// Implement a syn visitor to find the pattern
struct UnwrapVisitor<'a> {
    violations: Vec<Violation>,
    file_path: &'a str,
    checker: &'a E1002DirectUnwrapExpect,
}

impl<'a> Visit<'a> for UnwrapVisitor<'a> {
    fn visit_expr(&mut self, node: &'a syn::Expr) {
        // Check for method calls to unwrap() or expect()
        if let syn::Expr::MethodCall(method) = node {
            let method_name = method.method.to_string();
            if method_name == "unwrap" || method_name == "expect" {
                let span = method.method.span();
                self.violations.push(
                    Violation::new(
                        self.checker.code(),
                        self.checker.name(),
                        self.checker.severity().into(),
                        "Direct unwrap/expect crashes on None/Err",
                        self.file_path,
                        span.start().line,
                        span.start().column + 1,
                    )
                    .with_suggestion(self.checker.suggestions()),
                );
            }
        }
        syn::visit::visit_expr(self, node);
    }
}

// Always include tests
#[cfg(test)]
mod tests {
    use super::*;

    fn check_code(code: &str) -> Vec<Violation> {
        let checker = E1002DirectUnwrapExpect::default();
        let file = syn::parse_file(code).expect("Failed to parse");
        let mut violations = Vec::new();
        for item in &file.items {
            violations.extend(checker.check_item(item, "test.rs").unwrap());
        }
        violations
    }

    #[test]
    fn test_detects_unwrap() {
        let code = r#"
            fn example() {
                let x: Option<i32> = None;
                x.unwrap();
            }
        "#;
        let violations = check_code(code);
        assert_eq!(violations.len(), 1);
    }

    #[test]
    fn test_allows_unwrap_or() {
        let code = r#"
            fn example() {
                let x: Option<i32> = None;
                x.unwrap_or(0);
            }
        "#;
        let violations = check_code(code);
        assert!(violations.is_empty());
    }
}
```

### Step 3: Register the Checker

**3a. Add to mod.rs:**

Edit `crates/hyp-checks-generic/src/checkers/e10/mod.rs`:

```rust
pub mod e1002_direct_unwrap_expect;
// ... other modules

pub use e1002_direct_unwrap_expect::{E1002Config, E1002DirectUnwrapExpect};
```

**3b. Add to registry.rs:**

Edit `crates/hyp-checks-generic/src/checkers/e10/registry.rs`:

```rust
use crate::checkers::e10::{
    // ... existing imports
    E1002Config, E1002DirectUnwrapExpect,
};

pub fn e10_registrations() -> Vec<CheckerRegistration> {
    vec![
        // ... existing registrations
        register_checker!(E1002DirectUnwrapExpect, E1002Config),
    ]
}
```

### Step 4: Create Problem Example

Create `crates/hyp-examples/src/e10_unsafe_code/e1002_direct_unwrap_expect.rs`:

```rust
/// E1002: Direct use of unwrap() and expect()
/// Severity: HIGH
/// LLM confusion: 2 (LOW)
///
/// Description: Using unwrap() and expect() crashes your program when values
/// are None or Err. Instead of gracefully handling errors, your code terminates.
///
/// Mitigation: Use `#![warn(clippy::unwrap_used)]` to catch these. Prefer
/// `?` operator, `if let`, `match`, or `unwrap_or_default()`.

/// PROBLEM E1002: Direct unwrap - crashes on None
pub fn e1002_option_unwrap() -> i32 {
    let data: Option<i32> = Some(42);
    data.unwrap() // What if data is None? Program crashes!
}

/// PROBLEM E1002: Direct unwrap on Result - crashes on Err
pub fn e1002_result_unwrap() -> String {
    let content = std::fs::read_to_string("config.txt").unwrap();
    content
}

// ============================================================================
// GOOD EXAMPLES - Proper alternatives
// ============================================================================

/// GOOD: Return Result to let caller decide
pub fn e1002_good_return_result() -> Result<i32, &'static str> {
    let data: Option<i32> = None;
    data.ok_or("value was not present")
}

/// GOOD: Provide defaults for optional values
pub fn e1002_good_unwrap_or_default() -> i32 {
    let data: Option<i32> = None;
    data.unwrap_or(0) // Safe! Returns 0 if None
}

/// Entry point (required!)
pub fn e1002_entry() -> Result<(), Box<dyn std::error::Error>> {
    let _ = e1002_good_return_result();
    let _ = e1002_good_unwrap_or_default();
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_good_alternatives() {
        assert_eq!(e1002_good_unwrap_or_default(), 0);
        assert!(e1002_good_return_result().is_err());
    }
}
```

### Step 5: Register Problem Example

**5a. Add to mod.rs:**

Edit `crates/hyp-examples/src/e10_unsafe_code/mod.rs`:

```rust
pub mod e1002_direct_unwrap_expect;
```

**5b. Update CLI (hyp-examples-cli/src/main.rs):**

Add the import:
```rust
use problem_examples::e10_unsafe_code::e1002_direct_unwrap_expect::e1002_entry;
```

Add to `show_category`:
```rust
"e10" => {
    // ... existing prints
    println!("E1002 - Direct use of unwrap()/expect() - crashes program");
}
```

Add to `define_problems!` macro:
```rust
("E1002", "Direct unwrap/expect crashes", e1002_entry),
```

### Step 6: Update Documentation

**6a. Update `crates/hyp-checks-generic/README.md`:**

Add your checker to the appropriate Phase table:
```markdown
| [x] | E1002 | Direct use of Unwrap/expect | HIGH | 3 | 2 | E10 Unsafe Code |
```

### Naming and Testing Rules for Problem Examples

#### Naming Conventions

| Pattern | Usage | Detected by hyp? |
|---------|-------|------------------|
| `eXXXX_bad_*` | Problematic code examples | YES - MUST trigger EXXXX error |
| `eXXXX_entry` | Entry point | No, treated as a regular function |
| `eXXXX_good_*` | Correct alternatives | YES - MUST NOT trigger any errors |

**Critical:** The `hyp` analyzer validates itself using these prefixes:
- All `eXXXX_bad_*` functions (except `_entry`) MUST trigger the corresponding EXXXX error
- All `eXXXX_good_*` functions MUST NOT trigger any EXXXX error
- `eXXXX_entry()` is a special case - it's an entry point, NOT bad code

#### Rules

* Functions **above** the `// GOOD EXAMPLES` separator must use the `eXXXX_bad_*` prefix (except the entry point). These functions demonstrate the problematic pattern and WILL be flagged by hyp.

* The entry point must be named `eXXXX_entry()`. Despite the `bad_` prefix, this is NOT bad code - it's a special entry point that exercises the bad examples safely. It demonstrates that the code is compilable and executable under some conditions, but can fail or cause harm under other conditions.

* Functions **below** the `// GOOD EXAMPLES` separator must use the `eXXXX_good_*` prefix. These demonstrate correct alternatives and WILL NOT be flagged by hyp.

* Add a unit-test section for the good examples, preceded by the header:
  ```
  // ============================================================================
  // GOOD EXAMPLES unit tests
  // ============================================================================
  ```
  Include unit tests for key `eXXXX_good_*` helpers to validate the recommended patterns.

For detailed guidance on problem example structure, see `crates/hyp-examples/ADD_YOUR_OWN_PROBLEM.md`.

## Checklist

- [ ] Created checker file in `hyp-checks-generic/src/checkers/eXX/`
- [ ] Used `define_checker!` macro with proper fields
- [ ] Implemented visitor pattern to find the AST pattern
- [ ] Added unit tests for positive and negative cases
- [ ] Registered in `mod.rs` with `pub mod` and `pub use`
- [ ] Registered in `registry.rs` with `register_checker!`
- [ ] Created problem example file in `hyp-examples/src/eXX_category/`
- [ ] Problem example has standard header format
- [ ] Problem functions start with `eXXXX_bad_` prefix
- [ ] Created `eXXXX_entry()` function
- [ ] Add `eXXXX_good_` functions to demonstrate proper alternatives
- [ ] Added to `hyp-examples/src/eXX_category/mod.rs`
- [ ] Added import to `hyp-examples-cli/src/main.rs`
- [ ] Added to `show_category()` in CLI
- [ ] Added to `define_problems!` macro in CLI
- [ ] Updated `hyp-checks-generic/README.md` roadmap table
- [ ] All tests pass: `cargo test --workspace`
- [ ] Checker appears in: `./target/release/hyp --list`

## Tips

1. **Start simple**: Get the basic detection working first, then add edge cases
2. **Use existing checkers as templates**: Look at similar checkers for patterns
3. **Test incrementally**: Run `cargo test e{your_code}` frequently
4. **Check the AST**: Use `syn::parse_file` to understand the AST structure
5. **Be conservative**: Better to have false negatives than false positives

## Common Patterns

### Method Call Detection
```rust
if let syn::Expr::MethodCall(method) = node {
    if method.method == "unwrap" { ... }
}
```

### Macro Detection
```rust
if let syn::Stmt::Macro(stmt_macro) = node {
    if is_panic_macro(&stmt_macro.mac) { ... }
}
```

### Nested Counting
```rust
struct DepthVisitor { depth: usize, max_depth: usize }
impl Visit<'_> for DepthVisitor {
    fn visit_expr(&mut self, node: &syn::Expr) {
        self.depth += 1;
        self.max_depth = self.max_depth.max(self.depth);
        syn::visit::visit_expr(self, node);
        self.depth -= 1;
    }
}
```

### Loop Detection
```rust
fn visit_expr(&mut self, node: &syn::Expr) {
    let in_loop = matches!(node, syn::Expr::ForLoop(_) | syn::Expr::While(_));
    if in_loop { self.loop_depth += 1; }
    // Check for patterns inside loop...
    syn::visit::visit_expr(self, node);
    if in_loop { self.loop_depth -= 1; }
}
```

## Questions?

Look at these well-documented examples:
- `e10/e1001_direct_panic.rs` - Basic macro detection
- `e10/e1002_direct_unwrap_expect.rs` - Method call detection
- `e11/e1106_long_function.rs` - Counting metrics
- `e11/e1110_deeply_nested_closures.rs` - Nesting depth analysis
