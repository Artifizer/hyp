# Hyp Analyzer

A static code analyzer for Rust that detects common code quality issues, unsafe patterns, and complexity problems.

## Overview

Hyp Analyzer is a pluggable static analysis tool that parses Rust source code and applies configurable checkers to detect problems across the following categories:

- **E10**: Unsafe Code
- **E11**: Code Surface Complexity
- **E12**: Code Pattern Complexity
- **E13**: Error Handling
- **E14**: Type Safety
- **E15**: Concurrency
- **E16**: Memory Safety
- **E17**: Performance
- **E18**: API Design

## Implementation Roadmap

This roadmap categorizes all problems by:
- **LLM Clarity**: How clear the problem is for an LLM to implement (1=very clear, 5=very unclear)
- **Implementation Difficulty**: Estimated complexity of implementing the checker (1=simple, 5=very complex)
- **Priority Phase**: When to implement (Phase 1=simple & clear, Phase 2=moderate, Phase 3=complex & unclear)

### Phase 1: Simple & Clear Checks (42 checkers)

These are straightforward pattern matches or simple metrics. Implement first.

| Supported | Code | Name | Severity | LLM Clarity | Impl Difficulty | Category |
|-----------|------|------|----------|-------------|-----------------|----------|
 [x] | E1001 | Direct call of panic() in production code | HIGH | 2 | 1 | E10 Unsafe Code |
 [x] | E1002 | Direct use of Unwrap/expect | HIGH | 3 | 2 | E10 Unsafe Code |
 [x] | E1004 | todo!/unimplemented! macros in code | HIGH | 2 | 1 | E10 Unsafe Code |
 [x] | E1005 | Raw pointer deref | HIGH | 4 | 2 | E10 Unsafe Code |
 [x] | E1015 | Unwrap/expect without context | HIGH | 3 | 2 | E10 Unsafe Code |
 [x] | E1016 | Mutex lock().unwrap() panic cascade | HIGH | 3 | 2 | E10 Unsafe Code |
 [x] | E1103 | Too many function parameters | LOW | 2 | 1 | E11 Surface Complexity |
 [x] | E1105 | Boolean parameter hell | LOW | 2 | 2 | E11 Surface Complexity |
 [x] | E1106 | Long function (too many lines) | LOW | 2 | 1 | E11 Surface Complexity |
 [x] | E1109 | Excessive method chaining | LOW | 2 | 2 | E11 Surface Complexity |
 [x] | E1305 | Non-exhaustive match on Result/Option | HIGH | 2 | 2 | E13 Error Handling |
 [x] | E1307 | Using String for error types | MED | 2 | 2 | E13 Error Handling |
 [x] | E1308 | Not using ? operator when appropriate | LOW | 2 | 2 | E13 Error Handling |
 [x] | E1402 | Division by zero | HIGH | 2 | 2 | E14 Type Safety |
 [x] | E1403 | Modulo by zero | HIGH | 2 | 2 | E14 Type Safety |
 [x] | E1405 | Integer division rounding errors | LOW | 2 | 2 | E14 Type Safety |
 [x] | E1408 | Unchecked array indexing | HIGH | 2 | 2 | E14 Type Safety |
 [x] | E1409 | Partial initialization | MED | 2 | 2 | E14 Type Safety |
 [x] | E1508 | Sleep instead of synchronization | LOW | 2 | 2 | E15 Concurrency |
 [x] | E1510 | Mutex instead of RwLock | LOW | 2 | 2 | E15 Concurrency |
 [x] | E1606 | Unnecessary clone | LOW | 2 | 2 | E16 Memory Safety |
 [x] | E1701 | Oversized struct passed by value | LOW | 2 | 2 | E17 Performance |
 [x] | E1702 | Unnecessary allocations | LOW | 2 | 2 | E17 Performance |
 [x] | E1703 | String concatenation in loop | LOW | 2 | 2 | E17 Performance |
 [x] | E1704 | Unnecessary collect() | LOW | 2 | 2 | E17 Performance |
 [x] | E1705 | Clone in hot path | LOW | 2 | 2 | E17 Performance |
 [x] | E1708 | Inefficient data structure | LOW | 2 | 2 | E17 Performance |
 [x] | E1709 | Unnecessary boxing | LOW | 2 | 2 | E17 Performance |
 [x] | E1801 | Glob imports | LOW | 2 | 1 | E18 API Design |
 [x] | E1802 | Public fields without validation | MED | 2 | 2 | E18 API Design |
 [x] | E1803 | Bad naming | LOW | 1 | 2 | E18 API Design |
 [x] | E1804 | Inconsistent error types | MED | 2 | 2 | E18 API Design |
 [x] | E1805 | Missing documentation | LOW | 1 | 1 | E18 API Design |
 [x] | E1806 | Exposing internal details | MED | 2 | 2 | E18 API Design |
 [x] | E1807 | Non-idiomatic builder | LOW | 2 | 2 | E18 API Design |
 [x] | E1808 | Mutable getter | MED | 2 | 2 | E18 API Design |
 [x] | E1809 | Fallible new() | MED | 2 | 2 | E18 API Design |
 [x] | E1810 | String instead of &str | LOW | 2 | 2 | E18 API Design |
 [x] | E1104 | Overly large struct (too many fields) | MED | 2 | 1 | E11 Surface Complexity |
 [x] | E1111 | Excessive tuple complexity | LOW | 2 | 2 | E11 Surface Complexity |
 [x] | E1112 | Hardcoded magic numbers | LOW | 2 | 2 | E11 Surface Complexity |
 [x] | E1410 | Float equality comparison with == | MED | 2 | 2 | E14 Type Safety |
 [x] | E1511 | Unbounded task/thread spawning in loop | HIGH | 2 | 2 | E15 Concurrency |
 [x] | E1611 | Method consumes self unnecessarily | MED | 2 | 2 | E16 Memory Safety |
 [x] | E1712 | Expensive operations inside loops | MED | 2 | 2 | E17 Performance |
 [x] | E1812 | Public enum without #[non_exhaustive] | LOW | 2 | 2 | E18 API Design |


