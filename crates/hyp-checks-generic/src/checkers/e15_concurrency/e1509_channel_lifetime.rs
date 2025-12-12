//! E1509: Channel lifetime issues
//!
//! Detects potential channel lifetime issues such as creating channels in loops
//! or holding receivers/senders longer than needed.

use crate::{checker::Checker, define_checker, violation::Violation};

use syn::visit::Visit;

define_checker! {
    /// Checker for E1509: Channel lifetime issues
    E1509ChannelLifetime,
    code = "E1509",
    name = "Channel lifetime issue",
    suggestions = "Create channels outside loops, drop senders/receivers when no longer needed",
    target_items = [Function],
    config_entry_name = "e1509_channel_lifetime",
    config = E1509Config {
        enabled: bool = true,
        severity: crate::config::SeverityLevel = crate::config::SeverityLevel::Medium,
        categories: Vec<crate::config::CheckerCategory> = vec![crate::config::CheckerCategory::Operations],
    },
    check_item(self, item, file_path) {
        let mut visitor = ChannelVisitor {
            violations: Vec::new(),
            file_path,
            checker: self,
            in_loop: false,
        };
        visitor.visit_item(item);
        Ok(visitor.violations)
    }
}

struct ChannelVisitor<'a> {
    violations: Vec<Violation>,
    file_path: &'a str,
    checker: &'a E1509ChannelLifetime,
    in_loop: bool,
}

impl<'a> Visit<'a> for ChannelVisitor<'a> {
    fn visit_expr_for_loop(&mut self, node: &'a syn::ExprForLoop) {
        let was_in_loop = self.in_loop;
        self.in_loop = true;
        syn::visit::visit_expr_for_loop(self, node);
        self.in_loop = was_in_loop;
    }

    fn visit_expr_while(&mut self, node: &'a syn::ExprWhile) {
        let was_in_loop = self.in_loop;
        self.in_loop = true;
        syn::visit::visit_expr_while(self, node);
        self.in_loop = was_in_loop;
    }

    fn visit_expr_loop(&mut self, node: &'a syn::ExprLoop) {
        let was_in_loop = self.in_loop;
        self.in_loop = true;
        syn::visit::visit_expr_loop(self, node);
        self.in_loop = was_in_loop;
    }

    fn visit_expr_call(&mut self, node: &'a syn::ExprCall) {
        if self.in_loop {
            if let syn::Expr::Path(path) = &*node.func {
                let path_str = path
                    .path
                    .segments
                    .iter()
                    .map(|s| s.ident.to_string())
                    .collect::<Vec<_>>()
                    .join("::");

                // Check for channel creation in loops
                if path_str.contains("channel") || path_str.contains("sync_channel") {
                    use syn::spanned::Spanned;
                    let start = node.span().start();
                    self.violations.push(
                        Violation::new(
                            self.checker.code(),
                            self.checker.name(),
                            self.checker.severity().into(),
                            "Creating channel inside a loop. Consider creating the channel outside the loop.",
                            self.file_path,
                            start.line,
                            start.column + 1,
                        )
                        .with_suggestion(self.checker.suggestions()),
                    );
                }
            }
        }

        syn::visit::visit_expr_call(self, node);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::checker::Checker;

    #[test]
    fn test_detects_channel_in_loop() {
        let code = r#"
            use std::sync::mpsc;

            fn example() {
                for _ in 0..10 {
                    let (tx, rx) = mpsc::channel();
                }
            }
        "#;

        let syntax = syn::parse_file(code).unwrap();
        let checker = E1509ChannelLifetime::default();

        let mut violations = Vec::new();
        for item in &syntax.items {
            violations.extend(checker.check_item(item, "test.rs").unwrap());
        }

        assert_eq!(violations.len(), 1);
    }

    #[test]
    fn test_channel_outside_loop_passes() {
        let code = r#"
            use std::sync::mpsc;

            fn example() {
                let (tx, rx) = mpsc::channel();
                for i in 0..10 {
                    tx.send(i).unwrap();
                }
            }
        "#;

        let syntax = syn::parse_file(code).unwrap();
        let checker = E1509ChannelLifetime::default();

        let mut violations = Vec::new();
        for item in &syntax.items {
            violations.extend(checker.check_item(item, "test.rs").unwrap());
        }

        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_no_channel_passes() {
        let code = r#"
            fn example() {
                for i in 0..10 {
                    println!("{}", i);
                }
            }
        "#;

        let syntax = syn::parse_file(code).unwrap();
        let checker = E1509ChannelLifetime::default();

        let mut violations = Vec::new();
        for item in &syntax.items {
            violations.extend(checker.check_item(item, "test.rs").unwrap());
        }

        assert_eq!(violations.len(), 0);
    }
}
