//! E1004: Unsafe without comments
//!
//! Detects `unsafe` blocks that lack a `// SAFETY:` comment explaining why the unsafe
//! code is sound. Every unsafe block should document its safety invariants.

use crate::{checker::Checker, define_checker, violation::Violation};

use syn::visit::Visit;

define_checker! {
    /// Checker for E1004: Unsafe without comments
    E1004UnsafeWithoutComment,
    code = "E1004",
    name = "Unsafe block without SAFETY comment",
    suggestions = "Add a `// SAFETY:` comment explaining why this unsafe code is sound",
    target_items = [Function],
    config_entry_name = "e1004_unsafe_without_comment",
    /// Configuration for E1004: Unsafe without comment checker
    config = E1004Config {
        /// Whether this checker is enabled
        enabled: bool = true,
        /// Severity level, defaults to High
        severity: crate::config::SeverityLevel = crate::config::SeverityLevel::High,
        /// Categories this checker belongs to
        categories: Vec<crate::config::CheckerCategory> = vec![crate::config::CheckerCategory::Operations],
    },
    check_item(self, item, file_path) {
        let mut visitor = UnsafeCommentVisitor {
            violations: Vec::new(),
            file_path,
            checker: self,
        };
        visitor.visit_item(item);
        Ok(visitor.violations)
    }
}

struct UnsafeCommentVisitor<'a> {
    violations: Vec<Violation>,
    file_path: &'a str,
    checker: &'a E1004UnsafeWithoutComment,
}

impl<'a> UnsafeCommentVisitor<'a> {
    fn create_violation(&self, span: proc_macro2::Span) -> Violation {
        let start = span.start();
        Violation::new(
            self.checker.code(),
            self.checker.name(),
            self.checker.severity().into(),
            "Unsafe block without a SAFETY comment. Every unsafe block should document why it is sound.",
            self.file_path,
            start.line,
            start.column + 1,
        )
        .with_suggestion(self.checker.suggestions())
    }
}

impl<'a> Visit<'a> for UnsafeCommentVisitor<'a> {
    fn visit_expr_unsafe(&mut self, node: &'a syn::ExprUnsafe) {
        // Check if there's a SAFETY comment in the unsafe block's attributes
        // Note: In practice, SAFETY comments are usually line comments before the block,
        // which syn doesn't capture. This is a simplified check that flags all unsafe blocks.
        // A more sophisticated implementation would need to parse the source file for comments.

        // For now, we flag all unsafe blocks as they should be reviewed
        // Real implementation would need access to source text to check for preceding comments
        let has_safety_comment = node.attrs.iter().any(|attr| {
            if let syn::Meta::NameValue(nv) = &attr.meta {
                if nv.path.is_ident("doc") {
                    if let syn::Expr::Lit(syn::ExprLit {
                        lit: syn::Lit::Str(s),
                        ..
                    }) = &nv.value
                    {
                        return s.value().to_uppercase().contains("SAFETY");
                    }
                }
            }
            false
        });

        if !has_safety_comment {
            self.violations
                .push(self.create_violation(node.unsafe_token.span));
        }

        syn::visit::visit_expr_unsafe(self, node);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detects_unsafe_without_comment() {
        let code = r#"
            fn example() {
                unsafe {
                    let x = 42;
                }
            }
        "#;

        let syntax = syn::parse_file(code).unwrap();
        let checker = E1004UnsafeWithoutComment::default();

        let mut violations = Vec::new();
        for item in &syntax.items {
            violations.extend(checker.check_item(item, "test.rs").unwrap());
        }

        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].code, "E1004");
    }

    #[test]
    fn test_safe_code_passes() {
        let code = r#"
            fn example() {
                let x = 42;
            }
        "#;

        let syntax = syn::parse_file(code).unwrap();
        let checker = E1004UnsafeWithoutComment::default();

        let mut violations = Vec::new();
        for item in &syntax.items {
            violations.extend(checker.check_item(item, "test.rs").unwrap());
        }

        assert_eq!(violations.len(), 0);
    }
}
