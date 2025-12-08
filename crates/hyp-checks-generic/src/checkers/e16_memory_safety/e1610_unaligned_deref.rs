//! E1610: Unaligned pointer dereference
//!
//! Detects patterns that can cause unaligned memory access:
//! - read_unaligned / write_unaligned usage (flagged as potentially intentional but risky)
//! - Casting between pointer types with different alignments
//! - Using packed structs without proper precautions
//!
//! Unaligned access can cause crashes on some architectures or performance penalties.

use crate::{checker::Checker, define_checker, violation::Violation};
use syn::visit::Visit;

define_checker! {
    /// Checker for E1610: Unaligned dereference detection
    E1610UnalignedDeref,
    code = "E1610",
    name = "Unaligned pointer dereference",
    suggestions = "Use read_unaligned/write_unaligned for intentional unaligned access, or ensure proper alignment before dereferencing.",
    target_items = [Function, Struct],
    config_entry_name = "e1610_unaligned_deref",
    config = E1610Config {
        enabled: bool = true,
        severity: crate::config::SeverityLevel = crate::config::SeverityLevel::High,
        categories: Vec<crate::config::CheckerCategory> = vec![crate::config::CheckerCategory::Operations],
    },
    check_item(self, item, file_path) {
        let mut visitor = UnalignedVisitor {
            violations: Vec::new(),
            file_path,
            checker: self,
        };
        visitor.visit_item(item);

        // Check for #[repr(packed)] structs
        if let syn::Item::Struct(s) = item {
            for attr in &s.attrs {
                if attr.path().is_ident("repr") {
                    let tokens = attr.meta.to_token_stream().to_string();
                    if tokens.contains("packed") {
                        use syn::spanned::Spanned;
                        let start = attr.span().start();
                        visitor.violations.push(
                            Violation::new(
                                self.code(),
                                self.name(),
                                self.severity().into(),
                                "Struct uses #[repr(packed)]. Field access creates unaligned references - use ptr::read_unaligned for safe access.",
                                file_path,
                                start.line,
                                start.column + 1,
                            )
                            .with_suggestion(self.suggestions()),
                        );
                    }
                }
            }
        }

        Ok(visitor.violations)
    }
}

use quote::ToTokens;

struct UnalignedVisitor<'a> {
    violations: Vec<Violation>,
    file_path: &'a str,
    checker: &'a E1610UnalignedDeref,
}

impl<'a> Visit<'a> for UnalignedVisitor<'a> {
    fn visit_expr_call(&mut self, node: &'a syn::ExprCall) {
        if let syn::Expr::Path(path) = &*node.func {
            let path_str = path.path.segments.iter()
                .map(|s| s.ident.to_string())
                .collect::<Vec<_>>()
                .join("::");

            // Flag read_unaligned / write_unaligned as intentional but potentially dangerous
            if path_str.contains("read_unaligned") || path_str.contains("write_unaligned") {
                use syn::spanned::Spanned;
                let start = node.span().start();
                self.violations.push(
                    Violation::new(
                        self.checker.code(),
                        self.checker.name(),
                        self.checker.severity().into(),
                        "Using unaligned memory access. This may be intentional but causes performance penalties on some architectures.",
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

    fn visit_expr_cast(&mut self, node: &'a syn::ExprCast) {
        // Check for pointer casts that might cause alignment issues
        // e.g., casting *const u8 to *const u64
        if let syn::Type::Ptr(ptr_ty) = &*node.ty {
            if let syn::Type::Path(path) = &*ptr_ty.elem {
                let type_name = path.path.segments.iter()
                    .map(|s| s.ident.to_string())
                    .collect::<Vec<_>>()
                    .join("::");

                // Flag casts to types with stricter alignment requirements
                if matches!(type_name.as_str(),
                    "u16" | "u32" | "u64" | "u128" |
                    "i16" | "i32" | "i64" | "i128" |
                    "f32" | "f64" | "usize" | "isize"
                ) {
                    // Check if source might be a byte pointer
                    if let syn::Expr::Cast(inner_cast) = &*node.expr {
                        if let syn::Type::Ptr(inner_ptr) = &*inner_cast.ty {
                            if let syn::Type::Path(inner_path) = &*inner_ptr.elem {
                                let inner_name = inner_path.path.segments.iter()
                                    .map(|s| s.ident.to_string())
                                    .collect::<Vec<_>>()
                                    .join("::");
                                if inner_name == "u8" || inner_name == "i8" {
                                    use syn::spanned::Spanned;
                                    let start = node.span().start();
                                    self.violations.push(
                                        Violation::new(
                                            self.checker.code(),
                                            self.checker.name(),
                                            self.checker.severity().into(),
                                            "Casting byte pointer to type with stricter alignment. This can cause unaligned access.",
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
        }

        syn::visit::visit_expr_cast(self, node);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::checker::Checker;

    #[test]
    fn test_detects_packed_struct() {
        let code = r#"
            #[repr(packed)]
            struct Packed {
                a: u8,
                b: u32,
            }
        "#;

        let syntax = syn::parse_file(code).unwrap();
        let checker = E1610UnalignedDeref::default();

        let mut violations = Vec::new();
        for item in &syntax.items {
            violations.extend(checker.check_item(item, "test.rs").unwrap());
        }

        assert_eq!(violations.len(), 1);
        assert!(violations[0].message.contains("packed"));
    }

    #[test]
    fn test_detects_read_unaligned() {
        let code = r#"
            fn read_unaligned_data(ptr: *const u32) -> u32 {
                unsafe { std::ptr::read_unaligned(ptr) }
            }
        "#;

        let syntax = syn::parse_file(code).unwrap();
        let checker = E1610UnalignedDeref::default();

        let mut violations = Vec::new();
        for item in &syntax.items {
            violations.extend(checker.check_item(item, "test.rs").unwrap());
        }

        assert_eq!(violations.len(), 1);
        assert!(violations[0].message.contains("unaligned"));
    }

    #[test]
    fn test_normal_struct_passes() {
        let code = r#"
            #[repr(C)]
            struct Normal {
                a: u8,
                b: u32,
            }
        "#;

        let syntax = syn::parse_file(code).unwrap();
        let checker = E1610UnalignedDeref::default();

        let mut violations = Vec::new();
        for item in &syntax.items {
            violations.extend(checker.check_item(item, "test.rs").unwrap());
        }

        assert_eq!(violations.len(), 0);
    }
}
