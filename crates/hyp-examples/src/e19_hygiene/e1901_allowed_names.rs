/// E1901: Allowed names and paths control
/// Severity: HIGH
/// LLM confusion: 3 (MEDIUM)
///
/// Description: Enforces project-specific naming and location rules for AST items.
/// Prevents unwanted patterns like DTOs outside API layers, wildcard imports in
/// specific modules, or configuration files in wrong locations.
///
/// ## Why This Matters
///
/// 1. **Architectural boundaries**: Ensures DTOs, models, and other types stay in their designated layers
/// 2. **Import hygiene**: Prevents wildcard imports that can cause name conflicts
/// 3. **Maintainability**: Makes codebase structure predictable and navigable
/// 4. **Team standards**: Enforces naming conventions across the project
///
/// ## The Right Solutions
///
/// ### Option 1: Follow naming conventions
/// ```rust
/// // In src/api/user.rs
/// pub struct UserDTO {
///     id: i32,
///     name: String,
/// }
/// ```
///
/// ### Option 2: Use explicit imports
/// ```rust
/// // Instead of: use sqlx::*;
/// use sqlx::{Pool, Postgres, query};
/// ```
///
/// ### Option 3: Move files to correct locations
/// ```rust
/// // Move DTO definitions from src/models/ to src/api/
/// ```
///
/// Mitigation: Configure E1901 in Hyp.toml with project-specific rules for
/// item names, types, and allowed paths. Use regex patterns for flexibility.

// ============================================================================
// PROBLEMATIC PATTERNS
// ============================================================================
// Note: This file demonstrates both naming conventions:
// - Structs use PascalCase: E1901BadUserDtoInModels, E1901GoodUserModel
// - Functions use snake_case: e1901_bad_wildcard_import_usage, e1901_good_explicit_imports
// Both conventions are supported and follow Rust naming standards.

/// PROBLEM E1901: DTO struct defined outside api/ directory
/// This would be flagged if E1901 is configured to restrict DTOs to api/
pub struct E1901BadUserDtoInModels {
    pub id: i32,
    pub name: String,
}

/// PROBLEM E1901: Request/Response types outside api/
pub struct E1901BadCreateUserRequest {
    pub name: String,
    pub email: String,
}

/// PROBLEM E1901: Wildcard import in restricted location
/// This demonstrates the pattern (actual detection would need use statement)
pub fn e1901_bad_wildcard_import_usage() {
    // If this file had: use sqlx::*;
    // And E1901 was configured to block sqlx::* in api/, it would be flagged
    let _ = "sqlx::* import would be here";
}

/// PROBLEM E1901: Configuration struct in wrong location
pub struct E1901BadAppConfigInModels {
    pub database_url: String,
    pub port: u16,
}

/// Entry point for problem demonstration
pub fn e1901_entry() -> Result<(), Box<dyn std::error::Error>> {
    let _ = E1901BadUserDtoInModels { id: 1, name: "test".to_string() };
    let _ = E1901BadCreateUserRequest { name: "test".to_string(), email: "test@example.com".to_string() };
    e1901_bad_wildcard_import_usage();
    let _ = E1901BadAppConfigInModels { database_url: "".to_string(), port: 8080 };
    Ok(())
}

// ============================================================================
// GOOD ALTERNATIVES
// ============================================================================

/// GOOD: Domain model without DTO suffix (allowed anywhere)
pub struct E1901GoodUserModel {
    pub id: i32,
    pub name: String,
    pub email: String,
}

/// GOOD: Internal struct without restricted naming pattern
pub struct E1901GoodUserData {
    pub id: i32,
    pub name: String,
}

/// GOOD: Explicit imports instead of wildcards
pub fn e1901_good_explicit_imports() {
    // Instead of: use sqlx::*;
    // Use: use sqlx::{Pool, Postgres};
    let _ = "explicit imports";
}

/// GOOD: Configuration in proper location (config/ directory)
pub struct E1901GoodSettings {
    pub database_url: String,
    pub port: u16,
}

// ============================================================================
// GOOD EXAMPLES unit tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_good_model_structure() {
        let user = E1901GoodUserModel {
            id: 1,
            name: "Alice".to_string(),
            email: "alice@example.com".to_string(),
        };
        assert_eq!(user.id, 1);
    }

    #[test]
    fn test_good_data_structure() {
        let data = E1901GoodUserData {
            id: 2,
            name: "Bob".to_string(),
        };
        assert_eq!(data.name, "Bob");
    }

    #[test]
    fn test_good_settings() {
        let settings = E1901GoodSettings {
            database_url: "postgres://localhost".to_string(),
            port: 5432,
        };
        assert_eq!(settings.port, 5432);
    }
}
