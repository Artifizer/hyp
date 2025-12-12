//! E1611: Methods consuming `self` when `&self` or `&mut self` would suffice
//!
//! Detects methods that take `self` by value (consuming ownership) when they
//! don't actually need to. This forces callers to give up ownership unnecessarily.
//!
//! Example:
//! ```text
//! // Bad: Consumes self unnecessarily
//! impl Data {
//!     fn get_value(self) -> i32 { self.value }  // Takes ownership!
//! }
//!
//! // Good: Borrows instead
//! impl Data {
//!     fn get_value(&self) -> i32 { self.value }  // Borrows only
//! }
//! ```

use crate::{checker::Checker, define_checker, violation::Violation};

define_checker! {
    /// Checker for E1611: Consuming self unnecessarily
    E1611ConsumingSelf,
    code = "E1611",
    name = "Method consumes self unnecessarily",
    suggestions = "Use &self for read-only access, &mut self for mutation. Reserve self for builders or when returning Self.",
    target_items = [Impl],
    config_entry_name = "e1611_consuming_self",
    config = E1611Config {
        enabled: bool = true,
        severity: crate::config::SeverityLevel = crate::config::SeverityLevel::Low,
        categories: Vec<crate::config::CheckerCategory> = vec![crate::config::CheckerCategory::Complexity],
        /// Allow self-consuming methods that return Self (builder pattern)
        allow_builder_pattern: bool = true,
    },
    check_item(self, item, file_path) {
        let mut violations = Vec::new();

        if let syn::Item::Impl(impl_block) = item {
            for item in &impl_block.items {
                if let syn::ImplItem::Fn(method) = item {
                    if let Some(violation) = check_method(self, method, file_path) {
                        violations.push(violation);
                    }
                }
            }
        }

        Ok(violations)
    }
}

fn check_method(
    checker: &E1611ConsumingSelf,
    method: &syn::ImplItemFn,
    file_path: &str,
) -> Option<Violation> {
    // Check if first argument is `self` (not &self or &mut self)
    let takes_self_by_value = method.sig.inputs.first().is_some_and(|arg| {
        if let syn::FnArg::Receiver(receiver) = arg {
            receiver.reference.is_none() // No & means by value
        } else {
            false
        }
    });

    if !takes_self_by_value {
        return None;
    }

    // Check if method returns Self (builder pattern exception)
    if checker.config.allow_builder_pattern {
        if let syn::ReturnType::Type(_, ty) = &method.sig.output {
            if is_self_type(ty) {
                return None; // Builder pattern is allowed
            }
        }
    }

    // Check method name - some patterns are expected to consume
    let method_name = method.sig.ident.to_string();
    let consuming_names = [
        "into_",
        "to_owned",
        "build",
        "finish",
        "unwrap",
        "take",
        "consume",
    ];
    if consuming_names.iter().any(|prefix| method_name.starts_with(prefix)) {
        return None;
    }

    // Getters that consume are suspicious
    let getter_prefixes = ["get", "is_", "has_", "len", "count", "size", "as_"];
    let is_getter = getter_prefixes.iter().any(|prefix| method_name.starts_with(prefix))
        || (method_name.len() < 15 && !method_name.contains('_'));

    if is_getter {
        let span = method.sig.ident.span();
        return Some(
            Violation::new(
                checker.code(),
                checker.name(),
                checker.severity().into(),
                format!(
                    "Method '{}' takes self by value but appears to be a getter. Use &self instead.",
                    method_name
                ),
                file_path,
                span.start().line,
                span.start().column + 1,
            )
            .with_suggestion(checker.suggestions()),
        );
    }

    None
}

fn is_self_type(ty: &syn::Type) -> bool {
    match ty {
        syn::Type::Path(path) => {
            path.path.segments.last().is_some_and(|seg| seg.ident == "Self")
        }
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::checker::Checker;

    fn check_code(code: &str) -> Vec<Violation> {
        let checker = E1611ConsumingSelf::default();
        let file = syn::parse_file(code).expect("Failed to parse");
        let mut violations = Vec::new();
        for item in &file.items {
            violations.extend(checker.check_item(item, "test.rs").unwrap());
        }
        violations
    }

    #[test]
    fn test_detects_getter_consuming_self() {
        let code = r#"
            struct Data { value: i32 }
            impl Data {
                fn get_value(self) -> i32 { self.value }
            }
        "#;
        let violations = check_code(code);
        assert_eq!(violations.len(), 1);
    }

    #[test]
    fn test_allows_builder_pattern() {
        let code = r#"
            struct Builder { value: i32 }
            impl Builder {
                fn with_value(self, v: i32) -> Self {
                    Self { value: v }
                }
            }
        "#;
        let violations = check_code(code);
        assert!(violations.is_empty());
    }

    #[test]
    fn test_allows_into_methods() {
        let code = r#"
            struct Data { value: String }
            impl Data {
                fn into_inner(self) -> String { self.value }
            }
        "#;
        let violations = check_code(code);
        assert!(violations.is_empty());
    }

    #[test]
    fn test_reference_self_passes() {
        let code = r#"
            struct Data { value: i32 }
            impl Data {
                fn get_value(&self) -> i32 { self.value }
            }
        "#;
        let violations = check_code(code);
        assert!(violations.is_empty());
    }
}
