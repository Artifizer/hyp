/// E1801: Glob imports
/// Severity: LOW
/// LLM confusion: 2 (LOW)
///
/// Description: Glob imports (`use module::*`) bring all public items from a module into scope,
/// making it unclear where each item comes from. This hurts code readability and can cause name
/// conflicts. Fix by importing items explicitly by name, which makes dependencies clear and
/// prevents accidental name collisions.
///
/// ## The Hidden Dependency Problem
///
/// ```text
/// use std::collections::*;
/// use std::io::*;
///
/// let result = Result::Ok(42);  // Which Result? std::result::Result or io::Result?
/// let error = Error::new(...);  // Which Error?
/// ```
///
/// ## Why This Matters
///
/// 1. **Name collisions**: Multiple modules may export same names
/// 2. **Unclear origin**: Hard to know where items come from
/// 3. **Refactoring hazard**: Adding exports can break downstream code
/// 4. **IDE confusion**: Harder for tools to provide accurate suggestions
///
/// ## The Right Solutions
///
/// ### Option 1: Explicit imports
/// ```rust
/// use std::collections::{HashMap, HashSet};
/// ```
///
/// ### Option 2: Module aliases
/// ```rust
/// use std::collections as coll;
/// coll::HashMap::new()
/// ```
///
/// ### Option 3: Qualified paths
/// ```rust
/// std::collections::HashMap::new()
/// ```
///
/// Mitigation: Use `#![warn(clippy::wildcard_imports)]` to catch glob imports. Import items
/// explicitly: `use std::collections::{HashMap, HashSet}`. Glob imports are acceptable for preludes
/// and test modules, but avoid them in production code.

use std::collections::{HashMap, HashSet};

// ============================================================================
// PROBLEMATIC PATTERNS
// ============================================================================

#[allow(clippy::module_inception)]
#[allow(clippy::wildcard_imports)]
pub mod bad_glob_imports {
    // PROBLEM E1801: Glob imports make it unclear where items come from
    use std::collections::*;

    pub fn e1801_bad_glob() {
        let _map: HashMap<i32, i32> = HashMap::new();
        let _set: HashSet<i32> = HashSet::new();
        // Where do HashMap and HashSet come from? Not obvious!
    }
}

#[allow(clippy::wildcard_imports)]
#[allow(unused_imports)]
pub mod bad_multiple_globs {
    // PROBLEM E1801: Multiple globs can cause conflicts
    use std::collections::*;
    use std::io::*;

    pub fn e1801_bad_multiple() {
        // Both std::result::Result and std::io::Result are in scope!
        let _map: HashMap<i32, i32> = HashMap::new();
        // let _result: Result<i32, Error> = Ok(42); // Which Result? Which Error?
    }
}

/// Entry point for problem demonstration
pub fn e1801_entry() -> Result<(), Box<dyn std::error::Error>> {
    bad_glob_imports::e1801_bad_glob();
    Ok(())
}

// ============================================================================
// GOOD ALTERNATIVES
// ============================================================================

/// GOOD: Explicit imports
pub mod good_explicit {
    use std::collections::{HashMap, HashSet};

    pub fn explicit_imports() {
        let _map: HashMap<i32, i32> = HashMap::new();
        let _set: HashSet<i32> = HashSet::new();
        // Clear where each type comes from
    }
}

/// GOOD: Module alias
pub mod good_alias {
    use std::collections as coll;

    pub fn module_alias() {
        let _map: coll::HashMap<i32, i32> = coll::HashMap::new();
        // Clear that HashMap comes from collections
    }
}

/// GOOD: Qualified paths
pub mod good_qualified {
    pub fn qualified_paths() {
        let _map: std::collections::HashMap<i32, i32> = std::collections::HashMap::new();
        // Fully qualified, no ambiguity
    }
}

// ============================================================================
// GOOD EXAMPLES unit tests
// ============================================================================


/// GOOD: Glob only in tests (acceptable)
#[cfg(test)]
mod tests {
    use super::*; // Glob in tests is acceptable

    #[test]
    fn test_explicit() {
        good_explicit::explicit_imports();
    }

    #[test]
    fn test_alias() {
        good_alias::module_alias();
    }

    #[test]
    fn test_qualified() {
        good_qualified::qualified_paths();
    }
}

/// GOOD: Glob for prelude (acceptable)
pub mod good_prelude {
    // Preludes are designed for glob import
    // use my_crate::prelude::*;
}

/// When globs are acceptable:
/// - Test modules: `use super::*;`
/// - Prelude modules: `use crate::prelude::*;`
/// - Derive macros: `use serde::*;` (sometimes)
/// - Internal modules: `use crate::internal::*;`
pub fn e1801_acceptable_globs() {
    // In production code, prefer explicit imports
    let _: HashMap<String, i32> = HashMap::new();
    let _: HashSet<String> = HashSet::new();
}
