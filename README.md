> THIS IS EXPERIMENTAL AND NOT READY FOR PRODUCTION USE
> Version: 0.1, early preview.

# Hyp - Rust Code Quality Analyzer

**Hyp** is a static code analyzer that catches **compilable but problematic Rust patterns** that standard tools typically miss. It identifies code that compiles successfully but may confuse developers, cause runtime problems, or violate project-specific conventions helping teams write clearer, safer, and more maintainable code with less code review overhead.

## The Problem Hyp Solves

Rust's compiler excels at preventing memory bugs and race conditions, but it **assumes your logic is intentional**. This means the compiler allows code that panics at runtime, confuses developers, or violates your project's conventions.

**Clippy** and other tools **significantly extends** the compiler's capabilities by detecting potential problems through hundreds of lints. However, Clippy has important limitations:

1. **No custom rules**: Built-in lints only - you cannot define project-specific rules
2. **Easy to bypass**: Developers (or LLMs) can disable lints with `#[allow(...)]` annotations
3. **Hard to enforce**: Inline overrides make it difficult to maintain coding standards across large teams

**dylint** addresses the custom rules limitation by allowing you to build your own lints, but it analyzes HIR/HIM, not raw *.rs files. It only sees code that Cargo compiles and so it can not lint snippets

**Hyp fills this gap** by:
- Analyzing raw *.rs files directly without requiring full compilation (like `grep`)
- Enabling fast IDE integration for instant feedback without project builds
- Supporting **custom project-specific rules** (tenant filters, architectural boundaries, etc.)
- **Intentionally preventing per-file bypasses** - rules are enforced at the project level
- Providing **semantic analysis** for patterns that require understanding code structure
- Being designed for **AI code generation era** where strict, non-bypassable rules are essential

This is especially important when using AI code generation or onboarding new team members where consistency and safety cannot be compromised.

> Keep in mind that no tool including Rust's compiler, Clippy, DyLint, Kani, and Hyp can catch every possible bug or logic error. A comprehensive approach combining multiple tools and thorough manual code review remains essential.

## What Hyp Catches (That Other Tools Don't)

Here are examples of problems that **compile fine** and that **Clippy cannot detect even with all lints enabled**:

```rust
// Example 1: Unbounded thread spawning (NO Clippy lint exists)
// Clippy: No lint - compiles, runs fine in tests, exhausts resources in production
// Hyp: Detects E1511 (unbounded spawning in loop)
fn process_items(items: Vec<String>) {
    for item in items {
        thread::spawn(move || {
            expensive_operation(&item);
        });  // Spawns 1 million threads if items.len() == 1M!
    }
}

// Example 2: Higher-ranked trait bounds (HRTB) complexity (NO Clippy lint exists)
// Clippy: No lint - compiles, but very confusing for team members
// Hyp: Detects E1209 (HRTB complexity) - helps maintain code readability
fn complex_closure<F>(f: F)
where
    F: for<'a, 'b> Fn(&'a str, &'b str) -> &'a str  // HRTB - hard to understand!
{
    let result = f("hello", "world");
    // ...
}

// Example 3: Deep nesting in loops (Clippy cognitive_complexity is different)
// Clippy: cognitive_complexity counts all branches, not loop-specific nesting
// Hyp: Detects E1102 (deeply nested logic specifically in loops)
fn process_matrix(data: Vec<Vec<Vec<i32>>>) -> i32 {
    let mut sum = 0;
    for layer in &data {
        for row in layer {
            for cell in row {
                if *cell > 0 {
                    if *cell % 2 == 0 {
                        if *cell < 100 {  // 6 levels deep in loop!
                            sum += cell;
                        }
                    }
                }
            }
        }
    }
    sum
}

// Example 4: Missing tenant_id filter in multi-tenant SaaS (Custom project rule)
// Clippy: Cannot enforce project-specific business logic
// Hyp: Custom checker ensures all SeaORM queries include tenant isolation
async fn get_users(db: &DatabaseConnection) -> Result<Vec<User>> {
    User::find()
        // .filter(user::Column::TenantId.eq(tenant_id))  // MISSING! Security bug!
        .all(db)
        .await
}

// Example 5: DTO defined outside API layer (Custom project rule)
// Clippy: Cannot enforce architectural boundaries
// Hyp: Custom checker ensures DTOs only exist in api/ folder
// File: src/database/models.rs
#[derive(Serialize)]  // ❌ DTO in wrong layer!
pub struct UserResponse {  // Should be in api/dto/user_response.rs
    pub id: i64,
    pub email: String,
}

// Example 6: Clippy rule bypassed - confuses newcomers and violates team standards
// Clippy: Can be disabled with #[allow] - no way to enforce globally
// Hyp: Cannot be bypassed at file level - enforces team standards
#[allow(clippy::unwrap_used)]  // ❌ Junior dev or LLM bypassed the rule!
pub fn process_user_input(data: Option<String>) -> String {
    data.unwrap()  // Clippy won't warn - but this WILL panic in production!
    // Team policy: "Never use unwrap on user input" - but Clippy can't enforce it
}

// Example 7: Prohibit std::thread::spawn in async/tokio codebases (Custom project rule)
// Clippy: Cannot enforce async runtime boundaries
// Hyp: Custom checker E1512 ensures only tokio::task::spawn_blocking is used
// File: src/services/user_service.rs
pub async fn process_users(users: Vec<User>) {
    for user in users {
        std::thread::spawn(move || {  // ❌ Breaks tokio runtime!
            expensive_computation(&user);  // Context loss, no tracing, can't await
        });
    }
}

// CORRECT: Use tokio::task::spawn_blocking instead
pub async fn process_users_correct(users: Vec<User>) {
    for user in users {
        tokio::task::spawn_blocking(move || {  // ✅ Proper tokio integration
            expensive_computation(&user);
        }).await.unwrap();
    }
}

```

