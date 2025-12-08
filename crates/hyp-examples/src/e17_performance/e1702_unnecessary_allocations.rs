/// E1702: Unnecessary allocations
/// Severity: LOW
/// LLM confusion: 2 (LOW)
///
/// Description: Allocating memory (like creating Strings) inside tight loops is slow because
/// allocation is expensive. This code allocates a new String on every iteration. Fix by reusing
/// allocations - create the String once and reuse it, or use `with_capacity()` to pre-allocate
/// the Vec to avoid repeated reallocations.
///
/// ## The Allocation Cost
///
/// ```text
/// for i in 0..1000 {
///     let s = format!("Item {}", i);  // Allocates on every iteration!
///     result.push(s);                  // Vec may reallocate too!
/// }
/// // 1000+ allocations
/// ```
///
/// ## Why This Matters
///
/// 1. **Performance**: Allocation is ~10-100x slower than stack ops
/// 2. **Memory fragmentation**: Many small allocations fragment heap
/// 3. **Cache misses**: Scattered allocations cause cache misses
/// 4. **GC pressure**: More allocations = more cleanup work
///
/// ## The Right Solutions
///
/// ### Option 1: Pre-allocate with capacity
/// ```rust
/// let mut result = Vec::with_capacity(1000);  // One allocation
/// for i in 0..1000 {
///     result.push(format!("Item {}", i));
/// }
/// ```
///
/// ### Option 2: Reuse buffer
/// ```rust
/// let mut buffer = String::with_capacity(100);
/// for i in 0..1000 {
///     buffer.clear();  // Reuse allocation
///     write!(&mut buffer, "Item {}", i).unwrap();
///     process(&buffer);
/// }
/// ```
///
/// ### Option 3: Use stack allocation
/// ```rust
/// for i in 0..1000 {
///     let mut buffer = [0u8; 32];  // Stack allocated
///     let s = format_into(&mut buffer, i);
///     process(s);
/// }
/// ```
///
/// Mitigation: Use `Vec::with_capacity()` to pre-allocate when you know the size. Reuse String
/// buffers with `.clear()` instead of creating new ones. Use `format_args!` for formatting without
/// allocation. Profile to identify allocation hotspots.

// ============================================================================
// WASTEFUL PATTERNS
// ============================================================================

/// PROBLEM E1702: Allocating in loop without pre-allocation
pub fn e1702_bad_no_capacity(count: usize) -> Vec<String> {
    let mut result = Vec::new(); // No capacity hint!
    for i in 0..count {
        // PROBLEM E1702: Allocating String in loop instead of reusing
        let s = format!("Item {}", i);
        result.push(s); // Vec may reallocate multiple times
    }
    result
}

/// PROBLEM E1702: Creating new String each iteration
pub fn e1702_bad_new_string_each(items: &[i32]) -> String {
    let mut result = String::new();
    for item in items {
        let s = format!("{},", item); // New allocation each time!
        result.push_str(&s);
    }
    result
}

/// Entry point for problem demonstration
pub fn e1702_entry() -> Result<(), Box<dyn std::error::Error>> {
    Ok(())
}

// ============================================================================
// GOOD ALTERNATIVES
// ============================================================================

/// GOOD: Pre-allocate Vec with capacity
pub fn e1702_good_with_capacity(count: usize) -> Vec<String> {
    let mut result = Vec::with_capacity(count); // One allocation for Vec
    for i in 0..count {
        result.push(format!("Item {}", i));
    }
    result
}

/// GOOD: Use write! to avoid intermediate String
pub fn e1702_good_write_direct(items: &[i32]) -> String {
    use std::fmt::Write;

    let mut result = String::with_capacity(items.len() * 4); // Estimate size
    for item in items {
        write!(&mut result, "{},", item).unwrap(); // No intermediate allocation
    }
    result
}

/// GOOD: Reuse buffer with clear()
pub fn e1702_good_reuse_buffer(items: &[i32]) {
    use std::fmt::Write;

    let mut buffer = String::with_capacity(32);
    for item in items {
        buffer.clear(); // Reuse allocation
        write!(&mut buffer, "Processing: {}", item).unwrap();
        println!("{}", buffer);
    }
}

/// GOOD: Use collect with size hint
pub fn e1702_good_collect(count: usize) -> Vec<String> {
    (0..count).map(|i| format!("Item {}", i)).collect()
    // Iterator has size hint, Vec allocates once
}

/// GOOD: Use join for concatenation
pub fn e1702_good_join(items: &[i32]) -> String {
    items
        .iter()
        .map(|i| i.to_string())
        .collect::<Vec<_>>()
        .join(",")
}

/// GOOD: Use itertools for intersperse
pub fn e1702_good_intersperse(items: &[i32]) -> String {
    use std::fmt::Write;

    let mut result = String::new();
    let mut first = true;
    for item in items {
        if !first {
            result.push(',');
        }
        first = false;
        write!(&mut result, "{}", item).unwrap();
    }
    result
}

/// GOOD: Stack allocation for small buffers
pub fn e1702_good_stack_buffer() {
    use std::io::Write;

    for i in 0..100 {
        let mut buffer = [0u8; 32]; // Stack allocated
        let len = {
            let mut cursor = std::io::Cursor::new(&mut buffer[..]);
            write!(cursor, "Item {}", i).unwrap();
            cursor.position() as usize
        };
        let s = std::str::from_utf8(&buffer[..len]).unwrap();
        println!("{}", s);
    }
}

// ============================================================================
// GOOD EXAMPLES unit tests
// ============================================================================


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_with_capacity() {
        let result = e1702_good_with_capacity(100);
        assert_eq!(result.len(), 100);
        assert_eq!(result[0], "Item 0");
    }

    #[test]
    fn test_write_direct() {
        let items = vec![1, 2, 3];
        let result = e1702_good_write_direct(&items);
        assert_eq!(result, "1,2,3,");
    }

    #[test]
    fn test_collect() {
        let result = e1702_good_collect(5);
        assert_eq!(result.len(), 5);
    }

    #[test]
    fn test_join() {
        let items = vec![1, 2, 3];
        let result = e1702_good_join(&items);
        assert_eq!(result, "1,2,3");
    }

    #[test]
    fn test_intersperse() {
        let items = vec![1, 2, 3];
        let result = e1702_good_intersperse(&items);
        assert_eq!(result, "1,2,3");
    }
}
