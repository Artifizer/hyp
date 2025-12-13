//! E1612: Prohibit custom allocators
//!
//! Custom allocators (`#[global_allocator]`) add complexity that microservices
//! rarely need. They are appropriate for HPC, low-latency trading, embedded,
//! and real-time systems, but not for typical I/O-bound microservices.
//!
//! Example:
//! ```text
//! // Bad: Custom allocator in a microservice
//! use tikv_jemallocator::Jemalloc;
//!
//! #[global_allocator]
//! static GLOBAL: Jemalloc = Jemalloc;
//!
//! // Good: Use the default allocator (no code needed)
//! // The standard allocator is well-tested and compatible with debugging tools
//! ```

use crate::{define_checker, violation::Violation};

use syn::spanned::Spanned;

define_checker! {
    /// Checker for E1612: Prohibit custom allocators
    E1612ProhibitCustomAllocators,
    code = "E1612",
    name = "Custom allocator is prohibited",
    suggestions = "Microservices should use the default allocator. Custom allocators add complexity without measurable benefit for I/O-bound services. If this is intentional (HPC, low-latency trading), disable this checker.",
    target_items = [Static],
    config_entry_name = "e1612_prohibit_custom_allocators",
    config = E1612Config {
        enabled: bool = true,
        severity: crate::config::SeverityLevel = crate::config::SeverityLevel::Medium,
        categories: Vec<crate::config::CheckerCategory> = vec![crate::config::CheckerCategory::Operations],
    },
    check_item(self, item, file_path) {
        let mut violations = Vec::new();

        if let syn::Item::Static(static_item) = item {
            // Check for #[global_allocator] attribute
            let has_global_allocator = static_item.attrs.iter().any(|attr| {
                if let syn::Meta::Path(path) = &attr.meta {
                    path.segments.last()
                        .map(|seg| seg.ident == "global_allocator")
                        .unwrap_or(false)
                } else {
                    false
                }
            });

            if has_global_allocator {
                let span = static_item.span();
                let type_name = Self::extract_type_name(&static_item.ty);

                let description = if let Some(ref name) = type_name {
                    format!(
                        "#[global_allocator] with {} detected. Custom allocators add complexity without benefit for typical microservices.",
                        name
                    )
                } else {
                    "#[global_allocator] detected. Custom allocators add complexity without benefit for typical microservices.".to_string()
                };

                violations.push(
                    Violation::new(
                        self.code(),
                        self.name(),
                        self.severity().into(),
                        &description,
                        file_path,
                        span.start().line,
                        span.start().column + 1,
                    )
                    .with_suggestion(self.suggestions()),
                );
            }
        }

        Ok(violations)
    }
}

impl E1612ProhibitCustomAllocators {
    /// Extract type name from a type expression for better error messages
    fn extract_type_name(ty: &syn::Type) -> Option<String> {
        match ty {
            syn::Type::Path(type_path) => {
                let segments: Vec<String> = type_path
                    .path
                    .segments
                    .iter()
                    .map(|s| s.ident.to_string())
                    .collect();
                Some(segments.join("::"))
            }
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::checker::Checker;

    fn check_code(code: &str) -> Vec<Violation> {
        let checker = E1612ProhibitCustomAllocators::default();
        let file = syn::parse_file(code).expect("Failed to parse");
        let mut violations = Vec::new();
        for item in &file.items {
            violations.extend(checker.check_item(item, "test.rs").unwrap());
        }
        violations
    }

    #[test]
    fn test_detects_global_allocator_jemalloc() {
        let code = r#"
            use tikv_jemallocator::Jemalloc;

            #[global_allocator]
            static GLOBAL: Jemalloc = Jemalloc;
        "#;
        let violations = check_code(code);
        assert_eq!(violations.len(), 1);
        assert!(violations[0].code == "E1612");
        assert!(violations[0].message.contains("Jemalloc"));
    }

    #[test]
    fn test_detects_global_allocator_mimalloc() {
        let code = r#"
            use mimalloc::MiMalloc;

            #[global_allocator]
            static GLOBAL: MiMalloc = MiMalloc;
        "#;
        let violations = check_code(code);
        assert_eq!(violations.len(), 1);
        assert!(violations[0].message.contains("MiMalloc"));
    }

    #[test]
    fn test_no_global_allocator_passes() {
        let code = r#"
            static SOME_VALUE: u32 = 42;

            fn main() {
                println!("No custom allocator");
            }
        "#;
        let violations = check_code(code);
        assert!(violations.is_empty());
    }

    #[test]
    fn test_static_without_global_allocator_passes() {
        let code = r#"
            use std::sync::atomic::AtomicUsize;

            static COUNTER: AtomicUsize = AtomicUsize::new(0);
        "#;
        let violations = check_code(code);
        assert!(violations.is_empty());
    }

    #[test]
    fn test_detects_system_allocator_override() {
        let code = r#"
            use std::alloc::System;

            #[global_allocator]
            static GLOBAL: System = System;
        "#;
        let violations = check_code(code);
        assert_eq!(violations.len(), 1);
        // Even System override is flagged - if you want default, just don't set it
    }
}
