//! E1203: Complicated borrowing patterns
//!
//! Detects functions with complex borrowing patterns that are hard to reason about.
//! This includes:
//! - Multiple mutable references in the same function signature
//! - Functions with many lifetime parameters
//! - Nested references (&'a &'b T)
//!
//! These patterns often indicate code that could be simplified or restructured.

use crate::{define_checker, violation::Violation};
use syn::visit::Visit;

define_checker! {
    /// Checker for E1203: Complicated borrowing patterns
    E1203ComplicatedBorrowing,
    code = "E1203",
    name = "Complicated borrowing patterns",
    suggestions = "Consider restructuring to reduce reference complexity. Use owned types, Cow<T>, or Arc/Rc for shared ownership.",
    target_items = [Function],
    config_entry_name = "e1203_complicated_borrowing",
    config = E1203Config {
        enabled: bool = true,
        severity: crate::config::SeverityLevel = crate::config::SeverityLevel::Medium,
        categories: Vec<crate::config::CheckerCategory> = vec![crate::config::CheckerCategory::Complexity],
        /// Maximum number of lifetime parameters before flagging
        max_lifetimes: usize = 2,
        /// Maximum number of mutable references in parameters
        max_mut_refs: usize = 2,
    },
    check_item(self, item, file_path) {
        let mut violations = Vec::new();

        if let syn::Item::Fn(func) = item {
            // Count lifetime parameters
            let lifetime_count = func.sig.generics.lifetimes().count();

            if lifetime_count > self.config.max_lifetimes {
                let start = func.sig.fn_token.span.start();
                violations.push(
                    Violation::new(
                        self.code(),
                        self.name(),
                        self.severity().into(),
                        format!(
                            "Function has {} lifetime parameters (max {}). Consider simplifying.",
                            lifetime_count, self.config.max_lifetimes
                        ),
                        file_path,
                        start.line,
                        start.column + 1,
                    )
                    .with_suggestion(self.suggestions()),
                );
            }

            // Count mutable references in parameters
            let mut mut_ref_count = 0;
            let mut nested_ref_found = false;

            for param in &func.sig.inputs {
                if let syn::FnArg::Typed(pat_type) = param {
                    let mut visitor = RefVisitor {
                        mut_ref_count: 0,
                        nested_ref: false,
                        depth: 0,
                    };
                    visitor.visit_type(&pat_type.ty);
                    mut_ref_count += visitor.mut_ref_count;
                    if visitor.nested_ref {
                        nested_ref_found = true;
                    }
                }
            }

            if mut_ref_count > self.config.max_mut_refs {
                let start = func.sig.fn_token.span.start();
                violations.push(
                    Violation::new(
                        self.code(),
                        self.name(),
                        self.severity().into(),
                        format!(
                            "Function has {} mutable reference parameters (max {}). This can make the API confusing.",
                            mut_ref_count, self.config.max_mut_refs
                        ),
                        file_path,
                        start.line,
                        start.column + 1,
                    )
                    .with_suggestion(self.suggestions()),
                );
            }

            if nested_ref_found {
                let start = func.sig.fn_token.span.start();
                violations.push(
                    Violation::new(
                        self.code(),
                        self.name(),
                        self.severity().into(),
                        "Function has nested references (&&T or &mut &T). This pattern is usually a code smell.",
                        file_path,
                        start.line,
                        start.column + 1,
                    )
                    .with_suggestion(self.suggestions()),
                );
            }
        }

        Ok(violations)
    }
}

struct RefVisitor {
    mut_ref_count: usize,
    nested_ref: bool,
    depth: usize,
}

impl<'ast> Visit<'ast> for RefVisitor {
    fn visit_type_reference(&mut self, node: &'ast syn::TypeReference) {
        if node.mutability.is_some() {
            self.mut_ref_count += 1;
        }

        // Check for nested references
        if self.depth > 0 {
            self.nested_ref = true;
        }

        self.depth += 1;
        syn::visit::visit_type_reference(self, node);
        self.depth -= 1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::checker::Checker;

    #[test]
    fn test_detects_many_lifetimes() {
        let code = r#"
            fn complex<'a, 'b, 'c>(x: &'a str, y: &'b str, z: &'c str) -> &'a str {
                x
            }
        "#;

        let syntax = syn::parse_file(code).unwrap();
        let checker = E1203ComplicatedBorrowing::default();

        let mut violations = Vec::new();
        for item in &syntax.items {
            violations.extend(checker.check_item(item, "test.rs").unwrap());
        }

        assert_eq!(violations.len(), 1);
        assert!(violations[0].message.contains("lifetime"));
    }

    #[test]
    fn test_detects_many_mut_refs() {
        let code = r#"
            fn many_muts(a: &mut i32, b: &mut i32, c: &mut i32) {
                *a = *b + *c;
            }
        "#;

        let syntax = syn::parse_file(code).unwrap();
        let checker = E1203ComplicatedBorrowing::default();

        let mut violations = Vec::new();
        for item in &syntax.items {
            violations.extend(checker.check_item(item, "test.rs").unwrap());
        }

        assert_eq!(violations.len(), 1);
        assert!(violations[0].message.contains("mutable reference"));
    }

    #[test]
    fn test_detects_nested_refs() {
        let code = r#"
            fn nested(x: &&i32) -> i32 {
                **x
            }
        "#;

        let syntax = syn::parse_file(code).unwrap();
        let checker = E1203ComplicatedBorrowing::default();

        let mut violations = Vec::new();
        for item in &syntax.items {
            violations.extend(checker.check_item(item, "test.rs").unwrap());
        }

        assert_eq!(violations.len(), 1);
        assert!(violations[0].message.contains("nested"));
    }

    #[test]
    fn test_simple_refs_pass() {
        let code = r#"
            fn simple(x: &str, y: &i32) -> &str {
                x
            }
        "#;

        let syntax = syn::parse_file(code).unwrap();
        let checker = E1203ComplicatedBorrowing::default();

        let mut violations = Vec::new();
        for item in &syntax.items {
            violations.extend(checker.check_item(item, "test.rs").unwrap());
        }

        assert_eq!(violations.len(), 0);
    }
}
