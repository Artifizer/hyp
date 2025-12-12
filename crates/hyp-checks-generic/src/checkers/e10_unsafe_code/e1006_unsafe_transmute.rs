//! E1006: Unsafe transmute usage
//!
//! Detects use of std::mem::transmute which can cause undefined behavior
//! by reinterpreting memory as a different type.

use crate::{checker::Checker, define_checker, violation::Violation};

use syn::visit::Visit;

define_checker! {
    /// Checker for E1006: Unsafe transmute
    E1006UnsafeTransmute,
    code = "E1006",
    name = "Unsafe transmute",
    suggestions = "Use safe alternatives like From/Into traits, TryFrom/TryInto, or type-specific conversion methods like to_bits()/from_bits()",
    target_items = [Function, Impl],
    config_entry_name = "e1006_unsafe_transmute",
    config = E1006Config {
        enabled: bool = true,
        severity: crate::config::SeverityLevel = crate::config::SeverityLevel::High,
        categories: Vec<crate::config::CheckerCategory> = vec![crate::config::CheckerCategory::Operations],
    },
    check_item(self, item, file_path) {
        let mut visitor = TransmuteVisitor {
            violations: Vec::new(),
            file_path,
            checker: self,
        };
        visitor.visit_item(item);
        Ok(visitor.violations)
    }
}

struct TransmuteVisitor<'a> {
    violations: Vec<Violation>,
    file_path: &'a str,
    checker: &'a E1006UnsafeTransmute,
}

impl<'a> Visit<'a> for TransmuteVisitor<'a> {
    fn visit_expr_call(&mut self, node: &'a syn::ExprCall) {
        use syn::spanned::Spanned;

        if let syn::Expr::Path(path) = &*node.func {
            let path_str = path
                .path
                .segments
                .iter()
                .map(|s| s.ident.to_string())
                .collect::<Vec<_>>()
                .join("::");

            if path_str.contains("transmute") {
                let start = node.span().start();
                self.violations.push(
                    Violation::new(
                        self.checker.code(),
                        self.checker.name(),
                        self.checker.severity().into(),
                        format!(
                            "Using {}. Transmute reinterprets memory as a different type which can cause undefined behavior.",
                            path_str
                        ),
                        self.file_path,
                        start.line,
                        start.column + 1,
                    )
                    .with_suggestion(self.checker.suggestions()),
                );
            }
        }

        syn::visit::visit_expr_call(self, node);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detects_transmute() {
        let code = r#"
            fn bad() {
                let x: u32 = 42;
                unsafe {
                    let y: f32 = std::mem::transmute(x);
                }
            }
        "#;

        let syntax = syn::parse_file(code).unwrap();
        let checker = E1006UnsafeTransmute::default();

        let mut violations = Vec::new();
        for item in &syntax.items {
            violations.extend(checker.check_item(item, "test.rs").unwrap());
        }

        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].code, "E1006");
    }

    #[test]
    fn test_safe_conversion_passes() {
        let code = r#"
            fn good() {
                let x: u32 = 42;
                let y = f32::from_bits(x);
            }
        "#;

        let syntax = syn::parse_file(code).unwrap();
        let checker = E1006UnsafeTransmute::default();

        let mut violations = Vec::new();
        for item in &syntax.items {
            violations.extend(checker.check_item(item, "test.rs").unwrap());
        }

        assert_eq!(violations.len(), 0);
    }
}
