//! E1209: Higher-ranked trait bounds (HRTB)
//!
//! Detects `for<'a>` higher-ranked trait bound syntax which can be
//! confusing and complex.
//!
//! Example:
//! ```text
//! // HRTB can be hard to understand
//! fn apply<F>(f: F) where F: for<'a> Fn(&'a str) -> &'a str { ... }
//! ```

use crate::{checker::Checker, define_checker, violation::Violation};
use syn::visit::Visit;

define_checker! {
    /// Checker for E1209: Higher-ranked trait bounds (HRTB)
    E1209Hrtb,
    code = "E1209",
    name = "Higher-ranked trait bounds (HRTB)",
    suggestions = "Consider if HRTB is necessary. Sometimes lifetime elision or explicit lifetimes work better. Document the HRTB's purpose.",
    target_items = [Function, Impl, Trait],
    config_entry_name = "e1209_hrtb",
    config = E1209Config {
        enabled: bool = true,
        severity: crate::config::SeverityLevel = crate::config::SeverityLevel::Low,
        categories: Vec<crate::config::CheckerCategory> = vec![crate::config::CheckerCategory::Complexity],
        /// Warn on first HRTB (informational)
        warn_on_single: bool = true,
        /// Maximum HRTBs before higher severity warning
        max_hrtb_count: usize = 2,
    },
    check_item(self, item, file_path) {
        let mut visitor = HrtbVisitor {
            violations: Vec::new(),
            file_path,
            checker: self,
            hrtb_count: 0,
            hrtb_locations: Vec::new(),
        };
        visitor.visit_item(item);

        // Report based on count
        if visitor.hrtb_count > self.config.max_hrtb_count {
            if let Some(first_span) = visitor.hrtb_locations.first() {
                let start = first_span.start();
                visitor.violations.push(
                    Violation::new(
                        self.code(),
                        self.name(),
                        crate::violation::CheckerSeverity::Medium.into(),
                        format!(
                            "Found {} higher-ranked trait bounds. Multiple HRTBs significantly increase complexity.",
                            visitor.hrtb_count
                        ),
                        file_path,
                        start.line,
                        start.column + 1,
                    )
                    .with_suggestion(self.suggestions()),
                );
            }
        }

        Ok(visitor.violations)
    }
}

struct HrtbVisitor<'a> {
    violations: Vec<Violation>,
    file_path: &'a str,
    checker: &'a E1209Hrtb,
    hrtb_count: usize,
    hrtb_locations: Vec<proc_macro2::Span>,
}

impl<'a> Visit<'a> for HrtbVisitor<'a> {
    fn visit_bound_lifetimes(&mut self, node: &'a syn::BoundLifetimes) {
        use syn::spanned::Spanned;

        self.hrtb_count += 1;
        self.hrtb_locations.push(node.span());

        if self.checker.config.warn_on_single && self.hrtb_count == 1 {
            let start = node.span().start();
            self.violations.push(
                Violation::new(
                    self.checker.code(),
                    self.checker.name(),
                    self.checker.severity().into(),
                    "Higher-ranked trait bound (for<'...>) detected. HRTBs add complexity; ensure this is necessary.",
                    self.file_path,
                    start.line,
                    start.column + 1,
                )
                .with_suggestion(self.checker.suggestions()),
            );
        }

        syn::visit::visit_bound_lifetimes(self, node);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::checker::Checker;

    #[test]
    fn test_detects_hrtb_in_where_clause() {
        let code = r#"
            fn apply<F>(f: F)
            where
                F: for<'a> Fn(&'a str) -> &'a str
            {
            }
        "#;

        let syntax = syn::parse_file(code).unwrap();
        let checker = E1209Hrtb::default();

        let mut violations = Vec::new();
        for item in &syntax.items {
            violations.extend(checker.check_item(item, "test.rs").unwrap());
        }

        assert_eq!(violations.len(), 1);
        assert!(violations[0].message.contains("Higher-ranked"));
    }

    #[test]
    fn test_detects_multiple_hrtbs() {
        let code = r#"
            fn complex<F, G>(f: F, g: G)
            where
                F: for<'a> Fn(&'a str) -> &'a str,
                G: for<'b> Fn(&'b i32) -> &'b i32,
                F: for<'c> Clone,
            {
            }
        "#;

        let syntax = syn::parse_file(code).unwrap();
        let checker = E1209Hrtb::default();

        let mut violations = Vec::new();
        for item in &syntax.items {
            violations.extend(checker.check_item(item, "test.rs").unwrap());
        }

        // Should have violation for multiple HRTBs
        assert!(violations.iter().any(|v| v.message.contains("3 higher-ranked")));
    }

    #[test]
    fn test_no_hrtb_passes() {
        let code = r#"
            fn simple<F>(f: F)
            where
                F: Fn(&str) -> String
            {
            }
        "#;

        let syntax = syn::parse_file(code).unwrap();
        let checker = E1209Hrtb::default();

        let mut violations = Vec::new();
        for item in &syntax.items {
            violations.extend(checker.check_item(item, "test.rs").unwrap());
        }

        assert!(violations.is_empty());
    }

    #[test]
    fn test_hrtb_in_trait_bound() {
        let code = r#"
            trait Handler: for<'a> Fn(&'a Request) -> Response {}
        "#;

        let syntax = syn::parse_file(code).unwrap();
        let checker = E1209Hrtb::default();

        let mut violations = Vec::new();
        for item in &syntax.items {
            violations.extend(checker.check_item(item, "test.rs").unwrap());
        }

        assert!(!violations.is_empty());
    }
}