### Phase 2: Moderate Complexity (40 checkers)

These require more sophisticated AST analysis or control flow understanding.

| Supported | Code | Name | Severity | LLM Clarity | Impl Difficulty | Category |
|-----------|------|------|----------|-------------|-----------------|----------|
 [x] | E1003 | Direct use of unsafe code | HIGH | 4 | 3 | E10 Unsafe Code |
 [x] | E1007 | Dereferencing null pointer | HIGH | 4 | 3 | E10 Unsafe Code |
 [x] | E1008 | Unsafe trait implementation | HIGH | 4 | 3 | E10 Unsafe Code |
 [x] | E1010 | Mutable static without synchronization | HIGH | 4 | 3 | E10 Unsafe Code |
 [x] | E1013 | Union with unsafe field access | HIGH | 4 | 3 | E10 Unsafe Code |
 [x] | E1014 | Raw pointer arithmetic without bounds checking | HIGH | 4 | 3 | E10 Unsafe Code |
 [x] | E1101 | High cyclomatic complexity | MED | 3 | 3 | E11 Surface Complexity |
 [x] | E1102 | Deeply nested logic in loops and conditions | MED | 4 | 3 | E11 Surface Complexity |
 [x] | E1107 | Deeply nested conditionals | MED | 3 | 3 | E11 Surface Complexity |
 [x] | E1108 | Deeply nested match expressions | MED | 3 | 3 | E11 Surface Complexity |
 [x] | E1204 | Multiple traits with same method names | MED | 3 | 3 | E12 Pattern Complexity |
 [x] | E1211 | Trait object coercion complexity | MED | 4 | 3 | E12 Pattern Complexity |
 [x] | E1213 | Const generics with complex constraints | MED | 4 | 3 | E12 Pattern Complexity |
 [x] | E1217 | Classical ABBA Deadlock | HIGH | 3 | 4 | E12 Pattern Complexity |
 [x] | E1301 | Unhandled Result values | HIGH | 3 | 2 | E13 Error Handling |
 [x] | E1302 | Constructors returning bare values instead of Result | MED | 3 | 2 | E13 Error Handling |
 [x] | E1303 | Ignoring errors with let _ = | HIGH | 3 | 2 | E13 Error Handling |
 [x] | E1304 | Using unwrap() in error paths | HIGH | 3 | 3 | E13 Error Handling |
 [x] | E1306 | Swallowing errors without logging | MED | 3 | 3 | E13 Error Handling |
 [x] | E1310 | Error context loss | MED | 3 | 3 | E13 Error Handling |
 [x] | E1401 | Integer overflow/underflow | HIGH | 3 | 3 | E14 Type Safety |
 [x] | E1404 | Narrowing conversions (as) | MED | 3 | 3 | E14 Type Safety |
 [x] | E1406 | Signed/unsigned mismatch | MED | 3 | 3 | E14 Type Safety |
 [x] | E1407 | Lossy float to int conversion | MED | 3 | 3 | E14 Type Safety |
 [x] | E1503 | Lock poisoning mishandled | MED | 3 | 3 | E15 Concurrency |
 [x] | E1506 | Deadlock from lock ordering | HIGH | 3 | 4 | E15 Concurrency |
 [x] | E1509 | Channel lifetime issues | MED | 3 | 3 | E15 Concurrency |
 [x] | E1605 | Rc cycle memory leak | MED | 3 | 3 | E16 Memory Safety |
 [x] | E1607 | Forgetting to drop | MED | 3 | 3 | E16 Memory Safety |
 [x] | E1603 | Dangling reference | HIGH | 4 | 4 | E16 Memory Safety |
 [x] | E1604 | Buffer overflow | HIGH | 4 | 4 | E16 Memory Safety |
 [x] | E1609 | Invalid slice creation | HIGH | 4 | 4 | E16 Memory Safety |
 [x] | E1610 | Unaligned dereference | HIGH | 4 | 4 | E16 Memory Safety |
 [x] | E1706 | Non-tail recursion | MED | 3 | 3 | E17 Performance |
 [x] | E1707 | Unbounded recursion | HIGH | 3 | 3 | E17 Performance |
 [x] | E1710 | Large stack allocation | MED | 3 | 3 | E17 Performance |
 [x] | E1201 | Overly complex generics | MED | 4 | 3 | E12 Pattern Complexity |
 [x] | E1203 | Complicated borrowing patterns | MED | 4 | 4 | E12 Pattern Complexity |
 [x] | E1210 | Recursive type definitions | HIGH | 4 | 4 | E12 Pattern Complexity |
 [x] | E1904 | Unsafe without comments | HIGH | 4 | 3 | E10 Unsafe Code |

