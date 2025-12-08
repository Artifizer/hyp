# Adding a New Problem Example to Hyp

This guide explains how to add a new problem example file that demonstrates a code pattern the `hyp` analyzer detects.

## Overview

Problem examples serve two purposes:
1. **Documentation**: Show developers what problematic patterns look like and how to fix them
2. **Testing**: Validate that `hyp` correctly detects bad patterns and ignores good alternatives

## File Structure

Every problem example file follows this structure:

```
┌─────────────────────────────────────────────────────────────────┐
│  PROLOG (Doc Comments)                                          │
│  - Error code, severity, LLM confusion level                    │
│  - Description of the problem                                   │
│  - Why it matters                                               │
│  - Solutions overview                                           │
│  - Mitigation strategies                                        │
├─────────────────────────────────────────────────────────────────┤
│  PROBLEMATIC PATTERNS SECTION                                   │
│  - eXXXX_bad_* functions demonstrating the problem              │
│  - Each function has PROBLEM EXXXX comment                      │
├─────────────────────────────────────────────────────────────────┤
│  ENTRY POINT                                                    │
│  - eXXXX_entry() - exercises bad examples safely                │
├─────────────────────────────────────────────────────────────────┤
│  GOOD ALTERNATIVES SECTION                                      │
│  - eXXXX_good_* functions showing correct patterns              │
│  - Each function has GOOD comment                               │
├─────────────────────────────────────────────────────────────────┤
│  UNIT TESTS                                                     │
│  - Tests for good alternatives only                             │
└─────────────────────────────────────────────────────────────────┘
```

## Naming Conventions

### Critical Naming Rules

| Pattern | Usage | Detected by hyp? |
|---------|-------|------------------|
| `eXXXX_bad_*` | Problematic code examples | YES |
| `eXXXX_entry` | Entry point (special case) | No |
| `eXXXX_good_*` | Correct alternatives | YES |

**Why this matters:**
- The `hyp` analyzer uses these prefixes to validate its detection
- `eXXXX_bad_*` functions MUST trigger at least the corresponding EXXXX error
- `eXXXX_good_*` functions MUST NOT trigger any error
- `eXXXX_entry()` is the exception - it's an entry point, not bad code

## Step-by-Step Guide

### Step 1: Choose Your Problem Code

Problem codes follow the pattern `E{category}{number}`:

| Category | Range | Description |
|----------|-------|-------------|
| E10xx | Unsafe Code | panics, unwrap, unsafe blocks, FFI |
| E11xx | Code Surface Complexity | long functions, many parameters |
| E12xx | Code Pattern Complexity | complex generics, lifetimes |
| E13xx | Error Handling | unhandled results, ignored errors |
| E14xx | Type Safety | overflow, division by zero |
| E15xx | Concurrency | race conditions, deadlocks |
| E16xx | Memory Safety | use-after-free, leaks |
| E17xx | Performance | unnecessary allocations, clones |
| E18xx | API Design | bad naming, missing docs |

Pick the next available number in the appropriate category.

### Step 2: Create the File

Create a new file in the appropriate category directory:

```bash
touch crates/hyp-examples/src/e{XX}_{category}/e{XXXX}_{description}.rs
```

Example:
```bash
touch crates/hyp-examples/src/e10_unsafe_code/e1002_direct_unwrap_expect.rs
```

### Step 3: Write the Prolog

The prolog is a doc comment block at the top of the file:

```rust
/// E1002: Direct use of unwrap() and expect()
/// Severity: HIGH
/// LLM confusion: 2 (LOW)
///
/// Description: Using unwrap() and expect() crashes your program when values
/// are None or Err. Instead of gracefully handling errors, your code terminates
/// abruptly, potentially leaving resources in an inconsistent state.
///
/// ## Why This Matters
///
/// 1. **Crashes in production**: None/Err values cause immediate termination
/// 2. **No recovery**: Unlike Result, panics can't be handled gracefully
/// 3. **Hidden assumptions**: unwrap() assumes success without documenting why
/// 4. **Debugging difficulty**: Panic messages may not indicate root cause
///
/// ## The Right Solutions
///
/// ### Option 1: Propagate with ?
/// ```rust
/// fn read_config() -> Result<Config, Error> {
///     let content = std::fs::read_to_string("config.toml")?;
///     Ok(parse(content)?)
/// }
/// ```
///
/// ### Option 2: Provide defaults
/// ```rust
/// let value = option.unwrap_or(default);
/// let value = option.unwrap_or_else(|| compute_default());
/// ```
///
/// ### Option 3: Handle explicitly
/// ```rust
/// match result {
///     Ok(value) => process(value),
///     Err(e) => log_and_recover(e),
/// }
/// ```
///
/// Mitigation: Use `#![warn(clippy::unwrap_used)]` to catch these. Prefer
/// `?` operator, `if let`, `match`, or `unwrap_or_default()`.
```

#### Prolog Components

| Component | Required | Description |
|-----------|----------|-------------|
| Error code | ✅ | `E{XXXX}: {Short title}` |
| Severity | ✅ | `HIGH`, `MED`, or `LOW` |
| LLM confusion | ✅ | `1-5 (LOW/MED/HIGH)` - how often LLMs produce this bug |
| Description | ✅ | What the problem is and why it's bad |
| Why This Matters | ✅ | Numbered list of consequences |
| The Right Solutions | ✅ | Multiple options with code examples |
| Mitigation | ✅ | Clippy lints, tools, best practices |

### Step 4: Write Problematic Patterns

```rust
// ============================================================================
// PROBLEMATIC PATTERNS
// ============================================================================

