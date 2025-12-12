//! Hyp Analyzer CLI
//!
//! Command-line interface for the Hyp Rust code analyzer.
//! This is a minimal CLI that delegates most logic to hyp_analyzer::cli_helper.

use anyhow::Result;
use clap::{Parser, Subcommand};
use hyp_analyzer::{
    cli_helper::{
        filter_registrations, print_checker_list_from_registrations,
        print_guidelines_from_registrations, run_cli,
    },
    find_config_file, get_all_checkers, parse_categories, print_default_config,
    print_validation_results, run_validation, split_csv, CliOptions, CliOutputFormat,
};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "hyp")]
#[command(about = "Hyp Rust Code Analyzer - Detect code problems and anti-patterns", long_about = None)]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,

    /// Enable all checkers (overrides Hyp.toml config)
    #[arg(long, global = true)]
    all: bool,

    /// Include only specific checkers (comma-separated, supports substring matching)
    #[arg(long, global = true)]
    include: Option<String>,

    /// Exclude specific checkers (comma-separated, supports substring matching)
    #[arg(long, global = true)]
    exclude: Option<String>,

    /// Minimum severity level (1=Low, 2=Medium, 3=High)
    #[arg(long, global = true)]
    severity: Option<u8>,

    /// Filter by categories (comma-separated: operations, complexity, compliance)
    #[arg(long, global = true)]
    category: Option<String>,

    /// Output format (text or json)
    #[arg(short = 'f', long, default_value = "text", global = true)]
    format: String,

    /// Verbose mode: -v for info, -vv for debug
    #[arg(short = 'v', long, action = clap::ArgAction::Count, global = true)]
    verbose: u8,

    /// Check unit tests (by default, tests are skipped)
    #[arg(long, global = true)]
    check_tests: bool,

    /// Path to configuration file (default: Hyp.toml in current or parent directories)
    #[arg(short = 'c', long, global = true)]
    config: Option<PathBuf>,
}

#[derive(Subcommand)]
enum Commands {
    /// Scan source code for problems
    Check {
        /// Path to source code (file or directory)
        path: Option<PathBuf>,
    },

    /// Print the effective TOML configuration
    PrintConfig,

    /// List all available checkers
    List,

    /// Print condensed guidelines for LLMs
    Guideline,

    /// Validate problem examples against the analyzer
    VerifyExamples {
        /// Path to hyp-examples source directory
        #[arg(default_value = "crates/hyp-examples/src")]
        path: Option<PathBuf>,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let categories = parse_categories(&cli.category);

    match &cli.command {
        Some(Commands::Check { path }) => {
            let source = path.clone().unwrap_or_else(|| PathBuf::from("."));
            let config_path = cli.config.clone().unwrap_or_else(|| find_config_file());
            let opts = CliOptions {
                source,
                config_path,
                severity: cli.severity,
                categories,
                all: cli.all,
                include: cli.include.as_ref().map(|s| split_csv(s)),
                exclude: cli.exclude.as_ref().map(|s| split_csv(s)),
                format: if cli.format == "json" {
                    CliOutputFormat::Json
                } else {
                    CliOutputFormat::Text
                },
                verbose: cli.verbose,
                check_tests: cli.check_tests,
            };
            run_cli(opts, get_all_checkers)?;
        }

        Some(Commands::PrintConfig) => {
            print_default_config(
                get_all_checkers,
                cli.include.as_deref(),
                cli.exclude.as_deref(),
                cli.severity,
            )?;
        }

        Some(Commands::List) => {
            let config_path = cli.config.clone().unwrap_or_else(|| find_config_file());
            let opts = CliOptions {
                source: PathBuf::from("."),
                config_path,
                severity: cli.severity,
                categories,
                all: cli.all,
                include: cli.include.as_ref().map(|s| split_csv(s)),
                exclude: cli.exclude.as_ref().map(|s| split_csv(s)),
                format: CliOutputFormat::Text,
                verbose: cli.verbose,
                check_tests: false,
            };
            print_checker_list_from_registrations(&opts, get_all_checkers())?;
        }

        Some(Commands::Guideline) => {
            let config_path = cli.config.clone().unwrap_or_else(|| find_config_file());
            let opts = CliOptions {
                source: PathBuf::from("."),
                config_path,
                severity: cli.severity,
                categories,
                all: cli.all,
                include: cli.include.as_ref().map(|s| split_csv(s)),
                exclude: cli.exclude.as_ref().map(|s| split_csv(s)),
                format: CliOutputFormat::Text,
                verbose: cli.verbose,
                check_tests: false,
            };
            print_guidelines_from_registrations(&opts, get_all_checkers())?;
        }

        Some(Commands::VerifyExamples { path }) => {
            let source = path
                .clone()
                .unwrap_or_else(|| PathBuf::from("crates/hyp-examples/src"));
            println!("Validating hyp against problem examples...\n");
            println!("Source directory: {}\n", source.display());

            let registrations = if cli.include.is_some() || cli.exclude.is_some() {
                let config_path = cli.config.clone().unwrap_or_else(|| find_config_file());
                let opts = CliOptions {
                    source: source.clone(),
                    config_path,
                    severity: cli.severity,
                    categories: None,
                    all: cli.all,
                    include: cli.include.as_ref().map(|s| split_csv(s)),
                    exclude: cli.exclude.as_ref().map(|s| split_csv(s)),
                    format: CliOutputFormat::Text,
                    verbose: cli.verbose,
                    check_tests: cli.check_tests,
                };
                filter_registrations(get_all_checkers(), &opts)
            } else {
                get_all_checkers()
            };

            let summary = run_validation(&source, || registrations)?;
            println!("Found {} problem example files\n", summary.files_processed);
            print_validation_results(&summary);

            if !summary.all_passed() {
                std::process::exit(1);
            }
        }

        None => {
            // Default to help
            let _ = Cli::parse_from(["hyp", "--help"]);
        }
    }

    Ok(())
}