### Phase 3: Complex & Unclear (26 checkers)

These require deep semantic analysis, advanced type system understanding, or are ambiguous to detect.
Some Phase 3 checkers have been implemented with **heuristic/syntactic analysis** (marked [x]) while others truly require semantic analysis and cannot be implemented with `syn` alone (marked [ ]).

| Supported | Code | Name | Severity | LLM Clarity | Impl Difficulty | Category |
|-----------|------|------|----------|-------------|-----------------|----------|
 [x] | E1006 | Transmute without size/alignment checks | HIGH | 5 | 5 | E10 Unsafe Code |
 [x] | E1110 | Deeply nested callbacks/closures | MED | 5 | 4 | E11 Surface Complexity |
 [x] | E1202 | Complex lifetime annotations | MED | 5 | 5 | E12 Pattern Complexity |
 [x] | E1205 | Complex handler with nested trait bounds | MED | 5 | 5 | E12 Pattern Complexity |
 [x] | E1206 | Deeply nested generic types | HIGH | 4 | 4 | E12 Pattern Complexity |
 [x] | E1207 | Complex user-defined generic constraints | HIGH | 5 | 5 | E12 Pattern Complexity |
 [x] | E1208 | Phantom types and zero-sized markers | MED | 5 | 4 | E12 Pattern Complexity |
 [x] | E1209 | Higher-ranked trait bounds (HRTB) | HIGH | 5 | 5 | E12 Pattern Complexity |
 [x] | E1212 | Generic Associated Types (GATs) complexity | HIGH | 5 | 5 | E12 Pattern Complexity |
 [x] | E1214 | Macro-generated trait implementations | MED | 5 | 5 | E12 Pattern Complexity |
 [x] | E1215 | Type-level programming with const evaluation | HIGH | 5 | 5 | E12 Pattern Complexity |
 [x] | E1216 | Chained transformations with associated type bounds | MED | 5 | 5 | E12 Pattern Complexity |
 [x] | E1309 | Panic in Drop implementation | HIGH | 4 | 4 | E13 Error Handling |
 [x] | E1502 | Lock held across await (heuristic) | HIGH | 4 | 4 | E15 Concurrency |
 [ ] | E1009 | UnsafeCell misuse and interior mutability violations | HIGH | 5 | 5 | E10 Unsafe Code |
 [ ] | E1011 | Uninitialized memory | HIGH | 5 | 5 | E10 Unsafe Code |
 [ ] | E1012 | Unsafe auto trait implementation | HIGH | 5 | 5 | E10 Unsafe Code |
 [ ] | E1411 | Type confusion with transmute | HIGH | 5 | 5 | E14 Type Safety |
 [ ] | E1501 | Non-Send types across threads | HIGH | 4 | 4 | E15 Concurrency |
 [ ] | E1504 | Interior mutability race | HIGH | 4 | 4 | E15 Concurrency |
 [ ] | E1505 | Non-Send future | HIGH | 4 | 4 | E15 Concurrency |
 [ ] | E1507 | Unsynchronized shared state | HIGH | 4 | 4 | E15 Concurrency |
 [ ] | E1601 | Aliasing violation | HIGH | 5 | 5 | E16 Memory Safety |
 [ ] | E1602 | Use-after-free | HIGH | 5 | 5 | E16 Memory Safety |
 [ ] | E1608 | Double free | HIGH | 5 | 5 | E16 Memory Safety |

