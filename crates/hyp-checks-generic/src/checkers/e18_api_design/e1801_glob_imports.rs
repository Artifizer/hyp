//! E1801: Glob imports
//!
//! Detects use of glob imports (use foo::*) which can pollute the namespace
//! and make code harder to understand.

use crate::{define_checker, violation::Violation};

use syn::spanned::Spanned;

define_checker! {
    /// Checker for E1801: Glob imports
    E1801GlobImports,
    code = "E1801",
    name = "Glob imports",
    suggestions = "Import specific items instead of using glob imports",
    target_items = [Use],
    config_entry_name = "e1801_glob_imports",
    config = E1801Config {
        enabled: bool = true,
        severity: crate::config::SeverityLevel = crate::config::SeverityLevel::Low,
        categories: Vec<crate::config::CheckerCategory> = vec![crate::config::CheckerCategory::Complexity],
        /// Allow glob imports from these modules (e.g., prelude)
        allowed_globs: Vec<String> = vec!["prelude".to_string()],
    },
    check_item(self, item, file_path) {
        let mut violations = Vec::new();

        if let syn::Item::Use(use_item) = item {
            check_use_tree(&use_item.tree, file_path, self, &mut violations);
        }

        Ok(violations)
    }
}

fn check_use_tree(
    tree: &syn::UseTree,
    file_path: &str,
    checker: &E1801GlobImports,
    violations: &mut Vec<Violation>,
) {
    use crate::checker::Checker;

    match tree {
        syn::UseTree::Glob(glob) => {
            let start = glob.span().start();
            violations.push(
                Violation::new(
                    checker.code(),
                    checker.name(),
                    checker.severity().into(),
                    "Glob import (use foo::*) can pollute namespace. Import specific items instead.",
                    file_path,
                    start.line,
                    start.column + 1,
                )
                .with_suggestion(checker.suggestions()),
            );
        }
        syn::UseTree::Path(path) => {
            // Check if this is an allowed glob path (e.g., prelude)
            let path_name = path.ident.to_string();
            if checker.config.allowed_globs.contains(&path_name) {
                return;
            }
            check_use_tree(&path.tree, file_path, checker, violations);
        }
        syn::UseTree::Group(group) => {
            for item in &group.items {
                check_use_tree(item, file_path, checker, violations);
            }
        }
        _ => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::checker::Checker;

    #[test]
    fn test_detects_glob_import() {
        let code = r#"
            use std::collections::*;
        "#;

        let syntax = syn::parse_file(code).unwrap();
        let checker = E1801GlobImports::default();

        let mut violations = Vec::new();
        for item in &syntax.items {
            violations.extend(checker.check_item(item, "test.rs").unwrap());
        }

        assert_eq!(violations.len(), 1);
    }

    #[test]
    fn test_specific_import_passes() {
        let code = r#"
            use std::collections::HashMap;
        "#;

        let syntax = syn::parse_file(code).unwrap();
        let checker = E1801GlobImports::default();

        let mut violations = Vec::new();
        for item in &syntax.items {
            violations.extend(checker.check_item(item, "test.rs").unwrap());
        }

        assert_eq!(violations.len(), 0);
    }
}
