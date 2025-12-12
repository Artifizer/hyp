//! Shared helper functions for CLI frontends.
//!
//! This module provides reusable CLI functionality that can be used by both
//! the default `hyp` CLI and custom CLI tools built on top of `hyp-checks-generic`.

use std::{collections::HashMap, collections::HashSet, path::Path, path::PathBuf};
use walkdir::WalkDir;

use crate::{
    analyzer::{Analyzer, AnalyzerFilters},
    config::{AnalyzerConfig, CheckerCategory},
    registry::{checkers_for_groups, CheckerGroup, CheckerRegistration},
    violation::Violation,
    Result,
};

/// CLI output format options.
#[derive(Debug, Clone, Copy)]
pub enum CliOutputFormat {
    /// Plain text output.
    Text,
    /// JSON output.
    Json,
}

/// Load configuration from file path.
///
/// Supports both TOML (Hyp.toml) and YAML (hyp.yaml) formats.
/// The format is auto-detected based on file extension.
pub fn load_config(config_path: &Path) -> Result<AnalyzerConfig> {
    if !config_path.exists() {
        return Ok(AnalyzerConfig::default());
    }

    let content = std::fs::read_to_string(config_path)?;
    let extension = config_path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("");

    match extension.to_lowercase().as_str() {
        "toml" => AnalyzerConfig::from_toml(&content)
            .map_err(|e| crate::AnalyzerError::Config(format!("TOML parse error: {}", e))),
        "yaml" | "yml" => AnalyzerConfig::from_yaml(&content)
            .map_err(|e| crate::AnalyzerError::Config(format!("YAML parse error: {}", e))),
        _ => {
            // Try TOML first (for Hyp.toml without extension matching), then YAML
            AnalyzerConfig::from_toml(&content)
                .map_err(|e| crate::AnalyzerError::Config(format!("TOML parse error: {}", e)))
                .or_else(|_| {
                    AnalyzerConfig::from_yaml(&content).map_err(|e| {
                        crate::AnalyzerError::Config(format!("YAML parse error: {}", e))
                    })
                })
        }
    }
}

/// Generic CLI options structure that any CLI frontend can use.
pub struct CliOptions {
    /// Path to source code (file or directory).
    pub source: PathBuf,
    /// Path to configuration file.
    pub config_path: PathBuf,
    /// Minimum severity filter (1-3).
    pub severity: Option<u8>,
    /// Category filters.
    pub categories: Option<HashSet<CheckerCategory>>,
    /// Enable all checkers flag.
    pub all: bool,
    /// Include only specific checkers.
    pub include: Option<Vec<String>>,
    /// Exclude specific checkers.
    pub exclude: Option<Vec<String>>,
    /// Output format.
    pub format: CliOutputFormat,
    /// Verbosity level (0=normal, 1=info, 2=debug).
    pub verbose: u8,
    /// Whether to check unit tests (default: false, tests are skipped).
    pub check_tests: bool,
}

/// Build an analyzer from an explicit list of checker registrations.
///
/// # Errors
/// Returns an error if configuration is invalid (unknown checker names or wrong types).
pub fn build_analyzer_from_registrations(
    config: AnalyzerConfig,
    filters: AnalyzerFilters,
    registrations: Vec<CheckerRegistration>,
) -> Result<Analyzer> {
    Analyzer::new_with_checkers(config, filters, registrations)
}

/// Build an analyzer from registrations, skipping unknown-checker validation.
///
/// Use this when you've already validated the config separately (e.g., against all checkers
/// before CLI filtering was applied).
///
/// # Errors
/// Returns an error if configuration has wrong parameter types.
pub fn build_analyzer_from_registrations_skip_validation(
    config: AnalyzerConfig,
    filters: AnalyzerFilters,
    registrations: Vec<CheckerRegistration>,
) -> Result<Analyzer> {
    Analyzer::new_with_checkers_skip_unknown_validation(config, filters, registrations)
}

/// Validate configuration against a set of checker registrations.
///
/// This checks that:
/// - All configured checker names are known
/// - All configured parameters have correct types
///
/// # Errors
/// Returns an error if any validation fails.
pub fn validate_config_against_registrations(
    config: &AnalyzerConfig,
    registrations: &[CheckerRegistration],
) -> Result<()> {
    use std::collections::HashSet;

    // Build a set of known config entry names
    let known_config_names: HashSet<&str> = registrations
        .iter()
        .map(|r| r.config_entry_name)
        .collect();

    // Validate that all configured checker names are known
    for configured_name in config.configured_checker_keys() {
        if !known_config_names.contains(configured_name.as_str()) {
            let mut available: Vec<_> = known_config_names.into_iter().collect();
            available.sort();
            return Err(crate::AnalyzerError::Config(format!(
                "Unknown checker '{}' in configuration. Available checkers: {}",
                configured_name,
                available.join(", ")
            )));
        }
    }

    // Validate config types by trying to deserialize each configured checker
    for registration in registrations {
        if config.checkers.contains_key(registration.config_entry_name) {
            // Try to create the checker - this will fail if config types are wrong
            (registration.factory)(config).map_err(crate::AnalyzerError::Config)?;
        }
    }

    Ok(())
}