## Limitations

Hyp Analyzer uses **syntactic analysis** (AST parsing via `syn`) rather than **semantic analysis** (type checking, data flow). This means some checkers cannot be fully implemented without deeper compiler integration.

### Phase 3 Checkers: What's Possible

Phase 3 checkers were originally marked as "Complex & Unclear" requiring semantic analysis. However, **13 of 26** have been implemented using heuristic/syntactic analysis:

#### Implemented with Heuristics (13 checkers)

These detect patterns visible in the AST, though they may have edge cases:

| Code | Name | Approach |
|------|------|----------|
| E1110 | Deeply nested callbacks/closures | Counts closure/async block nesting depth |
| E1202 | Complex lifetime annotations | Counts lifetime parameters and bounds |
| E1205 | Complex handler with nested trait bounds | Counts where clause predicates and total bounds |
| E1206 | Deeply nested generic types | Measures generic type nesting depth |
| E1207 | Complex user-defined generic constraints | Counts type and const generic parameters |
| E1208 | Phantom types and zero-sized markers | Detects PhantomData usage patterns |
| E1209 | Higher-ranked trait bounds (HRTB) | Detects `for<'a>` syntax |
| E1212 | Generic Associated Types (GATs) | Detects GAT declarations in traits |
| E1214 | Macro-generated trait implementations | Detects derive macros on large types, macros in impl blocks |
| E1215 | Type-level programming with const evaluation | Detects const generic patterns |
| E1216 | Chained associated type bounds | Counts associated type projections and bounds |
| E1309 | Panic in Drop implementation | Detects panic!/unwrap/assert in Drop::drop |
| E1502 | Lock held across await | Heuristic: detects lock followed by .await in same block |

#### Not Implementable with Syn (13 checkers)

These **require semantic analysis** that `syn` cannot provide:

