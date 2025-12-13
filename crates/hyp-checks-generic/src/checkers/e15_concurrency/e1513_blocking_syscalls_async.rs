//! E1513: Blocking syscalls inside async code
//!
//! Blocking syscalls in async functions block the Tokio executor thread,
//! preventing other tasks from making progress. This causes severe
//! performance degradation and potential deadlocks in microservices.
//!
//! Example:
//! ```text
//! // Bad: Blocks the executor thread
//! async fn read_config() -> String {
//!     std::fs::read_to_string("config.json").unwrap()
//! }
//!
//! // Good: Uses async I/O
//! async fn read_config() -> String {
//!     tokio::fs::read_to_string("config.json").await.unwrap()
//! }
//! ```

use crate::{checker::Checker, define_checker, violation::Violation};

use syn::{spanned::Spanned, visit::Visit};

define_checker! {
    /// Checker for E1513: Blocking syscalls in async code
    E1513BlockingSyscallsAsync,
    code = "E1513",
    name = "Blocking syscall in async code",
    suggestions = "Use async alternatives: tokio::fs::* for file I/O, tokio::net::* for networking, tokio::time::sleep for sleeping.",
    target_items = [Function],
    config_entry_name = "e1513_blocking_syscalls_async",
    config = E1513Config {
        enabled: bool = true,
        severity: crate::config::SeverityLevel = crate::config::SeverityLevel::Medium,
        categories: Vec<crate::config::CheckerCategory> = vec![crate::config::CheckerCategory::Operations],
    },
    check_item(self, item, file_path) {
        let mut visitor = BlockingSyscallVisitor {
            violations: Vec::new(),
            file_path,
            checker: self,
        };
        visitor.visit_item(item);
        Ok(visitor.violations)
    }
}

struct BlockingSyscallVisitor<'a> {
    violations: Vec<Violation>,
    file_path: &'a str,
    checker: &'a E1513BlockingSyscallsAsync,
}

/// Blocking call patterns to detect
struct BlockingPattern {
    /// Path segments to match (e.g., ["std", "fs", "read"])
    segments: &'static [&'static str],
    /// Description of what's blocked
    description: &'static str,
    /// Suggested async alternative
    suggestion: &'static str,
}

const BLOCKING_PATTERNS: &[BlockingPattern] = &[
    // std::fs operations
    BlockingPattern {
        segments: &["std", "fs", "read"],
        description: "std::fs::read blocks the executor",
        suggestion: "Use tokio::fs::read().await instead",
    },
    BlockingPattern {
        segments: &["std", "fs", "write"],
        description: "std::fs::write blocks the executor",
        suggestion: "Use tokio::fs::write().await instead",
    },
    BlockingPattern {
        segments: &["std", "fs", "read_to_string"],
        description: "std::fs::read_to_string blocks the executor",
        suggestion: "Use tokio::fs::read_to_string().await instead",
    },
    BlockingPattern {
        segments: &["std", "fs", "metadata"],
        description: "std::fs::metadata blocks the executor",
        suggestion: "Use tokio::fs::metadata().await instead",
    },
    BlockingPattern {
        segments: &["std", "fs", "remove_file"],
        description: "std::fs::remove_file blocks the executor",
        suggestion: "Use tokio::fs::remove_file().await instead",
    },
    BlockingPattern {
        segments: &["std", "fs", "create_dir"],
        description: "std::fs::create_dir blocks the executor",
        suggestion: "Use tokio::fs::create_dir().await instead",
    },
    BlockingPattern {
        segments: &["std", "fs", "create_dir_all"],
        description: "std::fs::create_dir_all blocks the executor",
        suggestion: "Use tokio::fs::create_dir_all().await instead",
    },
    BlockingPattern {
        segments: &["std", "fs", "copy"],
        description: "std::fs::copy blocks the executor",
        suggestion: "Use tokio::fs::copy().await instead",
    },
    BlockingPattern {
        segments: &["std", "fs", "rename"],
        description: "std::fs::rename blocks the executor",
        suggestion: "Use tokio::fs::rename().await instead",
    },
    // std::fs::File operations
    BlockingPattern {
        segments: &["fs", "File", "open"],
        description: "std::fs::File::open blocks the executor",
        suggestion: "Use tokio::fs::File::open().await instead",
    },
    BlockingPattern {
        segments: &["fs", "File", "create"],
        description: "std::fs::File::create blocks the executor",
        suggestion: "Use tokio::fs::File::create().await instead",
    },
    // std::net operations
    BlockingPattern {
        segments: &["net", "TcpStream", "connect"],
        description: "std::net::TcpStream::connect blocks the executor",
        suggestion: "Use tokio::net::TcpStream::connect().await instead",
    },
    BlockingPattern {
        segments: &["TcpStream", "connect"],
        description: "std::net::TcpStream::connect blocks the executor",
        suggestion: "Use tokio::net::TcpStream::connect().await instead",
    },
    BlockingPattern {
        segments: &["net", "TcpListener", "bind"],
        description: "std::net::TcpListener::bind blocks the executor",
        suggestion: "Use tokio::net::TcpListener::bind().await instead",
    },
    BlockingPattern {
        segments: &["TcpListener", "bind"],
        description: "std::net::TcpListener::bind blocks the executor",
        suggestion: "Use tokio::net::TcpListener::bind().await instead",
    },
    BlockingPattern {
        segments: &["net", "UdpSocket", "bind"],
        description: "std::net::UdpSocket::bind blocks the executor",
        suggestion: "Use tokio::net::UdpSocket::bind().await instead",
    },
    BlockingPattern {
        segments: &["UdpSocket", "bind"],
        description: "std::net::UdpSocket::bind blocks the executor",
        suggestion: "Use tokio::net::UdpSocket::bind().await instead",
    },
    // std::thread::sleep
    BlockingPattern {
        segments: &["thread", "sleep"],
        description: "std::thread::sleep blocks the executor",
        suggestion: "Use tokio::time::sleep().await instead",
    },
];

