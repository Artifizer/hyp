# Hyp Analyzer CLI

The reference command-line interface for the Hyp Rust code analyzer.

## Installation

```bash
# Build and install locally
cargo install --path crates/hyp

# Or run directly from source
cargo run --bin hyp -- [COMMAND] [OPTIONS]
```

## Usage

Running `hyp` without any command displays help information.

### Commands

```bash
# Display help (default)
hyp
hyp help

# Scan source code for problems
hyp check [PATH]              # PATH defaults to current directory
hyp check src/                # Check specific directory
hyp check src/main.rs         # Check specific file
hyp check src/ --check-tests  # Check both code and tests in src/

# List available checkers
hyp list                      # All checkers
hyp list --severity 3         # Only high-severity
hyp list --category operations # Only operations category

# Print effective configuration
hyp print-config              # Show all settings
hyp print-config --include e10 # Show E10xx settings only

# Generate AI guidelines
hyp guideline                 # All enabled checkers
hyp guideline --include e10,e13 # Specific checkers only

# Validate problem examples
hyp verify-examples           # Validate all examples
hyp verify-examples --include e10 # Validate E10xx only
```

### Global Options

These work with all commands:

```bash
--all                         # Enable all checkers
--include e10,e1401           # Include only specified (supports prefixes)
--exclude e1002,e11           # Exclude specified (supports prefixes)
--severity 3                  # Minimum severity (1=Low, 2=Med, 3=High)
--category operations         # Filter by category
-f json                       # Output format (text or json)
-v, -vv                       # Verbose output (info or debug)
```

### Examples

```bash
# Check with all checkers enabled
hyp check --all

# Check excluding some checkers
hyp check src/ --exclude e1001,e1002

# Check only high-severity issues
hyp check --severity 3

# List only complexity checkers
hyp list --category complexity

# Output as JSON
hyp check src/ -f json

# Verbose checking
hyp check -vv
```

## As a Template for Custom CLIs

This CLI serves as a **reference implementation** for building your own project-specific
`cargo hyp-myproject` command. Key patterns:

1. **Use `get_all_checkers()`** to include all built-in Hyp checkers
2. **Add custom checkers** with `register_checker!`
3. **Call `run_cli()`** with your registrations

See `src/main.rs` for the full implementation and the
[main README](../../README.md#building-custom-validators) for a step-by-step guide
on building your own custom Hyp CLI.
