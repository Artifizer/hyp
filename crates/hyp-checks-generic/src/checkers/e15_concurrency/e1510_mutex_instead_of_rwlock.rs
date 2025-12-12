//! E1510: Mutex instead of RwLock
//!
//! Detects use of Mutex where RwLock might be more appropriate
//! (when the protected data is primarily read, not written).

use crate::{checker::Checker, define_checker, violation::Violation};

use syn::visit::Visit;

define_checker! {
    /// Checker for E1510: Mutex instead of RwLock
    E1510MutexInsteadOfRwLock,
    code = "E1510",
    name = "Mutex instead of RwLock",
    suggestions = "Consider using RwLock if data is read more often than written",
    target_items = [Function, Struct],
    config_entry_name = "e1510_mutex_instead_of_rwlock",
    config = E1510Config {
        enabled: bool = true,
        severity: crate::config::SeverityLevel = crate::config::SeverityLevel::Low,
        categories: Vec<crate::config::CheckerCategory> = vec![crate::config::CheckerCategory::Complexity],
    },
    check_item(self, item, file_path) {
        let mut visitor = MutexVisitor {
            violations: Vec::new(),
            file_path,
            checker: self,
        };
        visitor.visit_item(item);
        Ok(visitor.violations)
    }
}

struct MutexVisitor<'a> {
    violations: Vec<Violation>,
    file_path: &'a str,
    checker: &'a E1510MutexInsteadOfRwLock,
}

impl<'a> Visit<'a> for MutexVisitor<'a> {
    fn visit_field(&mut self, node: &'a syn::Field) {
        if type_contains_mutex(&node.ty) {
            use syn::spanned::Spanned;
            let start = node.span().start();
            let field_name = node
                .ident
                .as_ref()
                .map(|i| i.to_string())
                .unwrap_or_else(|| "unnamed".to_string());
            self.violations.push(
                Violation::new(
                    self.checker.code(),
                    self.checker.name(),
                    self.checker.severity().into(),
                    format!(
                        "Field '{}' uses Mutex. Consider RwLock if data is read more often than written.",
                        field_name
                    ),
                    self.file_path,
                    start.line,
                    start.column + 1,
                )
                .with_suggestion(self.checker.suggestions()),
            );
        }

        syn::visit::visit_field(self, node);
    }

    fn visit_local(&mut self, node: &'a syn::Local) {
        // Check let bindings with Mutex type annotation
        if let Some(init) = &node.init {
            if let syn::Expr::Call(call) = &*init.expr {
                if let syn::Expr::Path(path) = &*call.func {
                    let path_str = path
                        .path
                        .segments
                        .iter()
                        .map(|s| s.ident.to_string())
                        .collect::<Vec<_>>()
                        .join("::");

                    if path_str.ends_with("Mutex::new") || path_str == "new" {
                        // Check if previous segment was Mutex
                        if let Some(last) = path.path.segments.iter().rev().nth(1) {
                            if last.ident == "Mutex" {
                                // Check for patterns where Mutex is appropriate:
                                // 1. Accumulator pattern (named with accumulator, counter, etc.)
                                // 2. Counter pattern
                                // 3. Atomic pattern
                                if let syn::Pat::Ident(pat_ident) = &node.pat {
                                    let var_name = pat_ident.ident.to_string().to_lowercase();
                                    // Skip if variable name suggests write-heavy pattern
                                    if var_name.contains("accumulator")
                                        || var_name.contains("counter")
                                        || var_name.contains("atomic")
                                        || var_name.contains("writer")
                                        || var_name.contains("state")
                                    {
                                        syn::visit::visit_local(self, node);
                                        return;
                                    }
                                }

                                use syn::spanned::Spanned;
                                let start = node.span().start();
                                self.violations.push(
                                    Violation::new(
                                        self.checker.code(),
                                        self.checker.name(),
                                        self.checker.severity().into(),
                                        "Using Mutex::new. Consider RwLock if data is read more often than written.",
                                        self.file_path,
                                        start.line,
                                        start.column + 1,
                                    )
                                    .with_suggestion(self.checker.suggestions()),
                                );
                            }
                        }
                    }
                }
            }
        }

        syn::visit::visit_local(self, node);
    }
}

fn type_contains_mutex(ty: &syn::Type) -> bool {
    if let syn::Type::Path(type_path) = ty {
        for segment in &type_path.path.segments {
            if segment.ident == "Mutex" {
                return true;
            }
            // Check generic arguments
            if let syn::PathArguments::AngleBracketed(args) = &segment.arguments {
                for arg in &args.args {
                    if let syn::GenericArgument::Type(inner_ty) = arg {
                        if type_contains_mutex(inner_ty) {
                            return true;
                        }
                    }
                }
            }
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detects_mutex_field() {
        let code = r#"
            use std::sync::Mutex;

            struct MyStruct {
                data: Mutex<Vec<i32>>,
            }
        "#;

        let syntax = syn::parse_file(code).unwrap();
        let checker = E1510MutexInsteadOfRwLock::default();

        let mut violations = Vec::new();
        for item in &syntax.items {
            violations.extend(checker.check_item(item, "test.rs").unwrap());
        }

        assert_eq!(violations.len(), 1);
    }

    #[test]
    fn test_rwlock_passes() {
        let code = r#"
            use std::sync::RwLock;

            struct MyStruct {
                data: RwLock<Vec<i32>>,
            }
        "#;

        let syntax = syn::parse_file(code).unwrap();
        let checker = E1510MutexInsteadOfRwLock::default();

        let mut violations = Vec::new();
        for item in &syntax.items {
            violations.extend(checker.check_item(item, "test.rs").unwrap());
        }

        assert_eq!(violations.len(), 0);
    }
}
