//! E1407: Lossy float to int conversion
//!
//! Detects conversions from floating-point to integer types that can
//! lose precision or produce unexpected results for special values.

use crate::{checker::Checker, define_checker, violation::Violation};

use syn::{spanned::Spanned, visit::Visit};

define_checker! {
    /// Checker for E1407: Lossy float to int conversion
    E1407LossyFloatConversion,
    code = "E1407",
    name = "Lossy float to int conversion",
    suggestions = "Use f64::round(), f64::floor(), or f64::ceil() before casting, and handle NaN/infinity",
    target_items = [Function],
    config_entry_name = "e1407_lossy_float_conversion",
    /// Configuration for E1407: Lossy float conversion checker
    config = E1407Config {
        /// Whether this checker is enabled
        enabled: bool = true,
        /// Severity level, defaults to Medium
        severity: crate::config::SeverityLevel = crate::config::SeverityLevel::Medium,
        /// Categories this checker belongs to
        categories: Vec<crate::config::CheckerCategory> = vec![crate::config::CheckerCategory::Operations],
    },
    check_item(self, item, file_path) {
        let mut visitor = FloatConversionVisitor {
            violations: Vec::new(),
            file_path,
            checker: self,
        };
        visitor.visit_item(item);
        Ok(visitor.violations)
    }
}

struct FloatConversionVisitor<'a> {
    violations: Vec<Violation>,
    file_path: &'a str,
    checker: &'a E1407LossyFloatConversion,
}

impl<'a> FloatConversionVisitor<'a> {
    fn create_violation(&self, span: proc_macro2::Span, to_type: &str) -> Violation {
        let start = span.start();
        Violation::new(
            self.checker.code(),
            self.checker.name(),
            self.checker.severity().into(),
            format!(
                "Converting float to {} truncates the fractional part and may produce unexpected results for NaN/infinity.",
                to_type
            ),
            self.file_path,
            start.line,
            start.column + 1,
        )
        .with_suggestion(self.checker.suggestions())
    }
}

impl<'a> Visit<'a> for FloatConversionVisitor<'a> {
    fn visit_expr_cast(&mut self, node: &'a syn::ExprCast) {
        let to_type = type_to_string(&node.ty);

        // Check if casting to an integer type (potential float source)
        if is_integer_type(&to_type) {
            // Check if the expression being cast looks like a float
            if looks_like_float(&node.expr) {
                self.violations.push(self.create_violation(node.span(), &to_type));
            }
        }

        syn::visit::visit_expr_cast(self, node);
    }
}

fn type_to_string(ty: &syn::Type) -> String {
    match ty {
        syn::Type::Path(type_path) => {
            type_path.path.segments.iter()
                .map(|s| s.ident.to_string())
                .collect::<Vec<_>>()
                .join("::")
        }
        _ => "unknown".to_string()
    }
}

fn is_integer_type(ty: &str) -> bool {
    matches!(ty, "i8" | "i16" | "i32" | "i64" | "i128" | "isize" |
                 "u8" | "u16" | "u32" | "u64" | "u128" | "usize")
}

/// Heuristic to detect if an expression is likely a float
fn looks_like_float(expr: &syn::Expr) -> bool {
    match expr {
        syn::Expr::Lit(lit) => matches!(lit.lit, syn::Lit::Float(_)),
        syn::Expr::MethodCall(call) => {
            // Methods like .sin(), .sqrt() return floats
            let method = call.method.to_string();
            matches!(method.as_str(),
                "sin" | "cos" | "tan" | "sqrt" | "abs" | "floor" | "ceil" | "round" |
                "exp" | "ln" | "log" | "log10" | "powf" | "powi")
        }
        syn::Expr::Binary(bin) => {
            // If either operand is a float, result is float
            looks_like_float(&bin.left) || looks_like_float(&bin.right)
        }
        syn::Expr::Paren(paren) => looks_like_float(&paren.expr),
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detects_float_literal_cast() {
        let code = r#"
            fn example() -> i32 {
                3.14 as i32
            }
        "#;

        let syntax = syn::parse_file(code).unwrap();
        let checker = E1407LossyFloatConversion::default();

        let mut violations = Vec::new();
        for item in &syntax.items {
            violations.extend(checker.check_item(item, "test.rs").unwrap());
        }

        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].code, "E1407");
    }

    #[test]
    fn test_integer_cast_passes() {
        let code = r#"
            fn example() -> i64 {
                42i32 as i64
            }
        "#;

        let syntax = syn::parse_file(code).unwrap();
        let checker = E1407LossyFloatConversion::default();

        let mut violations = Vec::new();
        for item in &syntax.items {
            violations.extend(checker.check_item(item, "test.rs").unwrap());
        }

        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_detects_float_expression() {
        let code = r#"
            fn example() -> i32 {
                (1.5 + 2.5) as i32
            }
        "#;

        let syntax = syn::parse_file(code).unwrap();
        let checker = E1407LossyFloatConversion::default();

        let mut violations = Vec::new();
        for item in &syntax.items {
            violations.extend(checker.check_item(item, "test.rs").unwrap());
        }

        assert_eq!(violations.len(), 1);
    }
}