See complete comparison in [HYP_VS_CLIPPY.md](HYP_VS_CLIPPY.md) and [hyp-examples](hyp-examples/)

## Quick start

Compile the `hyp-examples` repository binary crate and ensure Rust compilation succeeds and code is runnable:
```bash
cargo run --bin hyp-examples run-all
```

Build Hyp and run it over the `hyp-examples` source code and see the errors:
```bash
cargo run --bin hyp -- check crates/hyp-examples/
```

Now run it over the Hyp code itself, no flagged errors expected (TODO - 328 violations found):
```bash
cargo run --bin hyp -- check crates/hyp-checks-generic/
```

Ensure all the examples and Hyp lints are correct (TODO - 388 validation issues found):
```bash
cargo run --bin hyp -- verify-examples crates/hyp-examples
```

## Conceptual Model

Hyp intentionally takes a **conservative approach**, sometimes flagging potential issues that may not always represent real errors. This allows developers to make an informed decision about whether to suppress a specific check or to enhance the checker to better support additional use cases.

At a high level Hyp does two things:

- **Parses Rust code into an AST** using the `syn` crate.
- **Runs a set of applicable checkers over that AST**, where each checker looks for one specific kind of problem.

The project owners and Hyp users can defined their own rules and configurations in project config file

### Problem Categories and Codes

All problems Hyp can detect are grouped into **categories**. The built‑in categories focus on
purely **Rust-specific** issues:

- **E10** – Unsafe code (panics, unwrap, unsafe blocks, FFI)
- **E11** – Code surface complexity (long functions, many parameters, deep nesting)
- **E12** – Code pattern complexity (complex generics, lifetimes, trait bounds)
- **E13** – Error handling patterns
- **E14** – Type safety (overflow, division/modulo by zero, unchecked indexing)
- **E15** – Concurrency
- **E16** – Memory safety
- **E17** – Performance
- **E18** – API design
- **E19** – Code hygiene (naming rules, file locations, inline directives)

Within each category, individual problems are identified by **checker codes**,
similar to PEP8 or Clippy lints, e.g.:

- `E1001` – Direct call of `panic!`
- `E1002` – Direct call of `unwrap` / `expect`
- `E1004` – `todo!()` / `unimplemented!()` in production code
- `E1106` – Long function (too many lines)
- `E1112` – Hardcoded magic numbers
- `E1401` – Integer overflow/underflow
- `E1410` – Float equality comparison with `==`
- `E1511` – Unbounded task/thread spawning in loops
- `E1611` – Method consumes `self` unnecessarily
- `E1712` – Expensive operations inside loops (Regex, File::open)
- `E1812` – Public enum without `#[non_exhaustive]`
- …and 90+ more listed with `hyp --list`.

These codes are what you **enable/disable** in configuration and on the CLI:

