//! E1710: Large stack allocation
//!
//! Detects large arrays allocated on the stack which can cause stack overflow,
//! especially in recursive functions or deeply nested calls.

use crate::{define_checker, violation::Violation};

use syn::spanned::Spanned;

define_checker! {
    /// Checker for E1710: Large stack allocation
    E1710LargeStackAllocation,
    code = "E1710",
    name = "Large stack allocation",
    suggestions = "Use Vec or Box for large allocations instead of stack arrays",
    target_items = [Function],
    config_entry_name = "e1710_large_stack_allocation",
    config = E1710Config {
        enabled: bool = true,
        severity: crate::config::SeverityLevel = crate::config::SeverityLevel::Medium,
        categories: Vec<crate::config::CheckerCategory> = vec![crate::config::CheckerCategory::Complexity],
        /// Maximum stack array size in bytes before warning
        max_stack_size: usize = 102400, // 100KB
    },
    check_item(self, item, file_path) {
        let mut violations = Vec::new();

        if let syn::Item::Fn(func) = item {
            // Check for array type declarations in local variables
            for stmt in &func.block.stmts {
                if let syn::Stmt::Local(local) = stmt {
                    if let Some(init) = &local.init {
                        if let syn::Expr::Array(arr) = &*init.expr {
                            if arr.elems.len() > self.config.max_stack_size / 8 {
                                let start = local.span().start();
                                violations.push(
                                    Violation::new(
                                        self.code(),
                                        self.name(),
                                        self.severity().into(),
                                        format!(
                                            "Array with {} elements may be too large for stack allocation.",
                                            arr.elems.len()
                                        ),
                                        file_path,
                                        start.line,
                                        start.column + 1,
                                    )
                                    .with_suggestion(self.suggestions()),
                                );
                            }
                        }
                        // Check for array repeat syntax [0u8; size]
                        if let syn::Expr::Repeat(repeat) = &*init.expr {
                            if let syn::Expr::Lit(lit) = &*repeat.len {
                                if let syn::Lit::Int(int_lit) = &lit.lit {
                                    if let Ok(size) = int_lit.base10_parse::<usize>() {
                                        // Estimate size based on type (assume worst case 8 bytes per element)
                                        if size > self.config.max_stack_size {
                                            let start = local.span().start();
                                            violations.push(
                                                Violation::new(
                                                    self.code(),
                                                    self.name(),
                                                    self.severity().into(),
                                                    format!(
                                                        "Array of {} bytes is too large for stack. Use Vec or Box.",
                                                        size
                                                    ),
                                                    file_path,
                                                    start.line,
                                                    start.column + 1,
                                                )
                                                .with_suggestion(self.suggestions()),
                                            );
                                        }
                                    }
                                }
                            }
                        }
                    }

                    // Check type annotation for arrays
                    if let syn::Pat::Type(pat_type) = &local.pat {
                        if let syn::Type::Array(arr_type) = &*pat_type.ty {
                            if let syn::Expr::Lit(lit) = &arr_type.len {
                                if let syn::Lit::Int(int_lit) = &lit.lit {
                                    if let Ok(size) = int_lit.base10_parse::<usize>() {
                                        if size > self.config.max_stack_size {
                                            let start = local.span().start();
                                            violations.push(
                                                Violation::new(
                                                    self.code(),
                                                    self.name(),
                                                    self.severity().into(),
                                                    format!(
                                                        "Array of {} bytes is too large for stack. Use Vec or Box.",
                                                        size
                                                    ),
                                                    file_path,
                                                    start.line,
                                                    start.column + 1,
                                                )
                                                .with_suggestion(self.suggestions()),
                                            );
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
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
    fn test_detects_large_array() {
        let code = r#"
            fn example() {
                let large = [0u8; 1048576]; // 1MB
            }
        "#;

        let syntax = syn::parse_file(code).unwrap();
        let checker = E1710LargeStackAllocation::default();

        let mut violations = Vec::new();
        for item in &syntax.items {
            violations.extend(checker.check_item(item, "test.rs").unwrap());
        }

        assert_eq!(violations.len(), 1);
    }

    #[test]
    fn test_small_array_passes() {
        let code = r#"
            fn example() {
                let small = [0u8; 1024]; // 1KB
            }
        "#;

        let syntax = syn::parse_file(code).unwrap();
        let checker = E1710LargeStackAllocation::default();

        let mut violations = Vec::new();
        for item in &syntax.items {
            violations.extend(checker.check_item(item, "test.rs").unwrap());
        }

        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_vec_passes() {
        let code = r#"
            fn example() {
                let large = vec![0u8; 1024 * 1024]; // 1MB on heap
            }
        "#;

        let syntax = syn::parse_file(code).unwrap();
        let checker = E1710LargeStackAllocation::default();

        let mut violations = Vec::new();
        for item in &syntax.items {
            violations.extend(checker.check_item(item, "test.rs").unwrap());
        }

        assert_eq!(violations.len(), 0);
    }
}
