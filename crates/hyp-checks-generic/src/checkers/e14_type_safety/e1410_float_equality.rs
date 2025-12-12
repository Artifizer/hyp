//! E1410: Float equality comparison with `==`
//!
//! Detects direct equality comparisons between floating-point numbers using `==` or `!=`.
//! Due to floating-point precision issues, this almost always leads to bugs.
//!
//! Example:
//! ```text
//! // Bad: May fail due to precision
//! if 0.1 + 0.2 == 0.3 { ... }  // This is FALSE!
//!
//! // Good: Use epsilon comparison
//! if (a - b).abs() < f64::EPSILON { ... }
//! ```

use crate::{checker::Checker, define_checker, violation::Violation};

use syn::{spanned::Spanned, visit::Visit};

define_checker! {
    /// Checker for E1410: Float equality comparison
    E1410FloatEquality,
    code = "E1410",
    name = "Float equality comparison with ==",
    suggestions = "Use epsilon comparison: (a - b).abs() < f64::EPSILON, or use the `approx` crate",
    target_items = [Function],
    config_entry_name = "e1410_float_equality",
    config = E1410Config {
        enabled: bool = true,
        severity: crate::config::SeverityLevel = crate::config::SeverityLevel::High,
        categories: Vec<crate::config::CheckerCategory> = vec![crate::config::CheckerCategory::Operations],
    },
    check_item(self, item, file_path) {
        let mut visitor = FloatEqualityVisitor {
            violations: Vec::new(),
            file_path,
            checker: self,
        };
        visitor.visit_item(item);
        Ok(visitor.violations)
    }
}

struct FloatEqualityVisitor<'a> {
    violations: Vec<Violation>,
    file_path: &'a str,
    checker: &'a E1410FloatEquality,
}

impl<'a> FloatEqualityVisitor<'a> {
    fn is_float_literal(expr: &syn::Expr) -> bool {
        if let syn::Expr::Lit(lit) = expr {
            matches!(lit.lit, syn::Lit::Float(_))
        } else {
            false
        }
    }

    fn is_float_cast(expr: &syn::Expr) -> bool {
        if let syn::Expr::Cast(cast) = expr {
            if let syn::Type::Path(path) = &*cast.ty {
                let type_name = path
                    .path
                    .segments
                    .last()
                    .map(|s| s.ident.to_string())
                    .unwrap_or_default();
                return type_name == "f32" || type_name == "f64";
            }
        }
        false
    }

    fn looks_like_float(expr: &syn::Expr) -> bool {
        Self::is_float_literal(expr) || Self::is_float_cast(expr)
    }
}

impl<'a> Visit<'a> for FloatEqualityVisitor<'a> {
    fn visit_expr(&mut self, node: &'a syn::Expr) {
        if let syn::Expr::Binary(binary) = node {
            // Check for == or != operators
            let is_equality = matches!(
                binary.op,
                syn::BinOp::Eq(_) | syn::BinOp::Ne(_)
            );

            if is_equality {
                // Check if either side looks like a float
                let left_float = Self::looks_like_float(&binary.left);
                let right_float = Self::looks_like_float(&binary.right);

                if left_float || right_float {
                    let span = binary.op.span();
                    self.violations.push(
                        Violation::new(
                            self.checker.code(),
                            self.checker.name(),
                            self.checker.severity().into(),
                            "Comparing floats with == or != is unreliable due to precision. Use epsilon comparison instead.",
                            self.file_path,
                            span.start().line,
                            span.start().column + 1,
                        )
                        .with_suggestion(self.checker.suggestions()),
                    );
                }
            }
        }

        syn::visit::visit_expr(self, node);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::checker::Checker;

    fn check_code(code: &str) -> Vec<Violation> {
        let checker = E1410FloatEquality::default();
        let file = syn::parse_file(code).expect("Failed to parse");
        let mut violations = Vec::new();
        for item in &file.items {
            violations.extend(checker.check_item(item, "test.rs").unwrap());
        }
        violations
    }

    #[test]
    fn test_detects_float_literal_equality() {
        let code = r#"
            fn example() -> bool {
                0.1 + 0.2 == 0.3
            }
        "#;
        let violations = check_code(code);
        assert_eq!(violations.len(), 1);
    }

    #[test]
    fn test_detects_float_inequality() {
        let code = r#"
            fn example() -> bool {
                1.5 != 2.5
            }
        "#;
        let violations = check_code(code);
        assert_eq!(violations.len(), 1);
    }

    #[test]
    fn test_integer_equality_passes() {
        let code = r#"
            fn example() -> bool {
                1 == 2
            }
        "#;
        let violations = check_code(code);
        assert!(violations.is_empty());
    }

    #[test]
    fn test_float_cast_detected() {
        let code = r#"
            fn example(x: i32) -> bool {
                x as f64 == 1.0
            }
        "#;
        let violations = check_code(code);
        assert_eq!(violations.len(), 1);
    }
}
