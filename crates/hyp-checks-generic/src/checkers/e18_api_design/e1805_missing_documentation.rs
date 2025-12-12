//! E1805: Missing documentation
//!
//! Detects public items without documentation comments.

use crate::{define_checker, violation::Violation};

define_checker! {
    /// Checker for E1805: Missing documentation
    E1805MissingDocumentation,
    code = "E1805",
    name = "Missing documentation",
    suggestions = "Add documentation comments (///) to public items",
    target_items = [Function, Struct, Enum, Trait],
    config_entry_name = "e1805_missing_documentation",
    config = E1805Config {
        enabled: bool = true,
        severity: crate::config::SeverityLevel = crate::config::SeverityLevel::Low,
        categories: Vec<crate::config::CheckerCategory> = vec![crate::config::CheckerCategory::Complexity],
    },
    check_item(self, item, file_path) {
        let mut violations = Vec::new();

        let (is_public, has_docs, name, span) = match item {
            syn::Item::Fn(f) => (
                matches!(f.vis, syn::Visibility::Public(_)),
                has_doc_comments(&f.attrs),
                f.sig.ident.to_string(),
                f.sig.ident.span(),
            ),
            syn::Item::Struct(s) => (
                matches!(s.vis, syn::Visibility::Public(_)),
                has_doc_comments(&s.attrs),
                s.ident.to_string(),
                s.ident.span(),
            ),
            syn::Item::Enum(e) => (
                matches!(e.vis, syn::Visibility::Public(_)),
                has_doc_comments(&e.attrs),
                e.ident.to_string(),
                e.ident.span(),
            ),
            syn::Item::Trait(t) => (
                matches!(t.vis, syn::Visibility::Public(_)),
                has_doc_comments(&t.attrs),
                t.ident.to_string(),
                t.ident.span(),
            ),
            _ => return Ok(violations),
        };

        if is_public && !has_docs {
            let start = span.start();
            violations.push(
                Violation::new(
                    self.code(),
                    self.name(),
                    self.severity().into(),
                    format!("Public item '{}' is missing documentation.", name),
                    file_path,
                    start.line,
                    start.column + 1,
                )
                .with_suggestion(self.suggestions()),
            );
        }

        Ok(violations)
    }
}

fn has_doc_comments(attrs: &[syn::Attribute]) -> bool {
    attrs.iter().any(|attr| attr.path().is_ident("doc"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::checker::Checker;

    #[test]
    fn test_detects_missing_docs() {
        let code = r#"
            pub fn example() {}
        "#;

        let syntax = syn::parse_file(code).unwrap();
        let checker = E1805MissingDocumentation::default();

        let mut violations = Vec::new();
        for item in &syntax.items {
            violations.extend(checker.check_item(item, "test.rs").unwrap());
        }

        assert_eq!(violations.len(), 1);
    }

    #[test]
    fn test_documented_passes() {
        let code = r#"
            /// This function does something
            pub fn example() {}
        "#;

        let syntax = syn::parse_file(code).unwrap();
        let checker = E1805MissingDocumentation::default();

        let mut violations = Vec::new();
        for item in &syntax.items {
            violations.extend(checker.check_item(item, "test.rs").unwrap());
        }

        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_private_without_docs_passes() {
        let code = r#"
            fn example() {}
        "#;

        let syntax = syn::parse_file(code).unwrap();
        let checker = E1805MissingDocumentation::default();

        let mut violations = Vec::new();
        for item in &syntax.items {
            violations.extend(checker.check_item(item, "test.rs").unwrap());
        }

        assert_eq!(violations.len(), 0);
    }
}
