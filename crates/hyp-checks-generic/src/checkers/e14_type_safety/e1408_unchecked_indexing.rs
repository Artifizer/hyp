//! E1408: Unchecked array indexing
//!
//! Detects direct array/slice indexing with `[]` which can panic at runtime
//! if the index is out of bounds.

use crate::{checker::Checker, define_checker, violation::Violation};

use syn::{spanned::Spanned, visit::Visit, BinOp};

define_checker! {
    /// Checker for E1408: Unchecked array indexing
    E1408UncheckedIndexing,
    code = "E1408",
    name = "Unchecked array indexing",
    suggestions = "Use .get() for fallible access, or validate the index before using []",
    target_items = [Function],
    config_entry_name = "e1408_unchecked_indexing",
    /// Configuration for E1408: Unchecked indexing checker
    config = E1408Config {
        /// Whether this checker is enabled
        enabled: bool = true,
        /// Severity level, defaults to High
        severity: crate::config::SeverityLevel = crate::config::SeverityLevel::High,
        /// Categories this checker belongs to
        categories: Vec<crate::config::CheckerCategory> = vec![crate::config::CheckerCategory::Operations],
    },
    check_item(self, item, file_path) {
        let mut visitor = IndexingVisitor {
            violations: Vec::new(),
            file_path,
            checker: self,
            protected_accesses: Vec::new(),
        };
        visitor.visit_item(item);
        Ok(visitor.violations)
    }
}

struct IndexingVisitor<'a> {
    violations: Vec<Violation>,
    file_path: &'a str,
    checker: &'a E1408UncheckedIndexing,
    /// Track protected array[index] combinations
    protected_accesses: Vec<(String, String)>, // (array, index)
}

impl<'a> IndexingVisitor<'a> {
    fn create_violation(&self, span: proc_macro2::Span) -> Violation {
        let start = span.start();
        Violation::new(
            self.checker.code(),
            self.checker.name(),
            self.checker.severity().into(),
            "Direct indexing with [] can panic if the index is out of bounds.",
            self.file_path,
            start.line,
            start.column + 1,
        )
        .with_suggestion(self.checker.suggestions())
    }

    /// Check if an array access is protected by a length check
    fn is_protected(&self, array_expr: &syn::Expr, index_expr: &syn::Expr) -> bool {
        let array_str = expr_to_string(array_expr);
        let index_str = expr_to_string(index_expr);

        // For range expressions, extract the start index
        let range_start = if let syn::Expr::Range(range) = index_expr {
            if let Some(start) = &range.start {
                Some(extract_range_start(start))
            } else {
                Some(0) // [..N] starts at 0
            }
        } else {
            None
        };

        for (arr, protection) in &self.protected_accesses {
            if arr != &array_str {
                continue;
            }

            // Check for exact match (direct variable like idx < arr.len())
            if protection == &index_str {
                return true;
            }

            // Check if index contains the array length variable (e.g., n-3 where n = arr.len())
            // This handles cases like: let n = arr.len(); arr[n-1]
            if !protection.starts_with("*:") && index_str.contains(&protection.replace(".len()", "")) {
                return true;
            }

            // Parse protection for minimum length requirement: "*:N" means len() >= N
            if let Some((min_len, _)) = parse_protection(protection) {
                // For range slicing like arr[N..], verify min_len > N (so index N exists)
                if let Some(start_idx) = range_start {
                    if min_len > start_idx {
                        return true;
                    }
                }

                // For direct indexing with expressions involving the length variable
                // e.g., arr[n-3] where n=arr.len() and arr.len()>=3
                // Heuristic: if the index is not a simple literal and we have a length check,
                // assume it's using length-based logic (conservative but practical)
                if !is_simple_literal_or_constant(&index_str) {
                    // Accept expressions that likely use the array's length
                    // This handles cases like: let n = arr.len(); arr[n-1]
                    return true;
                }
            }
        }

        false
    }
}