- In `Hyp.toml`, by checker key (e.g. `e1001_direct_panic`, `e1106_long_function`).
- On the CLI, via `--include e1001,e1106` or `--exclude e1402`.

Additionally, you can assign categories and adjust the severity level for any problem, then run validators on only a selected subset of issues.

### Extending Categories for Your Project

Project owners are encouraged to add their own **local categories** in addition
to the built‑in Rust families. For example:

- **E2xx – Project‑specific Rust rules and patterns prevention**
  e.g. “prohibit user‑defined generics” or “no direct thread spawning” for certain crates or folders.
- **E3xx – Repository layout rules**
  e.g. “DTOs must live only in `api/`”, “no business logic in `routes/`”.
- **E4xx – Business logic and security rules**
  e.g. “database access must go through access‑controlled repositories”,
  “every admin API must check authorization middleware”, “no raw SQLx usage”.

These custom categories use the **same mechanism** as built‑in ones and you can easily build your own Hyp checkers and CLI tool:
- see [hyp-checks-generic/ADD_YOUR_OWN_CHECKER.md](hyp-checks-generic/ADD_YOUR_OWN_CHECKER.md) for new checkers definition
- see [hyp/BUILD_YOUR_OWN_HYP_CLI.md](hyp/BUILD_YOUR_OWN_HYP_CLI.md) for own CLI