/// PROBLEM E1002: Direct unwrap on Option - crashes on None
pub fn e1002_bad_option_unwrap() -> i32 {
    let data: Option<i32> = Some(42);
    data.unwrap()  // Crashes if None!
}

/// PROBLEM E1002: Direct unwrap on Result - crashes on Err
pub fn e1002_bad_result_unwrap() -> String {
    let content = std::fs::read_to_string("config.txt").unwrap();
    content
}

/// PROBLEM E1002: expect() is just unwrap() with a message
pub fn e1002_bad_expect() -> i32 {
    let data: Option<i32> = None;
    data.expect("data should exist")  // Still crashes!
}
```

**Rules for bad examples:**
- Function names MUST start with `eXXXX_bad_`
- Each function MUST have a `/// PROBLEM EXXXX:` doc comment
- Code should be compilable and runnable (under some conditions)
- Show realistic scenarios where the problem occurs

### Step 5: Write the Entry Point

Entry points enable the example CLI to call and run the corresponding `eXXXX_bad_` functions safely, proving that they can compile and execute without errors in controlled scenarios. However, these `eXXXX_bad_` functions still illustrate real-world problems—they may crash or behave badly given different inputs or situations, highlighting the danger of the underlying pattern.


```rust
/// Entry point for problem demonstration.
/// This exercises the bad examples in a way that doesn't crash.
pub fn e1002_entry() -> Result<(), Box<dyn std::error::Error>> {
    // Call bad examples only when they won't crash
    let _ = e1002_bad_option_unwrap();  // Safe because Some(42)
    // Don't call e1002_bad_result_unwrap() - file doesn't exist
    Ok(())
}
```

**Rules for entry point:**
- MUST be named `eXXXX_entry`
- MUST return `Result<(), Box<dyn std::error::Error>>`
- Should exercise bad examples safely (when possible)
- Is NOT considered bad code by hyp analyzer

### Step 6: Write Good Alternatives

```rust
// ============================================================================
// GOOD ALTERNATIVES
// ============================================================================

/// GOOD: Propagate error to caller with ?
pub fn e1002_good_propagate() -> Result<String, std::io::Error> {
    let content = std::fs::read_to_string("config.txt")?;
    Ok(content)
}

/// GOOD: Provide a default value
pub fn e1002_good_unwrap_or() -> i32 {
    let data: Option<i32> = None;
    data.unwrap_or(0)  // Returns 0 if None
}

/// GOOD: Compute default lazily
pub fn e1002_good_unwrap_or_else() -> i32 {
    let data: Option<i32> = None;
    data.unwrap_or_else(|| {
        println!("Computing default...");
        42
    })
}

/// GOOD: Handle both cases explicitly
pub fn e1002_good_match(data: Option<i32>) -> String {
    match data {
        Some(n) => format!("Got: {}", n),
        None => "No data".to_string(),
    }
}

/// GOOD: Use if let for optional processing
pub fn e1002_good_if_let(data: Option<i32>) {
    if let Some(n) = data {
        println!("Processing: {}", n);
    }
}
```

**Rules for good examples:**
- Function names MUST start with `eXXXX_good_`
- Each function MUST have a `/// GOOD:` doc comment
- Show idiomatic Rust solutions
- Cover multiple approaches when applicable

### Step 7: Write Unit Tests

```rust
// ============================================================================
// GOOD EXAMPLES unit tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_unwrap_or() {
        assert_eq!(e1002_good_unwrap_or(), 0);
    }

    #[test]
    fn test_unwrap_or_else() {
        assert_eq!(e1002_good_unwrap_or_else(), 42);
    }

    #[test]
    fn test_match_some() {
        assert_eq!(e1002_good_match(Some(42)), "Got: 42");
    }

    #[test]
    fn test_match_none() {
        assert_eq!(e1002_good_match(None), "No data");
    }
}
```

**Rules for tests:**
- Test ONLY the good examples
- Verify the correct behavior of recommended patterns
- Don't test bad examples (they may crash or fail)

### Step 8: Register the Problem Example

**8a. Add to category mod.rs:**

Edit `crates/hyp-examples/src/e{XX}_{category}/mod.rs`:

```rust
pub mod e1002_direct_unwrap_expect;
```

**8b. Update CLI (hyp-examples-cli/src/main.rs):**

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

## Complete Example

See examples in [src](src/)

## Validation Checklist

Before submitting your problem example:

- [ ] File is in correct category directory
- [ ] Prolog has all required components (code, severity, confusion, description, solutions, mitigation)
- [ ] All problematic functions start with `eXXXX_bad_`
- [ ] All problematic functions have `/// PROBLEM EXXXX:` comment
- [ ] Entry point is named `eXXXX_entry`
- [ ] Entry point returns `Result<(), Box<dyn std::error::Error>>`
- [ ] All good functions start with `eXXXX_good_`
- [ ] All good functions have `/// GOOD:` comment
- [ ] Unit tests cover good examples only
- [ ] Added to category `mod.rs`
- [ ] Added to `hyp-examples-cli/src/main.rs`
- [ ] Code compiles: `cargo build -p hyp-examples`
- [ ] Tests pass: `cargo test -p hyp-examples`
- [ ] hyp detects bad patterns: `cargo run -p hyp -- crates/hyp-examples/src/eXX_*/eXXXX_*.rs`

## Questions?

Look at these well-documented examples:
- `e10_unsafe_code/e1016_mutex_unwrap.rs` - Comprehensive example
- `e13_error_handling/e1301_unhandled_result.rs` - Error handling patterns
- `e17_performance/e1701_oversized_struct.rs` - Performance patterns
- `e18_api_design/e1807_non_idiomatic_builder.rs` - API design patterns