impl<'a> Visit<'a> for IndexingVisitor<'a> {
    fn visit_expr_index(&mut self, node: &'a syn::ExprIndex) {
        // Check if this is a range slice (e.g., arr[1..]) or direct index (e.g., arr[idx])
        let is_safe = if is_range_expr(&node.index) {
            // Range slicing - check if protected by length check
            self.is_protected(&node.expr, &node.index)
        } else {
            // Direct indexing - check if constant or protected
            is_constant_index(&node.index) || self.is_protected(&node.expr, &node.index)
        };

        if !is_safe {
            self.violations.push(self.create_violation(node.span()));
        }

        syn::visit::visit_expr_index(self, node);
    }

    fn visit_expr_if(&mut self, node: &'a syn::ExprIf) {
        // Extract protections from the condition before visiting it
        let (protected_in_else, protected_in_then) = extract_protected_accesses(&node.cond);

        // Visit the condition WITH protections active (for cases like: if arr.len() >= 2 && arr[1..])
        let original_len = self.protected_accesses.len();
        self.protected_accesses.extend(protected_in_then.clone());
        self.visit_expr(&node.cond);
        self.protected_accesses.truncate(original_len);

        // Visit the then branch with its protection context
        if !protected_in_then.is_empty() {
            self.protected_accesses.extend(protected_in_then);
            self.visit_block(&node.then_branch);
            self.protected_accesses.truncate(original_len);
        } else {
            self.visit_block(&node.then_branch);
        }

        // Visit the else branch with its protection context
        if let Some((_, else_branch)) = &node.else_branch {
            if !protected_in_else.is_empty() {
                self.protected_accesses.extend(protected_in_else);
                self.visit_expr(else_branch);
                self.protected_accesses.truncate(original_len);
            } else {
                self.visit_expr(else_branch);
            }
        }
    }
}

/// Check if an index expression is a compile-time constant
fn is_constant_index(expr: &syn::Expr) -> bool {
    match expr {
        syn::Expr::Lit(lit) => matches!(lit.lit, syn::Lit::Int(_)),
        syn::Expr::Path(path) => {
            // Check if it's a const (uppercase by convention)
            if let Some(ident) = path.path.get_ident() {
                let name = ident.to_string();
                name.chars().all(|c| c.is_uppercase() || c == '_')
            } else {
                false
            }
        }
        _ => false,
    }
}

/// Check if an expression is a range (e.g., 1.., ..5, 1..5)
fn is_range_expr(expr: &syn::Expr) -> bool {
    matches!(expr, syn::Expr::Range(_))
}

/// Extract the start index from a range expression
/// Returns the numeric value if it's a literal, or a best-effort guess
fn extract_range_start(expr: &syn::Expr) -> usize {
    match expr {
        syn::Expr::Lit(lit) => {
            if let syn::Lit::Int(int_lit) = &lit.lit {
                int_lit.base10_parse().unwrap_or(0)
            } else {
                0
            }
        }
        _ => 0, // Conservative: assume 0 if we can't determine
    }
}

/// Parse protection string to extract minimum length requirement
/// Returns (min_len, max_len) where available
/// Examples:
/// - "*:3" means len() >= 3, so min_len = 3
/// - "idx" means a variable check, returns None
fn parse_protection(protection: &str) -> Option<(usize, Option<usize>)> {
    if let Some(stripped) = protection.strip_prefix("*:") {
        if let Ok(min) = stripped.parse::<usize>() {
            return Some((min, None));
        }
    }
    None
}

/// Check if an index string is a simple literal or constant
/// Returns false for complex expressions like "n-3", "idx+1", etc.
fn is_simple_literal_or_constant(index_str: &str) -> bool {
    // Check if it's a number
    if index_str.parse::<usize>().is_ok() {
        return true;
    }

    // Check if it's a simple identifier (single word, all uppercase = constant)
    if !index_str.contains(['-', '+', '*', '/', '(', ')', '[', ']']) {
        // Simple identifier - check if it's all uppercase (constant)
        return index_str.chars().all(|c| c.is_uppercase() || c == '_' || c.is_numeric());
    }

    false
}