| Code | Name | Why Not Possible |
|------|------|------------------|
| E1009 | UnsafeCell misuse | Requires tracking interior mutability semantics across statements |
| E1011 | Uninitialized memory | Needs data flow analysis to track initialization state |
| E1012 | Unsafe auto trait implementation | Requires understanding Send/Sync auto-derivation semantics |
| E1410 | Type confusion with transmute | Needs type system information for compatibility checking |
| E1501 | Non-Send types across threads | Requires Send trait bound resolution |
| E1504 | Interior mutability race | Requires data race / happens-before analysis |
| E1505 | Non-Send future | Requires async + Send trait analysis |
| E1507 | Unsynchronized shared state | Requires cross-thread synchronization analysis |
| E1601 | Aliasing violation | Requires pointer alias analysis |
| E1602 | Use-after-free | Requires lifetime/ownership data flow analysis |
| E1608 | Double free | Requires ownership tracking across control flow |

### What This Means

- **Hyp can detect patterns** visible in the AST (syntax tree)
- **Hyp cannot detect** patterns requiring:
  - Type information (sizes, alignments, trait bounds)
  - Data flow analysis (tracking variable state across statements)
  - Cross-function analysis (understanding call graphs)
  - Semantic understanding (what code "means" vs. what it "looks like")

### Alternatives for Full Semantic Analysis

For the 13 non-implementable checkers, consider:
- **Clippy** - Compiler-integrated lints with full type information
- **MIRAI** - MIR-level static analysis for unsafe code
- **Kani** - Formal verification with model checking
- **Miri** - Runtime undefined behavior detection
- **Prusti** - Verification via specifications

Hyp complements these tools by focusing on **syntactic patterns** that are:
- Fast to detect (no type checking overhead)
- Easy to understand (clear AST patterns)
- Suitable for custom project rules (extensible without compiler changes)

## Architecture

### Core Components

1. **Parser**: Uses `syn` crate to parse Rust source files into AST
2. **Checker Registry**: Manages all available checkers
3. **Checker Trait**: Standard interface all checkers implement
4. **Configuration**: TOML configuration deserialized into per-checker config structs
5. **Reporter**: Formats and outputs violations

### Checker Structure

Each checker lives in its own file: `src/checkers/eXXXX_name.rs` and is typically
implemented with the `define_checker!` macro from `checker_config_macro.rs`. This macro:

- Generates a config struct (with `enabled`, `severity`, `categories`, plus custom fields)
- Defines the checker struct and its `CONFIG_ENTRY_NAME` used for config lookup
- Implements the `Checker` trait
- Lets you focus on writing the `check_item` logic only

```rust
use crate::{define_checker, violation::Violation};
use syn::visit::Visit;

define_checker! {
    /// Checker for E1001: Direct panic calls
    E1001DirectPanic,
    code = "E1001",
    name = "Direct panic() call",
    suggestions = "Return Result<T, E> instead of panicking",
    target_items = [Function],
    config_entry_name = "e1001_direct_panic",
    /// Configuration for E1001: Direct panic checker
    config = E1001Config {
        enabled: bool = true,
        severity: crate::config::SeverityLevel = crate::config::SeverityLevel::High,
        categories: Vec<crate::config::CheckerCategory> =
            vec![crate::config::CheckerCategory::Operations],
    },
    // AST node item checker – you implement just this part
    check_item(self, item, file_path) {
        let mut visitor = PanicVisitor {
            violations: Vec::new(),
            file_path,
            checker: self,
        };
        visitor.visit_item(item);
        Ok(visitor.violations)
    }
}
```

See these concrete implementations for patterns to follow:

- `checkers/e10/e1001_direct_panic.rs`
- `checkers/e11/e1106_long_function.rs`
- `checkers/e14/e1401_integer_overflow.rs`

### Configuration Schema

Each checker has a config type generated by `define_checker!`. All configs support:

- `enabled: bool`
- `severity: SeverityLevel` (Low/Medium/High)
- `categories: Vec<CheckerCategory>`

and may add custom fields (for example `max_lines` for long functions).

Example – E1106 (long function), see `e11/e1106_long_function.rs`:

