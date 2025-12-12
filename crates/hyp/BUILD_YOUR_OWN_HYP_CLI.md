# Build Your Own Hyp CLI

This guide shows how to create a custom `cargo` subcommand (e.g., `cargo hyp-myproject`) that combines Hyp's built-in checkers with your own project-specific rules.

## Architecture Overview

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
│            hyp-checks-generic library (cli_helper + macros)                 │
└───────────────────────────────────────────────────────────────────────┘
```

## Quick Start

### 1. Create the project

```bash
cargo new cargo-hyp-myproject --bin
cd cargo-hyp-myproject
```

### 2. Add dependencies (`Cargo.toml`)

```toml
[package]
name = "cargo-hyp-myproject"
version = "0.1.0"
edition = "2021"

[[bin]]
name = "cargo-hyp-myproject"
path = "src/main.rs"

[dependencies]
hyp-checks-generic = { git = "https://github.com/user/hyp", branch = "main" }
# or from crates.io once published:
# hyp-checks-generic = "0.1"

syn = { version = "2", features = ["full", "visit"] }
clap = { version = "4", features = ["derive"] }
anyhow = "1"
```

### 3. Create a minimal CLI (`src/main.rs`)

The CLI is intentionally minimal - all shared logic lives in `hyp_analyzer::cli_helper`:

```rust
use anyhow::Result;
use clap::{Parser, Subcommand};
use hyp_analyzer::{
    cli_helper::{run_cli, print_checker_list_from_registrations},
    find_config_file, get_all_checkers, parse_categories, split_csv,
    CliOptions, CliOutputFormat, CheckerRegistration,
    register_checker,
};
use std::path::PathBuf;

mod checkers;
use checkers::{E4001TransactionLeak, E4001Config};

#[derive(Parser)]
#[command(name = "cargo-hyp-myproject")]
#[command(about = "Custom Hyp analyzer for MyProject")]
struct Cli {
    /// Cargo passes "hyp-myproject" as first arg when invoked as `cargo hyp-myproject`
    #[arg(hide = true)]
    _cargo_subcommand: Option<String>,

    #[command(subcommand)]
    command: Option<Commands>,

    #[arg(long, global = true)]
    all: bool,

    #[arg(long, global = true)]
    include: Option<String>,

    #[arg(long, global = true)]
    exclude: Option<String>,

    #[arg(long, global = true)]
    severity: Option<u8>,

    #[arg(long, global = true)]
    category: Option<String>,

    #[arg(short = 'f', long, default_value = "text", global = true)]
    format: String,

    #[arg(short = 'v', long, action = clap::ArgAction::Count, global = true)]
    verbose: u8,
}

#[derive(Subcommand)]
enum Commands {
    Check { path: Option<PathBuf> },
    List,
}

/// Combine Hyp's built-in checkers with your custom ones
fn all_registrations() -> Vec<CheckerRegistration> {
    let mut regs = get_all_checkers();  // All E1xxx built-in checkers

    // Add your custom checkers
    regs.push(register_checker!(E4001TransactionLeak, E4001Config));

    regs
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let categories = parse_categories(&cli.category);

    let opts = CliOptions {
        source: PathBuf::from("."),
        config_path: find_config_file(),
        severity: cli.severity,
        categories,
        all: cli.all,
        include: cli.include.as_ref().map(|s| split_csv(s)),
        exclude: cli.exclude.as_ref().map(|s| split_csv(s)),
        format: if cli.format == "json" { CliOutputFormat::Json } else { CliOutputFormat::Text },
        verbose: cli.verbose,
    };

    match &cli.command {
        Some(Commands::Check { path }) => {
            let mut opts = opts;
            opts.source = path.clone().unwrap_or_else(|| PathBuf::from("."));
            run_cli(opts, all_registrations)?;
        }
        Some(Commands::List) => {
            print_checker_list_from_registrations(&opts, all_registrations())?;
        }
        None => {
            run_cli(opts, all_registrations)?;
        }
    }

    Ok(())
}
```

### 4. Define your custom checkers

Create `src/checkers/mod.rs`:

```rust
pub mod e4001_transaction_leak;
pub use e4001_transaction_leak::{E4001TransactionLeak, E4001Config};
```

Create `src/checkers/e4001_transaction_leak.rs`:

```rust
use hyp_analyzer::{define_checker, violation::Violation};
use syn::visit::Visit;