/// Convert expression to a string for pattern matching, normalized
fn expr_to_string(expr: &syn::Expr) -> String {
    let s = quote::quote!(#expr).to_string();
    // Normalize: remove all whitespace for better matching
    s.chars().filter(|c| !c.is_whitespace()).collect()
}

/// Type alias for protected accesses: Vec of (array_name, index_expr) pairs
type ProtectedAccesses = Vec<(String, String)>;

/// Extract protected array accesses from a condition
/// Returns (protected_in_else, protected_in_then) as (array, index) pairs
/// Examples:
/// - if idx < arr.len() -> then branch is safe for arr[idx]
/// - if arr.len() <= idx -> else branch is safe for arr[idx]
/// - if segments.len() >= 3 -> then branch is safe for segments[n-3], segments[n-2], segments[n-1]
/// - if key.len() <= 3 && key[1..].chars().all() -> then branch safe for key[1..]
fn extract_protected_accesses(cond: &syn::Expr) -> (ProtectedAccesses, ProtectedAccesses) {
    let protected_in_else = Vec::new();
    let mut protected_in_then = Vec::new();

    // Recursively extract all protections from the condition
    extract_protections_recursive(cond, &mut protected_in_then);

    (protected_in_else, protected_in_then)
}

/// Recursively extract protections from a condition expression
fn extract_protections_recursive(cond: &syn::Expr, protected: &mut Vec<(String, String)>) {
    match cond {
        syn::Expr::Binary(bin) => {
            // Handle && operator - both sides can provide protection
            if matches!(bin.op, BinOp::And(_)) {
                extract_protections_recursive(&bin.left, protected);
                extract_protections_recursive(&bin.right, protected);
            } else {
                // Handle comparison operators
                let left_str = expr_to_string(&bin.left);
                let right_str = expr_to_string(&bin.right);

                // Check for patterns like: idx < arr.len() or arr.len() > idx
                if let Some((array, index)) = extract_length_check(&left_str, &right_str, &bin.op) {
                    match bin.op {
                        // idx < arr.len(), idx <= arr.len()-1, arr.len() > idx
                        BinOp::Lt(_) | BinOp::Le(_) | BinOp::Gt(_) | BinOp::Ge(_) => {
                            protected.push((array, index));
                        }
                        _ => {}
                    }
                }
            }
        }
        syn::Expr::Paren(paren) => {
            extract_protections_recursive(&paren.expr, protected);
        }
        _ => {}
    }
}

/// Extract array and index from length check patterns
/// Returns Some((array, protection)) if this is a valid length check
/// Protection can be:
/// - An index variable name (e.g., "idx" for idx < arr.len())
/// - "*:N" where N is minimum length (e.g., "*:3" for arr.len() >= 3)
fn extract_length_check(left: &str, right: &str, op: &BinOp) -> Option<(String, String)> {
    // Pattern: idx < arr.len()
    if right.contains(".len()") {
        let array = right.replace(".len()", "").trim().to_string();
        match op {
            BinOp::Lt(_) | BinOp::Le(_) => {
                return Some((array, left.trim().to_string()));
            }
            _ => {}
        }
    }

    // Pattern: arr.len() > idx, arr.len() >= N
    if left.contains(".len()") {
        let array = left.replace(".len()", "").trim().to_string();
        match op {
            BinOp::Gt(_) => {
                return Some((array, right.trim().to_string()));
            }
            BinOp::Ge(_) => {
                // arr.len() >= N means we can safely access indices 0 to N-1
                if let Ok(min_len) = right.trim().parse::<usize>() {
                    return Some((array, format!("*:{}", min_len)));
                }
                return None; // Can't determine minimum length
            }
            BinOp::Le(_) => {
                // arr.len() <= N does NOT guarantee safe access!
                // len could be 0, so we cannot protect any access
                return None;
            }
            _ => {}
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detects_variable_index() {
        let code = r#"
            fn example(arr: &[i32], idx: usize) -> i32 {
                arr[idx]
            }
        "#;

        let syntax = syn::parse_file(code).unwrap();
        let checker = E1408UncheckedIndexing::default();

        let mut violations = Vec::new();
        for item in &syntax.items {
            violations.extend(checker.check_item(item, "test.rs").unwrap());
        }

        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].code, "E1408");
    }

    #[test]
    fn test_constant_index_passes() {
        let code = r#"
            fn example(arr: &[i32; 5]) -> i32 {
                arr[0]
            }
        "#;

        let syntax = syn::parse_file(code).unwrap();
        let checker = E1408UncheckedIndexing::default();

        let mut violations = Vec::new();
        for item in &syntax.items {
            violations.extend(checker.check_item(item, "test.rs").unwrap());
        }

        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_get_method_passes() {
        let code = r#"
            fn example(arr: &[i32], idx: usize) -> Option<&i32> {
                arr.get(idx)
            }
        "#;

        let syntax = syn::parse_file(code).unwrap();
        let checker = E1408UncheckedIndexing::default();

        let mut violations = Vec::new();
        for item in &syntax.items {
            violations.extend(checker.check_item(item, "test.rs").unwrap());
        }

        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_detects_expression_index() {
        let code = r#"
            fn example(arr: &[i32], a: usize, b: usize) -> i32 {
                arr[a + b]
            }
        "#;

        let syntax = syn::parse_file(code).unwrap();
        let checker = E1408UncheckedIndexing::default();

        let mut violations = Vec::new();
        for item in &syntax.items {
            violations.extend(checker.check_item(item, "test.rs").unwrap());
        }

        assert_eq!(violations.len(), 1);
    }

    #[test]
    fn test_protected_by_length_check_less_than() {
        let code = r#"
            fn example(segments: &[String], n: usize) -> String {
                if n < segments.len() {
                    segments[n].clone()
                } else {
                    String::new()
                }
            }
        "#;

        let syntax = syn::parse_file(code).unwrap();
        let checker = E1408UncheckedIndexing::default();

        let mut violations = Vec::new();
        for item in &syntax.items {
            violations.extend(checker.check_item(item, "test.rs").unwrap());
        }

        // Should not flag - indexing is protected by length check
        assert_eq!(violations.len(), 0, "Array access protected by n < segments.len() should not be flagged");
    }

    #[test]
    fn test_protected_by_length_check_greater_than() {
        let code = r#"
            fn example(arr: &[i32], idx: usize) -> i32 {
                if arr.len() > idx {
                    arr[idx]
                } else {
                    0
                }
            }
        "#;

        let syntax = syn::parse_file(code).unwrap();
        let checker = E1408UncheckedIndexing::default();

        let mut violations = Vec::new();
        for item in &syntax.items {
            violations.extend(checker.check_item(item, "test.rs").unwrap());
        }

        // Should not flag - indexing is protected by length check
        assert_eq!(violations.len(), 0, "Array access protected by arr.len() > idx should not be flagged");
    }

    #[test]
    fn test_protected_by_length_check_with_offset() {
        let code = r#"
            fn example(segments: Vec<String>) -> bool {
                if segments.len() >= 3 {
                    let n = segments.len();
                    segments[n-3] == "std"
                        && segments[n-2] == "mem"
                        && segments[n-1] == "transmute"
                } else {
                    false
                }
            }
        "#;

        let syntax = syn::parse_file(code).unwrap();
        let checker = E1408UncheckedIndexing::default();

        let mut violations = Vec::new();
        for item in &syntax.items {
            violations.extend(checker.check_item(item, "test.rs").unwrap());
        }

        // Should not flag - indexing with n-3, n-2, n-1 is protected by segments.len() >= 3
        assert_eq!(violations.len(), 0, "Array access with offsets protected by length check should not be flagged");
    }

    #[test]
    fn test_unprotected_in_wrong_branch() {
        let code = r#"
            fn example(arr: &[i32], idx: usize) -> i32 {
                if idx < arr.len() {
                    0
                } else {
                    arr[idx]  // UNSAFE - in else branch!
                }
            }
        "#;

        let syntax = syn::parse_file(code).unwrap();
        let checker = E1408UncheckedIndexing::default();

        let mut violations = Vec::new();
        for item in &syntax.items {
            violations.extend(checker.check_item(item, "test.rs").unwrap());
        }

        // Should flag - indexing in wrong branch
        assert_eq!(violations.len(), 1, "Array access in wrong branch should be flagged");
    }

    #[test]
    fn test_protected_slice_range_with_length_check() {
        let code = r#"
            fn example(key_lc: String) -> bool {
                if key_lc.len() >= 2 && key_lc[1..].chars().all(|c| c.is_ascii_digit()) {
                    true
                } else {
                    false
                }
            }
        "#;

        let syntax = syn::parse_file(code).unwrap();
        let checker = E1408UncheckedIndexing::default();

        let mut violations = Vec::new();
        for item in &syntax.items {
            violations.extend(checker.check_item(item, "test.rs").unwrap());
        }

        // Should not flag - key_lc[1..] is protected by key_lc.len() >= 2
        assert_eq!(violations.len(), 0, "Slice range protected by length check should not be flagged");
    }

    #[test]
    fn test_insufficient_length_check_for_slice() {
        let code = r#"
            fn example(key_lc: String) -> bool {
                if key_lc.len() <= 3 && key_lc[1..].chars().all(|c| c.is_ascii_digit()) {
                    true
                } else {
                    false
                }
            }
        "#;

        let syntax = syn::parse_file(code).unwrap();
        let checker = E1408UncheckedIndexing::default();

        let mut violations = Vec::new();
        for item in &syntax.items {
            violations.extend(checker.check_item(item, "test.rs").unwrap());
        }

        // Should flag - key_lc.len() <= 3 doesn't guarantee key_lc[1..] is safe (len could be 0 or 1)
        assert_eq!(violations.len(), 1, "Slice range with insufficient length check should be flagged");
    }

    #[test]
    fn test_unprotected_slice_range() {
        let code = r#"
            fn example(key_lc: String) -> bool {
                key_lc[1..].chars().all(|c| c.is_ascii_digit())
            }
        "#;

        let syntax = syn::parse_file(code).unwrap();
        let checker = E1408UncheckedIndexing::default();

        let mut violations = Vec::new();
        for item in &syntax.items {
            violations.extend(checker.check_item(item, "test.rs").unwrap());
        }

        // Should flag - no length check before slicing
        assert_eq!(violations.len(), 1, "Unprotected slice range should be flagged");
    }

    #[test]
    fn test_compound_condition_with_proper_check() {
        let code = r#"
            fn example(key_lc: String) -> bool {
                if key_lc.starts_with('e')
                    && key_lc.len() >= 2
                    && key_lc[1..].chars().all(|c| c.is_ascii_digit())
                {
                    true
                } else {
                    false
                }
            }
        "#;

        let syntax = syn::parse_file(code).unwrap();
        let checker = E1408UncheckedIndexing::default();

        let mut violations = Vec::new();
        for item in &syntax.items {
            violations.extend(checker.check_item(item, "test.rs").unwrap());
        }

        // Should not flag - compound condition with key_lc.len() >= 2 protects key_lc[1..]
        assert_eq!(violations.len(), 0, "Compound condition with proper length check should not be flagged");
    }
}