You can also **disable any built‑in Hyp checkers** that don’t match your project’s style while building your CLI version. Alternatively, you can fully configure project rules in the Hyp.toml config file enabling or disabling various checkers, redefining priority and adjusting categories as you wich. See the [Hyp configuration file](#hyp-configuration-file) section

## How Hyp Differs from Other Rust Tools

Hyp fills a unique niche in the Rust tooling ecosystem. Here's how it compares:

| Tool | Focus | Approach | When to Use |
|------|-------|----------|-------------|
| **Hyp** | Cognitive complexity, LLM-friendliness, custom business rules | AST pattern matching, pluggable checkers | Code clarity, team standards, project-specific rules |
| **Clippy** | Idiomatic Rust, common mistakes | Compiler plugin, 700+ lints | General code quality, learning Rust idioms |
| **Miri** | Undefined behavior detection | Interpreter-based execution | Testing unsafe code, catching UB at runtime |
| **Kani** | Formal verification of properties | Model checking, SAT/SMT solving | Proving correctness of critical algorithms |
| **Prusti** | Verification via specifications | Viper verifier, annotations | Formal contracts, pre/post conditions |
| **MIRAI** | Abstract interpretation | Static analysis of MIR | Finding bugs without running code |

### Key Differentiators

**Pluggable & Extensible**: Unlike compiler-integrated tools, Hyp is designed for **custom business logic checks**. Add your own project-specific rules without forking or modifying the tool.

**LLM-Aware**: Hyp is designed to identify patterns that often confuse AI code assistants or Rust newcomers helping teams produce clearer and more maintainable code.

**Cognitive Metrics**: While Clippy checks for correctness, Hyp also measures *understandability*—cyclomatic complexity, nesting depth, and complicated patterns that overload human working memory.

**Build Your Own Analyzer**: Create project-specific `cargo` commands with custom checks for your domain (see [Building Custom Validators](#building-custom-validators)).

## Built-in checks

### Problem Examples
The `hyp-examples` crate contains **compilable but complicated or unsafe Rust code** designed to illustrate real-world Rust patterns that:
- Compile successfully but are difficult to review or maintain by human developers or LLMs
- May cause runtime errors or undefined behavior
- Violate best practices despite being technically valid

Each example includes:
- **Severity rating** (LOW/MED/HIGH) - Impact on code safety and maintainability
- **LLM confusion score** - How likely the pattern confuses AI code assistants
- **Clear description** - What the problem is and why it matters
- **Mitigation strategies** - How to detect and fix the issue

The default `hyp` static analyzer detects these problematic patterns in real codebases and delivers actionable reports, empowering developers to address issues before committing code.

## Building Custom Validators

Hyp is designed to be extended with **project-specific checks**. You can create a custom
`cargo` command (e.g., `cargo hyp-myproject`) that combines Hyp's built-in checkers with
your own domain-specific rules—all in a separate repository or a local crate inside your project.

### Architecture Overview

```
┌───────────────────────────────────────────────────────────────────────┐
│            Your Custom CLI Binary (cargo-hyp-myproject)               │
├───────────────────────────────────────────────────────────────────────┤
│  ┌──────────────┐ ┌─────────────┐ ┌───────────────┐ ┌───────────────┐ │
│  │ Hyp Built-in │ │ Your Custom │ │   Your Repo   │ │ Your Business │ │
│  │  Checkers    │ │ Rust Checks │ │ Layout Checks │ │ Logic Checks  │ │
│  │   (E1...)    │ │   (E2...)   │ │   (E3...)     │ │   (E4...)     │ │
│  └──────────────┘ └─────────────┘ └───────────────┘ └───────────────┘ │
├───────────────────────────────────────────────────────────────────────┤
│                 hyp-checks-generic library with helpers                     │
└───────────────────────────────────────────────────────────────────────┘
```

See [crates/hyp/BUILD_YOUR_OWN_HYP_CLI.md](crates/hyp/BUILD_YOUR_OWN_HYP_CLI.md) for complete guide.

## Hyp Configuration File

Hyp configuration rules are defined in `Hyp.toml`. The file is discovered by searching from the current directory up through parent directories until found. If no file exists, defaults are used. This allows project-wide defaults with folder-specific exceptions.

To see the default configuration:

```bash
hyp print-config
```

Note: Running `hyp` without any command will display help information.

### Configuration Format

```toml
[checkers]
# Every checker has at least three common configurable properties (defaults shown):
e1001_direct_panic.enabled = true      # Enable/disable this checker
e1001_direct_panic.severity = 3        # Severity level: 1=Low, 2=Medium, 3=High
e1001_direct_panic.categories = ["operations"]  # Categorization for filtering

# Disable specific checks
e1002_direct_unwrap_expect.enabled = false

# Some checkers have additional properties, e.g. 'max_lines'
e1106_long_function.enabled = true
e1106_long_function.max_lines = 1000

# Override categories for custom filtering
e1402_division_by_zero.categories = ["my_category", "operations"]
```

### Category-Level Configuration

You can disable entire categories of checkers using short prefixes:

```toml
[checkers]
# Disable all E11xx (Code Surface Complexity) checkers
e11.enabled = false

# Disable all E14xx (Type Safety) checkers
e14.enabled = false

# Disable all E1xxx checkers (not recommended, but possible)
e1.enabled = false
```

This is useful when you want to focus on specific problem categories or temporarily ignore a whole class of issues.

### Advanced Configuration: E19 Code Hygiene Checkers

The E19 category provides powerful project-specific enforcement capabilities through configurable rules. These checkers help maintain architectural boundaries, naming conventions, and code organization standards.

#### E1904: Allowed Names and Paths Control

Enforces where specific types of items (structs, enums, functions, etc.) with certain naming patterns can be defined.

**Use cases:**
- Restrict DTOs to API layer only
- Prevent wildcard imports in specific modules
- Enforce naming conventions for specific file locations

**Configuration example:**

```toml
[checkers.e1904_allowed_names]
enabled = true
severity = 3
categories = ["compliance"]

# Define multiple rules
[[checkers.e1904_allowed_names.rules]]
enabled = true
item_types = ["struct"]
reference_type = "define"
name_patterns = [".*DTO$", ".*Dto$"]  # Matches any struct ending with "DTO"
allowed_paths = ["^.*/api/.*\\.rs$"]  # Only in api/ directory
message = "DTO struct '{name}' can not be defined outside {path}"

[[checkers.e1904_allowed_names.rules]]
enabled = true
item_types = ["use"]
reference_type = "use"
name_patterns = ["^sqlx::.*$"]  # Matches sqlx::* wildcard imports
allowed_paths = ["^.*/api/.*$"]  # NOT in api/ directory
message = "SQLx can not be imported and used in {path}"
```

**Fields:**
- `item_types`: Array of AST item types (`struct`, `enum`, `trait`, `function`, `const`, `static`, `type`, `use`, `mod`, `impl`)
- `reference_type`: Either `define` (where item is defined) or `refer` (where item is used)
- `name_patterns`: Array of regex patterns for item names
- `allowed_paths`: Array of regex patterns for file paths where items are allowed
- `message`: Custom violation message with placeholders: `{type}`, `{name}`, `{path}`, `{allowed_paths}`

#### E1905: Inline Directive Control

Prevents bypassing project rules with inline directives like `#[allow(clippy::...)]` in unauthorized locations. Critical for maintaining standards in AI-generated code.

**Use cases:**
- Block all Clippy bypasses to enforce project-level configuration
- Restrict warning suppressions to test code only
- Prevent LLMs from adding `#[allow(...)]` to silence warnings

**Configuration example:**

```toml
[checkers.e1905_inline_directives]
enabled = true
severity = 3
categories = ["compliance"]

# Block Clippy allows everywhere - force Clippy.toml usage
[[checkers.e1905_inline_directives.rules]]
enabled = true
directive_patterns = ["allow\\(clippy::.*\\)"]
allowed_paths = ["^$"]  # Empty = nowhere allowed
message = "Inline clippy '{directive}' is not alowed in {path} - use Clippy.toml for project-wide config"

# Allow warning suppression only in tests
[[checkers.e1905_inline_directives.rules]]
enabled = true
directive_patterns = ["allow\\(warnings\\)", "allow\\(dead_code\\)"]
allowed_paths = ["^.*/tests/.*\\.rs$", "^.*_test\\.rs$"]
message = "Warning suppression '{directive}' in '{path}' is not allowed - only allowed in tests"
```

**Fields:**
- `directive_patterns`: Array of regex patterns matching attribute directives (e.g., `allow\\(clippy::unwrap_used\\)`)
- `allowed_paths`: Array of regex patterns for file paths where directives are permitted
- `message`: Custom violation message with placeholders: `{directive}`, `{path}`, `{allowed_paths}`

#### E1903: File Location Control

Enforces that specific files can only exist in designated locations. Useful for configuration files, build scripts, and maintaining project structure.

**Use cases:**
- Ensure Clippy.toml and rustfmt.toml are at project root
- Restrict proto files to proto/ directory
- Enforce SQL migrations location
- Control where build.rs can exist

**Configuration example:**

```toml
[checkers.e1903_file_location]
enabled = true
severity = 2
categories = ["compliance"]

# Config files must be at project root
[[checkers.e1903_file_location.rules]]
filename_pattern = "^Clippy\\.toml$"
allowed_paths = ["^[^/]+/Clippy\\.toml$"]
message = "Clippy.toml in {path} must be at project root only"

[[checkers.e1903_file_location.rules]]
filename_pattern = "^rustfmt\\.toml$"
allowed_paths = ["^[^/]+/rustfmt\\.toml$"]
message = "rustfmt.toml in {path} must be at project root only"

# Proto files must be in proto/ directory
[[checkers.e1903_file_location.rules]]
filename_pattern = ".*\\.proto$"
allowed_paths = ["^.*/proto/.*\\.proto$"]
message = "Proto file '{filename}' in {path} must be in proto/ directory"
```

**Fields:**
- `filename_pattern`: Regex pattern matching the filename (e.g., `Cargo\\.toml`, `.*\\.proto$`)
- `allowed_paths`: Array of regex patterns for allowed file paths
- `message`: Custom violation message with placeholders: `{filename}`, `{path}`, `{allowed_paths}`

#### E1908: Unsafe Justification Requirement

Requires every unsafe block to have a justification comment (e.g., `// SAFETY:`). Optionally restricts unsafe blocks to specific paths.

**Use cases:**
- Enforce documentation of all unsafe code
- Restrict unsafe blocks to designated modules (e.g., `unsafe_ops/`)
- Require specific comment patterns for audit trails
- Maintain safety invariants documentation

**Configuration example:**

```toml
[checkers.e1908_unsafe_justification]
enabled = true
severity = 3
categories = ["compliance"]
require_justification = true
comment_patterns = ["SAFETY:", "UNSAFE:"]

# Optional: Restrict unsafe to specific paths
[[checkers.e1908_unsafe_justification.path_rules]]
comment_patterns = ["SAFETY:"]
allowed_paths = ["^.*/unsafe_ops/.*\\.rs$", "^.*/ffi/.*\\.rs$"]
message = "Unsafe blocks in {path} only allowed in unsafe_ops/ or ffi/ directories"
```

**Fields:**
- `require_justification`: Boolean - whether to require justification comments (default: true)
- `comment_patterns`: Array of strings that must appear in comments (e.g., `SAFETY:`, `UNSAFE:`)
- `path_rules`: Optional array of rules restricting unsafe blocks to specific paths
  - `comment_patterns`: Required comment patterns for this rule
  - `allowed_paths`: Regex patterns for paths where unsafe is permitted
  - `message`: Custom violation message with placeholders: `{path}`, `{allowed_paths}`

**Note:** Due to `syn` crate limitations, only doc comments (`///` or `/** */`) are detected, not regular line comments (`//`). For best results, use doc comments before unsafe blocks.

## Project Structure

This workspace contains 4 crates:

### Libraries

1. **hyp-examples** - Compilable examples of problematic Rust code patterns
   - 100+ examples across 9 categories (E10-E18)
   - Each example demonstrates a specific anti-pattern
   - Intentionally disables clippy warnings to compile
   - Used for testing, documentation, and education

2. **hyp-checks-generic** - The core static analysis library
   - AST-based pattern detection using `syn`
   - `define_checker!` and `register_checker!` macros for easy extension
   - Pluggable checker architecture
   - TOML-based configuration (`Hyp.toml`)

### CLI Tools

3. **hyp-examples-cli** - Interactive explorer for problem examples
   ```bash
   cargo run --bin hyp-examples list
   cargo run --bin hyp-examples show e10
   ```

4. **hyp** - Main analyzer CLI tool that can be used as is or as hyp-custom example
   ```bash
   hyp check src/
   hyp list
   hyp print-config
   ```

### Example Problems Hyp Catches

```rust
// E1015: unwrap without context - confuses LLMs about error handling
let value = data.unwrap();  // ❌ What could go wrong here?

// E1004: todo!/unimplemented! in production - LLMs often leave these behind
todo!("implement this later");  // ❌ Will panic at runtime!

// E1101: high cyclomatic complexity - too many paths to reason about
fn process(x: i32, y: i32, z: i32) -> i32 {
    if x > 0 { if y > 0 { if z > 0 { /* ... */ } } }  // ❌ Hard to follow
}

// E1410: float equality - classic precision bug
if 0.1 + 0.2 == 0.3 { ... }  // ❌ This is FALSE due to float precision!

// E1511: unbounded spawning - resource exhaustion
for item in items {
    std::thread::spawn(|| process(item));  // ❌ May spawn millions of threads!
}

// E1712: expensive ops in loop - performance antipattern
for line in lines {
    let re = Regex::new(r"\d+").unwrap();  // ❌ Compiles regex every iteration!
}
```

## Built-in Problem Categories

- **E10** - Unsafe Code (panics, unwrap, unsafe blocks, FFI)
- **E11** - Code Surface Complexity (cyclomatic complexity, long functions, many parameters)
- **E12** - Code Pattern Complexity (complex generics, lifetimes, trait bounds)
- **E13** - Error Handling (ignored results, poor error types, panic in Drop)
- **E14** - Type Safety (overflow, division by zero, unchecked indexing)
- **E15** - Concurrency (race conditions, deadlocks, non-Send types)
- **E16** - Memory Safety (use-after-free, dangling references, Rc cycles)
- **E17** - Performance (unnecessary allocations, inefficient data structures)
- **E18** - API Design (glob imports, public fields, poor naming)

## Quick Start

```bash
# Clone the repository
git clone https://github.com/yourusername/hyp.git
cd hyp

# Build all crates
cargo build

# List all available checkers
cargo run --bin hyp list

# Run the analyzer on source code
cargo run --bin hyp check crates/hyp-examples/src

# Print effective configuration
cargo run --bin hyp print-config

# Explore problem examples
cargo run --bin hyp-examples list
cargo run --bin hyp-examples show e10

# Run tests
cargo test
```

## CLI Commands

Hyp provides the following commands:

```bash
hyp [COMMAND] [OPTIONS]
```

**Note:** Running `hyp` without any command displays help information.

### Available Commands

| Command | Description |
|---------|-------------|
| `check [PATH]` | Scan source code for problems. `PATH` defaults to current directory if not specified. |
| `list` | List all available checkers with their code, name, severity, and categories. |
| `print-config` | Print the effective TOML configuration showing all checker settings. |
| `guideline` | Print condensed AI guidelines based on currently enabled checkers. |
| `verify-examples [PATH]` | Validate that Hyp correctly detects problems in example code. `PATH` defaults to `crates/hyp-examples/src`. |
| `help` | Print help information for Hyp or a specific subcommand. |

### Global Options

These options work with all commands:

| Option | Description | Example |
|--------|-------------|---------|
| `--all` | Enable all checkers (overrides `Hyp.toml` config) | `hyp check --all` |
| `--include <CODES>` | Include only specified checkers (comma-separated, supports prefixes) | `--include e10,e1401` |
| `--exclude <CODES>` | Exclude specific checkers (comma-separated, supports prefixes) | `--exclude e1002,e11` |
| `--severity <LEVEL>` | Filter by minimum severity level (1=Low, 2=Medium, 3=High) | `--severity 3` |
| `--category <CATS>` | Filter by categories (comma-separated: operations, complexity, compliance) | `--category operations` |
| `-f, --format <FMT>` | Output format: `text` (default) or `json` | `-f json` |
| `-v, --verbose` | Increase verbosity. Use `-v` for info, `-vv` for debug. | `-vv` |

### Usage Examples

```bash
# Install the Hyp checker locally
cargo install --path crates/hyp

# Display help (default behavior)
hyp
hyp help

# Check current directory with default config
hyp check

# Check specific path
hyp check src/

# Check with all checkers enabled
hyp check --all

# Check specific path, excluding some checkers
hyp check src/ --exclude e1001,e1002

# Check with only high-severity issues
hyp check --severity 3

# Check specific category
hyp check --category operations

# List all available checkers
hyp list

# List only high-severity checkers
hyp list --severity 3

# List checkers in a specific category
hyp list --category complexity

# Print effective configuration
hyp print-config

# Print configuration for specific checkers only
hyp print-config --include e10

# Generate AI guidelines for all enabled checkers
hyp guideline

# Generate guidelines for specific checkers
hyp guideline --include e10,e13

# Validate problem examples
hyp verify-examples

# Validate specific examples only
hyp verify-examples --include e10,e14

# Output results as JSON
hyp check src/ -f json
```

## Verify-Examples Command

The `verify-examples` command validates that Hyp correctly detects problems in the `hyp-examples` crate. This ensures checkers work as intended.

### How It Works

Problem example functions follow a naming convention:

| Pattern | Meaning | Expected Behavior |
|---------|---------|-------------------|
| `eXXXX_bad_*` | Problematic code | Hyp MUST detect error EXXXX |
| `eXXXX_good_*` | Correct alternative | Hyp MUST NOT detect error EXXXX |

### Validation Rules

1. **Bad functions** (`eXXXX_bad_*`): If Hyp detects error code EXXXX → **OK**. If not detected → **FAIL**.
2. **Good functions** (`eXXXX_good_*`): If Hyp does NOT detect error code EXXXX → **OK**. If detected → **FAIL**.

- Ensuring good examples don't trigger false positives

## Development Status

**Overall status**: proof-of-concept, actively developed
**Validators**: 97 checkers implemented across all 9 categories (E10-E18)
See detailed roadmap in [crates/hyp-checks-generic/README.md](crates/hyp-checks-generic/README.md)

## Target Audience

Hyp is designed for:

- **Rust learners** - Understand common pitfalls through compilable examples
- **Development teams** - Maintain consistent code quality and readability
- **AI-assisted development** - Write code that's clear to both humans and LLMs
- **Code reviewers** - Identify subtle issues that traditional linters miss
- **Library authors** - Ensure APIs are intuitive and hard to misuse
- **Platform teams** - Enforce project-specific conventions and business rules

## Contributing

Contributions are welcome! Areas where you can help:

1. **Add problem examples** - Found a confusing Rust pattern? Add it to `hyp-examples/`
2. **Improve descriptions** - Make explanations clearer for learners
3. **Build analyzer rules** - Implement detection for existing problem categories
4. **Test and report** - Try Hyp on real codebases and report findings
5. **Documentation** - Improve guides, examples, and API docs

See [CONTRIBUTING.md](CONTRIBUTING.md) for detailed guidelines.

## Related Projects

- **[Clippy](https://github.com/rust-lang/rust-clippy)** - Comprehensive Rust linter (correctness and idioms)
- **[Kani](https://github.com/model-checking/kani)** - Bit-precise model checker for Rust
- **[Miri](https://github.com/rust-lang/miri)** - Interpreter for detecting undefined behavior
- **[Prusti](https://github.com/viperproject/prusti-dev)** - Verification via Viper
- **[MIRAI](https://github.com/facebookexperimental/MIRAI)** - Abstract interpreter for Rust

Hyp complements these tools by focusing on **cognitive complexity, LLM-friendliness, and custom code validation**.

## ROADMAP

See [ROADMAP.md](ROADMAP.md)

## License

Apache-2.0