impl<'a> BlockingSyscallVisitor<'a> {
    /// Check if expression matches a blocking pattern
    fn check_blocking_call(&self, expr: &syn::Expr) -> Option<&'static BlockingPattern> {
        let segments = Self::extract_path_segments(expr)?;

        for pattern in BLOCKING_PATTERNS {
            if Self::matches_pattern(&segments, pattern.segments) {
                return Some(pattern);
            }
        }
        None
    }

    /// Extract path segments from a call expression
    fn extract_path_segments(expr: &syn::Expr) -> Option<Vec<String>> {
        match expr {
            syn::Expr::Call(call) => {
                if let syn::Expr::Path(path) = &*call.func {
                    Some(
                        path.path
                            .segments
                            .iter()
                            .map(|s| s.ident.to_string())
                            .collect(),
                    )
                } else {
                    None
                }
            }
            syn::Expr::MethodCall(method) => {
                // For method calls like `file.read()`, we check the method name
                // but this is less reliable since we don't have type info
                Some(vec![method.method.to_string()])
            }
            _ => None,
        }
    }

    /// Check if segments match a pattern (suffix match for flexibility)
    fn matches_pattern(segments: &[String], pattern: &[&str]) -> bool {
        if segments.len() < pattern.len() {
            return false;
        }

        // Check if segments end with the pattern
        let start = segments.len() - pattern.len();
        segments[start..]
            .iter()
            .zip(pattern.iter())
            .all(|(seg, pat)| seg == *pat)
    }
}

impl<'a> Visit<'a> for BlockingSyscallVisitor<'a> {
    fn visit_expr(&mut self, node: &'a syn::Expr) {
        if let Some(pattern) = self.check_blocking_call(node) {
            let span = node.span();
            self.violations.push(
                Violation::new(
                    self.checker.code(),
                    self.checker.name(),
                    self.checker.severity().into(),
                    pattern.description,
                    self.file_path,
                    span.start().line,
                    span.start().column + 1,
                )
                .with_suggestion(pattern.suggestion),
            );
        }

        syn::visit::visit_expr(self, node);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::checker::Checker;

    fn check_code(code: &str) -> Vec<Violation> {
        let checker = E1513BlockingSyscallsAsync::default();
        let file = syn::parse_file(code).expect("Failed to parse");
        let mut violations = Vec::new();
        for item in &file.items {
            violations.extend(checker.check_item(item, "test.rs").unwrap());
        }
        violations
    }

    #[test]
    fn test_detects_std_fs_read() {
        let code = r#"
            async fn example() {
                let data = std::fs::read("file.txt").unwrap();
            }
        "#;
        let violations = check_code(code);
        assert_eq!(violations.len(), 1);
        assert!(violations[0].code == "E1513");
    }

    #[test]
    fn test_detects_std_fs_read_to_string() {
        let code = r#"
            async fn example() {
                let content = std::fs::read_to_string("config.json").unwrap();
            }
        "#;
        let violations = check_code(code);
        assert_eq!(violations.len(), 1);
    }

    #[test]
    fn test_detects_thread_sleep() {
        let code = r#"
            use std::thread;
            use std::time::Duration;

            async fn example() {
                thread::sleep(Duration::from_secs(1));
            }
        "#;
        let violations = check_code(code);
        assert_eq!(violations.len(), 1);
    }

    #[test]
    fn test_detects_tcp_connect() {
        let code = r#"
            use std::net::TcpStream;

            async fn example() {
                let stream = TcpStream::connect("127.0.0.1:8080").unwrap();
            }
        "#;
        let violations = check_code(code);
        assert_eq!(violations.len(), 1);
    }

    #[test]
    fn test_tokio_fs_passes() {
        let code = r#"
            async fn example() {
                let data = tokio::fs::read("file.txt").await.unwrap();
            }
        "#;
        let violations = check_code(code);
        assert!(violations.is_empty());
    }

    #[test]
    fn test_tokio_time_sleep_passes() {
        let code = r#"
            async fn example() {
                tokio::time::sleep(std::time::Duration::from_secs(1)).await;
            }
        "#;
        let violations = check_code(code);
        assert!(violations.is_empty());
    }

    #[test]
    fn test_detects_multiple_blocking_calls() {
        let code = r#"
            async fn example() {
                let data = std::fs::read("a.txt").unwrap();
                let content = std::fs::read_to_string("b.txt").unwrap();
                std::fs::write("c.txt", "data").unwrap();
            }
        "#;
        let violations = check_code(code);
        assert_eq!(violations.len(), 3);
    }
}