/// Build an analyzer from one or more logical checker groups.
///
/// # Errors
/// Returns an error if configuration is invalid (unknown checker names or wrong types).
pub fn build_analyzer_from_groups(
    config: AnalyzerConfig,
    filters: AnalyzerFilters,
    groups: &[CheckerGroup],
) -> Result<Analyzer> {
    let registrations = checkers_for_groups(groups);
    Analyzer::new_with_checkers(config, filters, registrations)
}

/// Apply CLI include/exclude filtering to registrations (case-insensitive).
///
/// Supports substring matching:
/// - "e1" matches e1001-e1999
/// - "e10" matches e1000-e1099
/// - "e1001" matches exactly e1001
pub fn filter_registrations(
    registrations: Vec<CheckerRegistration>,
    opts: &CliOptions,
) -> Vec<CheckerRegistration> {
    filter_registrations_with_config(registrations, opts, None)
}

/// Apply CLI include/exclude filtering AND config-based category filtering.
///
/// Config supports category-level disabling:
/// ```toml
/// [checkers]
/// e11.enabled = false  # Disables all E11xx checkers
/// e14.enabled = false  # Disables all E14xx checkers
/// ```
pub fn filter_registrations_with_config(
    registrations: Vec<CheckerRegistration>,
    opts: &CliOptions,
    config: Option<&AnalyzerConfig>,
) -> Vec<CheckerRegistration> {
    let include_patterns: Option<Vec<String>> = opts
        .include
        .as_ref()
        .map(|v| v.iter().map(|s| s.to_lowercase()).collect());
    let exclude_patterns: Option<Vec<String>> = opts
        .exclude
        .as_ref()
        .map(|v| v.iter().map(|s| s.to_lowercase()).collect());

    // Extract category-level disabled prefixes from config (e.g., e11, e14)
    let disabled_prefixes: Vec<String> = config
        .map(|cfg| {
            cfg.checkers
                .iter()
                .filter_map(|(key, value)| {
                    // Check for category-level keys like "e11", "e14" (2-3 chars, starting with 'e')
                    let key_lc = key.to_lowercase();
                    if key_lc.starts_with('e')
                        && key_lc.len() <= 3
                        && key_lc.len() > 1
                        && key_lc[1..].chars().all(|c| c.is_ascii_digit())
                    {
                        // Check if .enabled = false
                        if let Some(enabled) = value.get("enabled") {
                            if enabled == false {
                                return Some(key_lc);
                            }
                        }
                    }
                    None
                })
                .collect()
        })
        .unwrap_or_default();

    registrations
        .into_iter()
        .filter(|reg| {
            let code_lc = reg.descriptor.code.to_lowercase();

            // Check if disabled by category-level config (e.g., e11.enabled = false)
            for prefix in &disabled_prefixes {
                if code_lc.starts_with(prefix) {
                    return false;
                }
            }

            // If include is present, only allow codes that match any include pattern
            if let Some(ref patterns) = include_patterns {
                let matches = patterns.iter().any(|pattern| code_lc.starts_with(pattern));
                if !matches {
                    return false;
                }
            }

            // Exclude overrides include - if code matches any exclude pattern, filter it out
            if let Some(ref patterns) = exclude_patterns {
                let matches = patterns.iter().any(|pattern| code_lc.starts_with(pattern));
                if matches {
                    return false;
                }
            }

            true
        })
        .collect()
}

/// Print a table of *eligible* checkers based on config + CLI filters.
pub fn print_checker_list_from_registrations(
    opts: &CliOptions,
    registrations: Vec<CheckerRegistration>,
) -> Result<()> {
    // Load config (same as run_cli)
    let config = load_config(&opts.config_path)?;

    // Build filters (severity/category already populated in opts)
    let mut filters = AnalyzerFilters::default();
    if let Some(sev) = opts.severity {
        filters.min_severity = Some(sev);
    }
    if let Some(cats) = opts.categories.clone() {
        filters.categories = Some(cats);
    }

    let registrations = filter_registrations_with_config(registrations, opts, Some(&config));
    let analyzer = build_analyzer_from_registrations(config, filters, registrations)?;

    println!("\nEligible Checkers:\n");
    println!("{:<8} {:<30} {:<10} Categories", "Code", "Name", "Severity");
    println!("{}", "-".repeat(80));

    for c in analyzer.enabled_checkers() {
        let severity_str = match c.severity {
            1 => "1 (Low)",
            2 => "2 (Medium)",
            3 => "3 (High)",
            _ => "Unknown",
        };
        println!(
            "{:<8} {:<30} {:<10} {}",
            c.code,
            c.name,
            severity_str,
            c.categories.join(", ")
        );
    }

    println!("\nTotal: {} checkers\n", analyzer.checker_count());
    Ok(())
}

