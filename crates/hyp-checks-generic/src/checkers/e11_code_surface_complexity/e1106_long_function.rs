//! E1106: Long function (too many lines)
//!
//! Detects functions that exceed a configurable line count threshold.
//! Long functions are hard to understand, test, and maintain.

use crate::{define_checker, violation::Violation};

use syn::spanned::Spanned;

define_checker! {
    /// Checker for E1106: Long functions
    E1106LongFunction,
    code = "E1106",
    name = "Long function",
    suggestions = "Extract logical sections into separate helper functions",
    target_items = [Function],
    config_entry_name = "e1106_long_function",
    /// Configuration for E1106: Long function checker
    config = E1106Config {
        /// Whether this checker is enabled
        enabled: bool = true,
        /// Severity level, defaults to Low
        severity: crate::config::SeverityLevel = crate::config::SeverityLevel::Low,
        /// Categories this checker belongs to, defaults to [Complexity]
        categories: Vec<crate::config::CheckerCategory> = vec![crate::config::CheckerCategory::Complexity],
        /// Maximum allowed lines in a function
        max_lines: usize = 250,
    },
    // AST node item checker
    check_item(self, item, file_path) {
        let mut violations = Vec::new();

        if let syn::Item::Fn(func) = item {
            // Count lines in the function body (excluding doc comments/attributes)
            let block_span = func.block.span();
            let block_start = block_span.start();
            let block_end = block_span.end();
            let line_count = block_end.line.saturating_sub(block_start.line) + 1;

            if line_count > self.config.max_lines {
                let func_name = func.sig.ident.to_string();
                // Report the function signature line, not the first doc comment
                let sig_start = func.sig.ident.span().start();
                let message = format!(
                    "Function '{}' has {} lines, exceeding the limit of {}. Long functions are harder to understand and maintain.",
                    func_name, line_count, self.config.max_lines
                );

                violations.push(
                    Violation::new(
                        self.code(),
                        self.name(),
                        self.severity().into(),
                        &message,
                        file_path,
                        sig_start.line,
                        sig_start.column + 1, // Convert 0-indexed to 1-indexed
                    )
                    .with_suggestion(self.suggestions()),
                );
            }
        }

        Ok(violations)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::checker::Checker;

    #[test]
    fn test_detects_long_function() {
        // Create a function with many lines
        let mut code = String::from("fn long_function() {\n");
        for i in 0..300 {
            code.push_str(&format!("    let x{} = {};\n", i, i));
        }
        code.push_str("}\n");

        let syntax = syn::parse_file(&code).unwrap();
        let mut checker = E1106LongFunction::default();
        checker
            .set_config(Box::new(E1106Config {
                enabled: true,
                severity: crate::config::SeverityLevel::Low,
                categories: vec![crate::config::CheckerCategory::Complexity],
                max_lines: 250,
            }))
            .unwrap();

        let mut violations = Vec::new();
        for item in &syntax.items {
            violations.extend(checker.check_item(item, "test.rs").unwrap());
        }

        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].code, "E1106");
        assert!(violations[0].message.contains("long_function"));
    }

    #[test]
    fn test_short_function_passes() {
        let code = r#"
            fn short_function() {
                let x = 1;
                let y = 2;
                x + y
            }
        "#;

        let syntax = syn::parse_file(code).unwrap();
        let checker = E1106LongFunction::default();

        let mut violations = Vec::new();
        for item in &syntax.items {
            violations.extend(checker.check_item(item, "test.rs").unwrap());
        }

        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_custom_threshold() {
        let code = r#"
            fn medium_function() {
                let x = 1;
                let y = 2;
                let z = 3;
                let a = 4;
                let b = 5;
                x + y + z + a + b
            }
        "#;

        let syntax = syn::parse_file(code).unwrap();
        let mut checker = E1106LongFunction::default();
        checker
            .set_config(Box::new(E1106Config {
                enabled: true,
                severity: crate::config::SeverityLevel::Low,
                categories: vec![crate::config::CheckerCategory::Complexity],
                max_lines: 5,
            }))
            .unwrap();

        let mut violations = Vec::new();
        for item in &syntax.items {
            violations.extend(checker.check_item(item, "test.rs").unwrap());
        }

        assert_eq!(violations.len(), 1);
    }
}
