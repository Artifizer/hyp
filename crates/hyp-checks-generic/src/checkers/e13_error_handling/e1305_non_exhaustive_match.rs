//! E1305: Non-exhaustive match on Result/Option
//!
//! Detects match expressions on Result or Option that use catch-all patterns
//! instead of explicitly handling all variants.

use crate::{checker::Checker, define_checker, violation::Violation};

use syn::visit::Visit;

define_checker! {
    /// Checker for E1305: Non-exhaustive match on Result/Option
    E1305NonExhaustiveMatch,
    code = "E1305",
    name = "Non-exhaustive match on Result/Option",
    suggestions = "Explicitly match Ok/Err or Some/None variants instead of using _ wildcard",
    target_items = [Function],
    config_entry_name = "e1305_non_exhaustive_match",
    config = E1305Config {
        enabled: bool = true,
        severity: crate::config::SeverityLevel = crate::config::SeverityLevel::High,
        categories: Vec<crate::config::CheckerCategory> = vec![crate::config::CheckerCategory::Operations],
    },
    check_item(self, item, file_path) {
        let mut visitor = MatchVisitor {
            violations: Vec::new(),
            file_path,
            checker: self,
        };
        visitor.visit_item(item);
        Ok(visitor.violations)
    }
}

struct MatchVisitor<'a> {
    violations: Vec<Violation>,
    file_path: &'a str,
    checker: &'a E1305NonExhaustiveMatch,
}

impl<'a> Visit<'a> for MatchVisitor<'a> {
    fn visit_expr_match(&mut self, node: &'a syn::ExprMatch) {
        // Check if the match expression looks like it's on Result/Option
        let has_wildcard = node.arms.iter().any(|arm| is_wildcard_pattern(&arm.pat));
        let has_result_option_arm = node.arms.iter().any(|arm| {
            is_result_or_option_pattern(&arm.pat)
        });

        if has_wildcard && has_result_option_arm {
            let start = node.match_token.span.start();
            self.violations.push(
                Violation::new(
                    self.checker.code(),
                    self.checker.name(),
                    self.checker.severity().into(),
                    "Match on Result/Option uses wildcard pattern. Explicitly handle all variants for better error handling.",
                    self.file_path,
                    start.line,
                    start.column + 1,
                )
                .with_suggestion(self.checker.suggestions()),
            );
        }

        syn::visit::visit_expr_match(self, node);
    }
}

fn is_wildcard_pattern(pat: &syn::Pat) -> bool {
    matches!(pat, syn::Pat::Wild(_))
}

fn is_result_or_option_pattern(pat: &syn::Pat) -> bool {
    if let syn::Pat::TupleStruct(ts) = pat {
        let path_str = ts.path.segments.iter()
            .map(|s| s.ident.to_string())
            .collect::<Vec<_>>()
            .join("::");
        return matches!(path_str.as_str(), "Ok" | "Err" | "Some" | "None" |
                       "Result::Ok" | "Result::Err" | "Option::Some" | "Option::None");
    }
    if let syn::Pat::Ident(ident) = pat {
        let name = ident.ident.to_string();
        return matches!(name.as_str(), "None");
    }
    if let syn::Pat::Path(path) = pat {
        let path_str = path.path.segments.iter()
            .map(|s| s.ident.to_string())
            .collect::<Vec<_>>()
            .join("::");
        return matches!(path_str.as_str(), "None" | "Option::None");
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detects_wildcard_on_result() {
        let code = r#"
            fn example(r: Result<i32, String>) {
                match r {
                    Ok(v) => println!("{}", v),
                    _ => {},
                }
            }
        "#;

        let syntax = syn::parse_file(code).unwrap();
        let checker = E1305NonExhaustiveMatch::default();

        let mut violations = Vec::new();
        for item in &syntax.items {
            violations.extend(checker.check_item(item, "test.rs").unwrap());
        }

        assert_eq!(violations.len(), 1);
    }

    #[test]
    fn test_explicit_match_passes() {
        let code = r#"
            fn example(r: Result<i32, String>) {
                match r {
                    Ok(v) => println!("{}", v),
                    Err(e) => eprintln!("{}", e),
                }
            }
        "#;

        let syntax = syn::parse_file(code).unwrap();
        let checker = E1305NonExhaustiveMatch::default();

        let mut violations = Vec::new();
        for item in &syntax.items {
            violations.extend(checker.check_item(item, "test.rs").unwrap());
        }

        assert_eq!(violations.len(), 0);
    }
}