/// Print condensed guidelines for LLMs based on enabled checkers.
pub fn print_guidelines_from_registrations(
    opts: &CliOptions,
    registrations: Vec<CheckerRegistration>,
) -> Result<()> {
    // Load config (same as run_cli)
    let config = load_config(&opts.config_path)?;

    // Build filters (severity/category already populated in opts)
    let mut filters = AnalyzerFilters::default();
    if let Some(sev) = opts.severity {
        filters.min_severity = Some(sev);
    }
    if let Some(cats) = opts.categories.clone() {
        filters.categories = Some(cats);
    }

    let registrations = filter_registrations_with_config(registrations, opts, Some(&config));
    let analyzer = build_analyzer_from_registrations(config, filters, registrations)?;

    println!("Do not use the following patterns:\n");

    // Get guidelines from the analyzer's enabled checkers
    for guideline in analyzer.enabled_guidelines() {
        println!(
            "- {} - {} - {}",
            guideline.code, guideline.name, guideline.suggestions
        );
    }

    println!("\nTotal: {} guidelines", analyzer.checker_count());
    Ok(())
}

/// Main CLI runner that handles the full analysis workflow.
pub fn run_cli<F>(opts: CliOptions, make_registrations: F) -> Result<()>
where
    F: FnOnce() -> Vec<CheckerRegistration>,
{
    // 1. Load configuration
    let config = load_config(&opts.config_path)?;

    // 2. Build filters
    let mut filters = AnalyzerFilters::default();
    if let Some(sev) = opts.severity {
        filters.min_severity = Some(sev);
    }
    if let Some(ref cats) = opts.categories {
        filters.categories = Some(cats.clone());
    }
    filters.check_tests = opts.check_tests;

    // 3. Get ALL registrations first for config validation
    let all_registrations = make_registrations();

    // 4. Validate config against ALL known checkers (before CLI filtering)
    validate_config_against_registrations(&config, &all_registrations)?;

    // 5. Apply CLI include/exclude and config category filtering
    let registrations =
        filter_registrations_with_config(all_registrations, &opts, Some(&config));

    // 6. Build analyzer with filtered registrations (skip validation since we already did it)
    let analyzer = build_analyzer_from_registrations_skip_validation(config, filters, registrations)?;

    // 4. Print enabled checkers (non-verbose mode)
    if opts.verbose == 0 {
        eprintln!("Analyzing: {}", opts.source.display());
        eprintln!("\nEnabled checkers ({}):", analyzer.checker_count());
        for checker in analyzer.enabled_checkers() {
            let severity_str = match checker.severity {
                1 => "Low",
                2 => "Med",
                3 => "High",
                _ => "?",
            };
            eprintln!(
                "  - {} - {} [{}] ({})",
                checker.code,
                checker.name,
                severity_str,
                checker.categories.join(", "),
            );
        }
        eprintln!();
    }

    // 5. Run analysis (verbose vs normal)
    let violations = if opts.verbose > 0 {
        analyze_with_verbose(&analyzer, &opts.source, opts.verbose)?
    } else {
        analyzer.analyze(&opts.source)?
    };

    // 6. Sort violations by file path, then by code
    let mut violations = violations;
    violations.sort_by(|a, b| {
        a.file_path
            .cmp(&b.file_path)
            .then_with(|| a.code.cmp(&b.code))
    });

    // 7. Count LOC for quality score
    let total_loc = count_total_loc(&opts.source);

    // 8. Output formatting
    match opts.format {
        CliOutputFormat::Json => {
            println!("{}", serde_json::to_string_pretty(&violations)?);
        }
        CliOutputFormat::Text => {
            // Always show detailed violations
            if !violations.is_empty() {
                println!("\nFound {} violation(s):\n", violations.len());
                for v in &violations {
                    println!(
                        "[{}] {} - {}",
                        v.code,
                        v.name,
                        match v.severity {
                            crate::violation::Severity::High => "HIGH",
                            crate::violation::Severity::Medium => "MEDIUM",
                            crate::violation::Severity::Low => "LOW",
                        }
                    );
                    println!("  File: {}:{}", v.file_path, v.line);
                    println!("  {}", v.message);
                    if let Some(suggestion) = &v.suggestion {
                        println!("  Suggestion: {}", suggestion);
                    }
                    println!();
                }
            }

            // Always print the analysis report
            print_structured_report(&violations, total_loc);
        }
    }

    Ok(())
}