define_checker! {
    /// E4001: Detects database queries outside transaction context
    E4001TransactionLeak,
    code = "E4001",
    name = "Database query outside transaction",
    suggestions = "Wrap database operations in a transaction block",
    target_items = [Function],
    config_entry_name = "e4001_transaction_leak",
    config = E4001Config {
        enabled: bool = true,
        severity: hyp_analyzer::config::SeverityLevel =
            hyp_analyzer::config::SeverityLevel::High,
        categories: Vec<hyp_analyzer::config::CheckerCategory> =
            vec![hyp_analyzer::config::CheckerCategory::Compliance],
    },
    check_item(self, item, file_path) {
        let mut visitor = TransactionVisitor {
            violations: Vec::new(),
            file_path,
            checker: self,
        };
        visitor.visit_item(item);
        Ok(visitor.violations)
    }
}

struct TransactionVisitor<'a> {
    violations: Vec<Violation>,
    file_path: &'a str,
    checker: &'a E4001TransactionLeak,
}

impl<'a> Visit<'a> for TransactionVisitor<'a> {
    fn visit_expr_method_call(&mut self, node: &'a syn::ExprMethodCall) {
        let method_name = node.method.to_string();
        if method_name == "query" || method_name == "execute" {
            use syn::spanned::Spanned;
            let span = node.span().start();
            self.violations.push(Violation::new(
                self.checker.code(),
                self.checker.name(),
                self.checker.severity().into(),
                "Database query called without explicit transaction context",
                self.file_path,
                span.line,
                span.column + 1,
            ).with_suggestion(self.checker.suggestions()));
        }
        syn::visit::visit_expr_method_call(self, node);
    }
}
```

### 5. Build and use

```bash
cargo build --release
cargo install --path .

# Now use it:
cargo hyp-myproject check src/
cargo hyp-myproject list
cargo hyp-myproject check --include e4001  # Only your custom checker
```

## Available CLI Helpers

The `hyp_analyzer::cli_helper` module provides these reusable functions:

| Function | Purpose |
|----------|---------|
| `run_cli()` | Main analysis runner with full workflow |
| `print_checker_list_from_registrations()` | Print checker list table |
| `print_guidelines_from_registrations()` | Print AI guidelines |
| `print_default_config()` | Generate TOML config template |
| `run_validation()` | Validate problem examples |
| `print_validation_results()` | Print validation summary |
| `find_config_file()` | Find Hyp.toml in directory tree |
| `parse_categories()` | Parse category CLI argument |
| `split_csv()` | Split comma-separated string |
| `filter_registrations()` | Apply include/exclude filters |

## Custom Checker Categories

| Code Range | Category | Example Checks |
|------------|----------|----------------|
| **E2xxx** | Custom Rust Rules | Prohibit user-defined generics, no direct thread spawning |
| **E3xxx** | Repo Layout Rules | DTOs only in `api/`, no business logic in `routes/` |
| **E4xxx** | Business Logic | Transaction leak, API auth middleware required |

## Disabling Built-in Checkers

**Via CLI:**
```bash
cargo hyp-myproject check --exclude e1106,e1002
```

**Via `Hyp.toml`:**
```toml
[checkers]
e1002_direct_unwrap_expect.enabled = false
e1106_long_function.enabled = false
```

**Or selectively include checkers in code:**
```rust
fn all_registrations() -> Vec<CheckerRegistration> {
    use hyp_analyzer::registry::{CheckerGroup, checkers_for_groups};

    // Only include E10 (unsafe) checkers from built-in
    let mut regs = checkers_for_groups(&[CheckerGroup::E10]);

    // Add your custom checkers
    regs.push(register_checker!(E4001TransactionLeak, E4001Config));

    regs
}
```

## Resources

- **Built-in checkers**: `hyp/crates/hyp-checks-generic/src/checkers/`
- **Reference CLI**: `hyp/crates/hyp/src/main.rs`
- **CLI helpers**: `hyp/crates/hyp-checks-generic/src/cli_helper.rs`