```rust
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(default)]
pub struct E1106Config {
    pub enabled: bool,
    pub severity: crate::config::SeverityLevel,
    pub categories: Vec<crate::config::CheckerCategory>,
    /// Maximum allowed lines in a function
    pub max_lines: usize,
}

impl Default for E1106Config {
    fn default() -> Self {
        Self {
            enabled: true,
            severity: crate::config::SeverityLevel::Low,
            categories: vec![crate::config::CheckerCategory::Complexity],
            max_lines: 250,
        }
    }
}
```

Example – E1401 (integer overflow), see `e14/e1401_integer_overflow.rs`:

```rust
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(default)]
pub struct E1401Config {
    pub enabled: bool,
    pub severity: crate::config::SeverityLevel,
    pub categories: Vec<crate::config::CheckerCategory>,
}

impl Default for E1401Config {
    fn default() -> Self {
        Self {
            enabled: true,
            severity: crate::config::SeverityLevel::High,
            categories: vec![crate::config::CheckerCategory::Operations],
        }
    }
}
```

At runtime, `AnalyzerConfig` deserializes TOML into these config structs via
`get_checker_config::<E1106Config>("e1106_long_function")`, using the
`CONFIG_ENTRY_NAME` defined by `define_checker!`.

## Usage

### CLI Tool

The `hyp` CLI provides several commands for analyzing code and managing configuration:

```bash
# Display help (default when no command given)
hyp
hyp help

# Scan source code
hyp check                     # Check current directory
hyp check /path/to/code       # Check specific path
hyp check --all               # Enable all checkers
hyp check --severity 3        # Only high-severity issues

# List available checkers
hyp list                      # All checkers
hyp list --severity 3         # High-severity only
hyp list --category operations # Specific category

# View configuration
hyp print-config              # Show all settings
hyp print-config --include e10 # Show E10xx only

# Generate AI guidelines
hyp guideline                 # All enabled checkers
hyp guideline --include e10   # Specific checkers

# Validate examples
hyp verify-examples           # Validate problem examples
```

### Global Options

```bash
--all                         # Enable all checkers (overrides config)
--include e10,e1401           # Include only specified (comma-separated)
--exclude e1002,e11           # Exclude specified (supports prefixes)
--severity 3                  # Minimum severity (1=Low, 2=Med, 3=High)
--category operations         # Filter by category
-f json                       # Output format (text or json)
-v, -vv                       # Verbose output
```

### Configuration File (Hyp.toml)

Hyp searches for `Hyp.toml` from the current directory upward through parent directories.

```toml
[checkers]
# Enable/disable individual checkers
e1001_direct_panic.enabled = true
e1002_direct_unwrap_expect.enabled = false

# Configure checker-specific settings
e1106_long_function.enabled = true
e1106_long_function.max_lines = 200

e1101_high_cyclomatic_complexity.enabled = true
e1101_high_cyclomatic_complexity.max_complexity = 15

# Adjust severity and categories
e1001_direct_panic.severity = 3
e1001_direct_panic.categories = ["operations"]

# Disable entire checker categories
e11.enabled = false  # Disable all E11xx (Code Surface Complexity)
e14.enabled = false  # Disable all E14xx (Type Safety)
```

## Development

### Adding a New Checker

1. Create `src/checkers/eXXXX_name.rs`.
2. Use the `define_checker!` macro to declare the checker and its config
   (see `e10/e1001_direct_panic.rs`, `e11/e1106_long_function.rs`,
   `e14/e1401_integer_overflow.rs` as templates).
3. Register the checker in the appropriate group module and registry using
   the `register_checker!` macro (for example in `checkers/e10/mod.rs` and
   `checkers/e10/registry.rs`).
4. Add unit tests in the checker file to cover both positive and negative cases.
5. Update this README roadmap table to mark the checker as supported.

### Testing

```bash
# Run all tests
cargo test

# Test specific checker
cargo test e1001

# Run against problem examples
cargo run --bin hyp -- --source ../hyp-examples/src
```

## License

See LICENSE file in repository root.