/// Print a structured analysis report grouped by severity
fn print_structured_report(violations: &[Violation], total_loc: usize) {
    use std::collections::HashMap;
    use crate::violation::Severity;

    if violations.is_empty() {
        println!("\nOK. No violations found!");
        return;
    }

    // Group violations by severity and code
    let mut high_violations: HashMap<String, Vec<&Violation>> = HashMap::new();
    let mut medium_violations: HashMap<String, Vec<&Violation>> = HashMap::new();
    let mut low_violations: HashMap<String, Vec<&Violation>> = HashMap::new();

    for v in violations {
        let map = match v.severity {
            Severity::High => &mut high_violations,
            Severity::Medium => &mut medium_violations,
            Severity::Low => &mut low_violations,
        };
        map.entry(v.code.clone()).or_insert_with(Vec::new).push(v);
    }

    // Print header
    println!("\n===================================================================================");
    println!("                                  ANALYSIS REPORT");
    println!("===================================================================================\n");

    println!("Scanned: {} lines of code", total_loc);
    println!("Found: {} violation(s)\n", violations.len());

    // Helper to print a severity section
    let print_section = |title: &str, map: &HashMap<String, Vec<&Violation>>| {
        if map.is_empty() {
            return;
        }

        let total: usize = map.values().map(|v| v.len()).sum();
        println!("{} Severity: {} errors", title, total);

        // Sort codes alphabetically
        let mut codes: Vec<_> = map.keys().collect();
        codes.sort();

        for code in codes {
            if let Some(violations) = map.get(code) {
                // Use first violation for name
                if let Some(first) = violations.first() {
                    println!("  * {} - {}: {}", code, first.name, violations.len());
                }
            }
        }
        println!();
    };

    // Print sections in order: HIGH, MEDIUM, LOW
    print_section("HIGH", &high_violations);
    print_section("MEDIUM", &medium_violations);
    print_section("LOW", &low_violations);

    // Calculate and print quality score
    println!("===================================================================================");
    let score = if violations.is_empty() {
        0.0
    } else {
        total_loc as f64 / violations.len() as f64
    };
    println!("QUALITY SCORE: {:.1} LOC/error", score);
    println!("===================================================================================\n");
}

/// Count total lines of code in analyzed files
fn count_total_loc(path: &PathBuf) -> usize {
    use walkdir::WalkDir;
    use std::fs;

    let mut total = 0;

    for entry in WalkDir::new(path)
        .follow_links(true)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let path = entry.path();
        if path.is_file() {
            if let Some(ext) = path.extension() {
                if ext == "rs" {
                    if let Ok(content) = fs::read_to_string(path) {
                        total += content.lines().count();
                    }
                }
            }
        }
    }

    total
}

