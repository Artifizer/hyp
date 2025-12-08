/// E1903: File location control
/// Severity: MEDIUM
/// LLM confusion: 2 (LOW)
///
/// Description: Enforces that specific files can only exist in designated locations.
/// Useful for ensuring configuration files, build scripts, or special modules
/// are in their proper places according to project structure.
///
/// ## Why This Matters
///
/// 1. **Project structure**: Maintains consistent file organization
/// 2. **Tool expectations**: Many tools expect config files in specific locations
/// 3. **Build reproducibility**: Ensures build configs are where build systems expect them
/// 4. **Team navigation**: Makes it easy to find files in expected locations
///
/// ## The Right Solutions
///
/// ### Option 1: Place config files at project root
/// ```
/// project/
/// ├── Cargo.toml
/// ├── Clippy.toml      ✓ At root
/// ├── rustfmt.toml     ✓ At root
/// └── src/
///     └── main.rs
/// ```
///
/// ### Option 2: Follow conventions for special files
/// ```
/// project/
/// ├── proto/           ✓ Proto files here
/// │   └── user.proto
/// ├── migrations/      ✓ SQL migrations here
/// │   └── 001_init.sql
/// └── src/
///     └── main.rs
/// ```
///
/// ### Option 3: Use standard directory structure
/// ```
/// project/
/// ├── src/             ✓ Source code
/// ├── tests/           ✓ Integration tests
/// ├── benches/         ✓ Benchmarks
/// └── examples/        ✓ Examples
/// ```
///
/// Mitigation: Configure E1903 in Hyp.toml to specify filename patterns and
/// their allowed paths. Use regex for flexible matching.

// ============================================================================
// PROBLEMATIC PATTERNS
// ============================================================================
// Note: Functions use snake_case following Rust naming conventions:
// e1903_bad_config_location_example, e1903_good_root_config, etc.

/// PROBLEM E1903: This file demonstrates the concept
/// In reality, E1903 checks file paths, not code content
/// If this file were named "Clippy.toml" and located in src/config/,
/// it would be flagged by E1903

/// PROBLEM E1903: Configuration file in wrong location
/// Example: src/config/Clippy.toml instead of ./Clippy.toml
pub fn e1903_bad_config_location_example() {
    // This function represents the concept that Clippy.toml
    // should be at project root, not in subdirectories
    let _ = "Clippy.toml in src/config/ would be flagged";
}

/// PROBLEM E1903: Proto file outside proto/ directory
/// Example: src/api/user.proto instead of proto/user.proto
pub fn e1903_bad_proto_location_example() {
    let _ = "user.proto in src/api/ would be flagged";
}

/// PROBLEM E1903: Build script in wrong location
/// Example: src/build.rs instead of ./build.rs
pub fn e1903_bad_build_script_location() {
    let _ = "build.rs in src/ would be flagged";
}

/// PROBLEM E1903: Migration file in wrong location
/// Example: src/migrations/001_init.sql instead of migrations/001_init.sql
pub fn e1903_bad_migration_location() {
    let _ = "SQL migration in src/ would be flagged";
}

/// Entry point for problem demonstration
pub fn e1903_entry() -> Result<(), Box<dyn std::error::Error>> {
    e1903_bad_config_location_example();
    e1903_bad_proto_location_example();
    e1903_bad_build_script_location();
    e1903_bad_migration_location();
    Ok(())
}

// ============================================================================
// GOOD ALTERNATIVES
// ============================================================================

/// GOOD: Configuration files at project root
pub fn e1903_good_root_config() {
    // Clippy.toml at ./Clippy.toml ✓
    // rustfmt.toml at ./rustfmt.toml ✓
    let _ = "Config files at project root";
}

/// GOOD: Proto files in proto/ directory
pub fn e1903_good_proto_location() {
    // proto/user.proto ✓
    // proto/order.proto ✓
    let _ = "Proto files in proto/ directory";
}

/// GOOD: Build scripts at project root
pub fn e1903_good_build_script() {
    // ./build.rs ✓
    let _ = "build.rs at project root";
}

/// GOOD: Migrations in migrations/ directory
pub fn e1903_good_migrations() {
    // migrations/001_init.sql ✓
    // migrations/002_add_users.sql ✓
    let _ = "Migrations in migrations/ directory";
}

/// GOOD: Source files in src/
pub fn e1903_good_source_location() {
    // src/main.rs ✓
    // src/lib.rs ✓
    // src/api/mod.rs ✓
    let _ = "Source code in src/ directory";
}

// ============================================================================
// GOOD EXAMPLES unit tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_good_config_concept() {
        e1903_good_root_config();
        // In practice, E1903 would check actual file paths
    }

    #[test]
    fn test_good_proto_concept() {
        e1903_good_proto_location();
    }

    #[test]
    fn test_good_build_concept() {
        e1903_good_build_script();
    }

    #[test]
    fn test_good_migrations_concept() {
        e1903_good_migrations();
    }

    #[test]
    fn test_good_source_concept() {
        e1903_good_source_location();
    }
}