/// Analyze with verbose output showing detailed progress.
pub fn analyze_with_verbose(
    analyzer: &Analyzer,
    path: &PathBuf,
    verbose_level: u8,
) -> Result<Vec<Violation>> {
    use walkdir::WalkDir;

    let mut all_violations = Vec::new();
    let mut files_analyzed = 0;
    let mut total_items = 0;

    eprintln!("\nVerbose Analysis (level: {})\n", verbose_level);

    eprintln!("Enabled checkers ({}):", analyzer.checker_count());
    for checker in analyzer.enabled_checkers() {
        let severity_str = match checker.severity {
            1 => "Low",
            2 => "Med",
            3 => "High",
            _ => "?",
        };
        eprintln!(
            "  - {} - {} [{}] ({})",
            checker.code,
            checker.name,
            severity_str,
            checker.categories.join(", ")
        );
    }
    eprintln!();

    let paths: Vec<_> = if path.is_file() {
        vec![path.to_path_buf()]
    } else {
        WalkDir::new(path)
            .follow_links(true)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| {
                e.path().extension().and_then(|s| s.to_str()) == Some("rs")
                    && !e.path().components().any(|c| c.as_os_str() == "target")
            })
            .map(|e| e.path().to_path_buf())
            .collect()
    };

    for file_path in paths {
        files_analyzed += 1;

        // Info mode (-v): print file being analyzed
        if verbose_level >= 1 {
            eprintln!("Analyzing: {}", file_path.display());
        }

        // Debug mode (-vv): parse and show AST items
        if verbose_level >= 2 {
            if let Ok(content) = std::fs::read_to_string(&file_path) {
                if let Ok(syntax) = syn::parse_file(&content) {
                    let items = &syntax.items;
                    total_items += items.len();

                    eprintln!("   Found {} items:", items.len());
                    for item in items {
                        let item_desc = match item {
                            syn::Item::Fn(f) => format!("   - fn {}", f.sig.ident),
                            syn::Item::Struct(s) => format!("   - struct {}", s.ident),
                            syn::Item::Enum(e) => format!("   - enum {}", e.ident),
                            syn::Item::Trait(t) => format!("   - trait {}", t.ident),
                            syn::Item::Impl(i) => {
                                if let Some((_, path, _)) = &i.trait_ {
                                    format!("   - impl {} for ...", quote::quote!(#path))
                                } else if let syn::Type::Path(p) = &*i.self_ty {
                                    format!("   - impl {}", quote::quote!(#p))
                                } else {
                                    "   - impl ...".to_string()
                                }
                            }
                            syn::Item::Mod(m) => format!("   - mod {}", m.ident),
                            syn::Item::Const(c) => format!("   - const {}", c.ident),
                            syn::Item::Static(s) => format!("   - static {}", s.ident),
                            syn::Item::Type(t) => format!("   - type {}", t.ident),
                            syn::Item::Use(u) => {
                                format!(
                                    "   - use {}",
                                    quote::quote!(#u).to_string().trim_end_matches(';')
                                )
                            }
                            _ => "   - (other item)".to_string(),
                        };
                        eprintln!("{}", item_desc);
                    }
                }
            }
        }

        // Run analysis
        match analyzer.analyze(&file_path) {
            Ok(violations) => {
                if verbose_level >= 1 && !violations.is_empty() {
                    eprintln!("   Found {} violation(s)", violations.len());
                }
                all_violations.extend(violations);
            }
            Err(e) => {
                eprintln!("   ERROR: {}", e);
            }
        }

        if verbose_level >= 1 {
            eprintln!();
        }
    }

    // Summary
    if verbose_level >= 1 {
        eprintln!("\nSummary:");
        eprintln!("  Files analyzed: {}", files_analyzed);
        if verbose_level >= 2 {
            eprintln!("  Total AST items: {}", total_items);
        }
        eprintln!("  Total violations: {}\n", all_violations.len());
    }

    // Sort violations by file path, then by code
    all_violations.sort_by(|a, b| {
        a.file_path
            .cmp(&b.file_path)
            .then_with(|| a.code.cmp(&b.code))
    });

    Ok(all_violations)
}

// =============================================================================
// Configuration Discovery
// =============================================================================

/// Find Hyp.toml by searching current directory and parent directories.
///
/// Returns the path to the first `Hyp.toml` found, or a default path if none exists.
pub fn find_config_file() -> PathBuf {
    let mut current = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));

    loop {
        let config_path = current.join("Hyp.toml");
        if config_path.exists() {
            return config_path;
        }

        if !current.pop() {
            // Reached filesystem root, return default (non-existent) path
            return PathBuf::from("Hyp.toml");
        }
    }
}

/// Print the effective configuration in TOML format.
///
/// This generates a complete TOML configuration showing all available checkers
/// with their default settings. Useful for users to see what can be configured.
pub fn print_default_config<F>(
    registrations_fn: F,
    include: Option<&str>,
    exclude: Option<&str>,
    severity: Option<u8>,
) -> Result<()>
where
    F: FnOnce() -> Vec<CheckerRegistration>,
{
    let registrations = registrations_fn();

    println!("# Hyp Configuration");
    println!("# Place this in Hyp.toml to customize checker behavior");
    println!();
    println!("[checkers]");

    for reg in &registrations {
        let desc = &reg.descriptor;
        let code = desc.code.to_lowercase();
        let name: String = desc
            .name
            .to_lowercase()
            .chars()
            .map(|c| if c.is_alphanumeric() { c } else { '_' })
            .collect();
        let key = format!("{}_{}", code, name.chars().take(30).collect::<String>());

        // Apply filters
        if let Some(min_sev) = severity {
            if desc.default_severity.as_u8() < min_sev {
                continue;
            }
        }
        if let Some(include_str) = include {
            let patterns: Vec<&str> = include_str.split(',').map(|s| s.trim()).collect();
            if !patterns.iter().any(|p| code.contains(&p.to_lowercase())) {
                continue;
            }
        }
        if let Some(exclude_str) = exclude {
            let patterns: Vec<&str> = exclude_str.split(',').map(|s| s.trim()).collect();
            if patterns.iter().any(|p| code.contains(&p.to_lowercase())) {
                continue;
            }
        }

        println!();
        println!("# {} - {}", desc.code, desc.name);
        println!("{}.enabled = true", key);
        println!("{}.severity = {}", key, desc.default_severity.as_u8());
        println!("{}.categories = {:?}", key, desc.default_categories);
    }

    Ok(())
}

// =============================================================================
// Example Validation
// =============================================================================

/// Detailed information about a single violation
#[derive(Debug, Clone)]
pub struct ViolationDetail {
    /// Error code (e.g., "E1001")
    pub code: String,
    /// Human-readable violation name
    pub name: String,
    /// Line number where violation occurred
    pub line: usize,
}

/// Result of validating a single function in problem examples.
#[derive(Debug)]
pub struct FunctionValidation {
    /// File path containing the function.
    pub file: String,
    /// Function name.
    pub function: String,
    /// Line number where function starts.
    pub line_number: usize,
    /// Expected error code (e.g., "E1001").
    pub error_code: String,
    /// Whether detection is expected (true for bad_ functions, false for good_).
    pub expected_detection: bool,
    /// Error codes actually detected in this function.
    pub detected_codes: Vec<String>,
    /// Detailed violation information with names and line numbers
    pub detected_violations: Vec<ViolationDetail>,
}

impl FunctionValidation {
    /// Check if this validation passed.
    pub fn is_valid(&self) -> bool {
        if self.expected_detection {
            // For bad functions: should detect at least the expected error code
            self.detected_codes
                .iter()
                .any(|c| c.to_uppercase() == self.error_code.to_uppercase())
        } else {
            // For good functions: must have ZERO violations of ANY kind
            // Good examples should demonstrate truly safe code
            self.detected_codes.is_empty()
        }
    }
}

/// Summary of validation results.
#[derive(Debug, Default)]
pub struct ValidationSummary {
    /// Total files processed.
    pub files_processed: usize,
    /// Total functions validated.
    pub total_functions: usize,
    /// Bad functions correctly detected.
    pub bad_passed: usize,
    /// Total bad functions.
    pub bad_total: usize,
    /// Good functions correctly ignored.
    pub good_passed: usize,
    /// Total good functions.
    pub good_total: usize,
    /// Functions where bad code wasn't detected.
    pub bad_not_detected: Vec<FunctionValidation>,
    /// Functions where good code was incorrectly flagged.
    pub good_incorrectly_detected: Vec<FunctionValidation>,
}

impl ValidationSummary {
    /// Total number of issues (failures).
    pub fn total_issues(&self) -> usize {
        self.bad_not_detected.len() + self.good_incorrectly_detected.len()
    }

    /// Whether all validations passed.
    pub fn all_passed(&self) -> bool {
        self.total_issues() == 0
    }
}

/// Run validation on problem examples to verify checkers work correctly.
///
/// This validates that:
/// - All `eXXXX_bad_*` functions trigger at least the corresponding EXXXX error
/// - All `eXXXX_good_*` functions do NOT trigger any EXXXX error
///
/// # Arguments
/// * `source` - Path to the hyp-examples source directory
/// * `registrations_fn` - Function that returns checker registrations to use
///
/// # Returns
/// A `ValidationSummary` containing all results.
pub fn run_validation<F>(source: &Path, registrations_fn: F) -> Result<ValidationSummary>
where
    F: FnOnce() -> Vec<CheckerRegistration>,
{
    // Collect all Rust files
    let rust_files: Vec<_> = WalkDir::new(source)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| {
            e.path().extension().is_some_and(|ext| ext == "rs")
                && !e.path().ends_with("mod.rs")
                && !e.path().ends_with("lib.rs")
        })
        .collect();

    if rust_files.is_empty() {
        return Ok(ValidationSummary::default());
    }

    // Create analyzer with all checkers enabled
    let config = AnalyzerConfig::default();
    let registrations = registrations_fn();
    let analyzer = Analyzer::new_with_checkers(config, AnalyzerFilters::default(), registrations)?;

    let mut all_validations: Vec<FunctionValidation> = Vec::new();
    let mut files_processed = 0;

    for entry in &rust_files {
        let path = entry.path();
        let file_content = match std::fs::read_to_string(path) {
            Ok(c) => c,
            Err(_) => continue,
        };

        // Parse the file to find function names
        let functions = extract_function_names(&file_content);
        if functions.is_empty() {
            continue;
        }

        files_processed += 1;

        // Run analyzer on the file
        let violations = match analyzer.analyze_file(path) {
            Ok(v) => v,
            Err(_) => continue,
        };

        // Build a map of line -> detected error codes
        let mut line_to_codes: HashMap<usize, Vec<String>> = HashMap::new();
        // Also build a map of line -> full violation details
        let mut line_to_violations: HashMap<usize, Vec<&Violation>> = HashMap::new();
        for v in &violations {
            line_to_codes
                .entry(v.line)
                .or_default()
                .push(v.code.clone());
            line_to_violations
                .entry(v.line)
                .or_default()
                .push(v);
        }

        // Build a map of function name -> lines
        let function_lines = map_functions_to_lines(&file_content, &functions);

        // Build a set of lines that are inside any function
        let mut lines_inside_functions: HashSet<usize> = HashSet::new();
        for (start_line, end_line) in function_lines.values() {
            for line in *start_line..=*end_line {
                lines_inside_functions.insert(line);
            }
        }

        // Collect module-level violations (outside any function)
        let mut module_level_codes: HashSet<String> = HashSet::new();
        for v in &violations {
            if !lines_inside_functions.contains(&v.line) {
                module_level_codes.insert(v.code.clone());
            }
        }

        // Check each function
        for (func_name, (start_line, end_line)) in &function_lines {
            // Extract error code from function name
            let error_code = match extract_error_code(func_name) {
                Some(code) => code,
                None => continue,
            };

            // Determine if this is a bad or good function
            let is_bad = func_name.contains("_bad_") && !func_name.ends_with("_entry");
            let is_good = func_name.contains("_good_");

            if !is_bad && !is_good {
                continue;
            }

            // Collect all error codes and violation details within this function's line range
            let mut detected_codes: Vec<String> = Vec::new();
            let mut detected_violations: Vec<ViolationDetail> = Vec::new();

            for line in *start_line..=*end_line {
                if let Some(codes) = line_to_codes.get(&line) {
                    detected_codes.extend(codes.iter().cloned());
                }
                if let Some(viols) = line_to_violations.get(&line) {
                    for v in viols {
                        detected_violations.push(ViolationDetail {
                            code: v.code.clone(),
                            name: v.name.clone(),
                            line: v.line,
                        });
                    }
                }
            }

            // For bad functions: also include module-level violations that match the expected code
            // This handles patterns like `unsafe impl Send` that exist outside functions
            if is_bad && module_level_codes.contains(&error_code.to_uppercase()) {
                detected_codes.push(error_code.clone());
            }

            // Remove duplicate codes
            let detected_codes: Vec<String> = detected_codes
                .into_iter()
                .collect::<HashSet<_>>()
                .into_iter()
                .collect();

            // Sort violations by line number for logical reading order
            detected_violations.sort_by_key(|v| v.line);

            all_validations.push(FunctionValidation {
                file: path.display().to_string(),
                function: func_name.clone(),
                line_number: *start_line,
                error_code,
                expected_detection: is_bad,
                detected_codes,
                detected_violations,
            });
        }
    }

    // Build summary
    let mut summary = ValidationSummary {
        files_processed,
        total_functions: all_validations.len(),
        ..Default::default()
    };

    for v in all_validations {
        if v.expected_detection {
            summary.bad_total += 1;
            if v.is_valid() {
                summary.bad_passed += 1;
            } else {
                summary.bad_not_detected.push(v);
            }
        } else {
            summary.good_total += 1;
            if v.is_valid() {
                summary.good_passed += 1;
            } else {
                summary.good_incorrectly_detected.push(v);
            }
        }
    }

    Ok(summary)
}

/// Print validation results to stdout.
pub fn print_validation_results(summary: &ValidationSummary) {
    // Print violations
    if !summary.bad_not_detected.is_empty() {
        println!(
            "❌ BAD FUNCTIONS NOT DETECTED ({}):",
            summary.bad_not_detected.len()
        );
        println!("   These eXXXX_bad_* functions should trigger EXXXX but didn't:\n");
        for v in &summary.bad_not_detected {
            println!("   * {}:{}", v.file, v.line_number);
            println!("     Function: {}", v.function);
            println!("     Expected: {} error", v.error_code.to_uppercase());

            if v.detected_violations.is_empty() {
                println!("     Detected: (none)");
            } else {
                println!("     Detected violations:");
                for violation in &v.detected_violations {
                    println!("       - {} - {} - line: {}",
                        violation.code, violation.name, violation.line);
                }
            }
            println!();
        }
    }

    if !summary.good_incorrectly_detected.is_empty() {
        println!(
            "❌ GOOD FUNCTIONS INCORRECTLY DETECTED ({}):",
            summary.good_incorrectly_detected.len()
        );
        println!("   These eXXXX_good_* functions should have ZERO violations but had:\n");
        for v in &summary.good_incorrectly_detected {
            println!("   * {}:{}", v.file, v.line_number);
            println!("     Function: {}", v.function);

            if v.detected_violations.is_empty() {
                println!("     Unexpected violations detected: (none)");
            } else {
                println!("     Unexpected violations detected:");
                for violation in &v.detected_violations {
                    println!("       - {} - {} - line: {}",
                        violation.code, violation.name, violation.line);
                }
            }
            println!();
        }
    }

    // Summary
    println!("===================================================================================");
    println!("                                 VALIDATION SUMMARY");
    println!(
        "===================================================================================\n"
    );

    println!("Files processed: {}", summary.files_processed);
    println!("Total functions checked: {}\n", summary.total_functions);

    if summary.bad_total > 0 {
        println!(
            "Bad functions (eXXXX_bad_*):   {} out of {} detected ({}%)",
            summary.bad_passed,
            summary.bad_total,
            (summary.bad_passed as f64 / summary.bad_total as f64 * 100.0) as u8
        );
    }
    if summary.good_total > 0 {
        println!(
            "Good functions (eXXXX_good_*): {} out of {} passed checks ({}%)\n",
            summary.good_passed,
            summary.good_total,
            (summary.good_passed as f64 / summary.good_total as f64 * 100.0) as u8
        );
    }

    if summary.all_passed() {
        println!("OK. All validations passed!");
    } else {
        println!("ALERT: {} validation issues found", summary.total_issues());
        println!("\nTo fix these issues:");
        if !summary.bad_not_detected.is_empty() {
            println!("  * Implement or fix hyp checkers to detect the missing patterns");
            println!(
                "  * Or update the _bad_ functions if they don't actually demonstrate the problem"
            );
        }
        if !summary.good_incorrectly_detected.is_empty() {
            println!("  * Fix hyp checkers that are too aggressive");
            println!("  * Or rename _good_ functions if they actually contain problems");
        }
    }
}

// =============================================================================
// Internal helpers for validation
// =============================================================================

/// Extract function names from source code using syn.
fn extract_function_names(content: &str) -> Vec<String> {
    let mut functions = Vec::new();

    if let Ok(file) = syn::parse_file(content) {
        for item in &file.items {
            if let syn::Item::Fn(func) = item {
                functions.push(func.sig.ident.to_string());
            }
            if let syn::Item::Impl(impl_block) = item {
                for item in &impl_block.items {
                    if let syn::ImplItem::Fn(method) = item {
                        functions.push(method.sig.ident.to_string());
                    }
                }
            }
        }
    }

    functions
}

/// Extract error code from function name (e.g., "e1001_bad_panic" -> "E1001").
fn extract_error_code(func_name: &str) -> Option<String> {
    // Look for pattern like e1001, e1234, etc.
    let re = regex::Regex::new(r"e(\d{4})").ok()?;
    if let Some(caps) = re.captures(func_name) {
        if let Some(m) = caps.get(0) {
            return Some(m.as_str().to_uppercase());
        }
    }
    None
}

/// Map function names to their line ranges in the source.
fn map_functions_to_lines(content: &str, functions: &[String]) -> HashMap<String, (usize, usize)> {
    let mut result = HashMap::new();

    if let Ok(file) = syn::parse_file(content) {
        for item in &file.items {
            if let syn::Item::Fn(func) = item {
                let name = func.sig.ident.to_string();
                if functions.contains(&name) {
                    let start = func.sig.ident.span().start().line;
                    let end = estimate_function_end(content, start);
                    result.insert(name, (start, end));
                }
            }
            if let syn::Item::Impl(impl_block) = item {
                for item in &impl_block.items {
                    if let syn::ImplItem::Fn(method) = item {
                        let name = method.sig.ident.to_string();
                        if functions.contains(&name) {
                            let start = method.sig.ident.span().start().line;
                            let end = estimate_function_end(content, start);
                            result.insert(name, (start, end));
                        }
                    }
                }
            }
        }
    }

    result
}

/// Estimate the end line of a function by counting braces.
fn estimate_function_end(content: &str, start_line: usize) -> usize {
    let lines: Vec<&str> = content.lines().collect();
    let mut brace_count = 0;
    let mut found_first_brace = false;

    for (i, line) in lines.iter().enumerate().skip(start_line.saturating_sub(1)) {
        for ch in line.chars() {
            if ch == '{' {
                brace_count += 1;
                found_first_brace = true;
            } else if ch == '}' {
                brace_count -= 1;
            }
        }

        if found_first_brace && brace_count == 0 {
            return i + 1; // 1-indexed
        }
    }

    // If we can't find the end, return a reasonable estimate
    (start_line + 50).min(lines.len())
}

/// Helper to split comma-separated string into Vec.
pub fn split_csv(s: &str) -> Vec<String> {
    s.split(',')
        .map(|p| p.trim().to_string())
        .filter(|p| !p.is_empty())
        .collect()
}

/// Parse category string into HashSet.
pub fn parse_categories(category: &Option<String>) -> Option<HashSet<CheckerCategory>> {
    if let Some(cat_str) = category {
        let set: HashSet<CheckerCategory> = cat_str
            .split(',')
            .filter_map(|s| {
                let trimmed_s = s.trim();
                if let Some(category) = CheckerCategory::parse_category(trimmed_s) {
                    Some(category)
                } else {
                    eprintln!("Warning: Unknown category '{}', ignoring", trimmed_s);
                    eprintln!("Valid categories: operations, complexity, compliance");
                    None
                }
            })
            .collect();
        if set.is_empty() {
            None
        } else {
            Some(set)
        }
    } else {
        None
    }
}
