/// E1710: Large stack allocation
/// Severity: MED
/// LLM confusion: 3 (MED)
///
/// Description: The stack has limited size (typically a few MB). Allocating large arrays on the
/// stack can cause stack overflow, especially in recursive functions or deeply nested calls. This
/// allocates 1MB on the stack, which is risky. Fix by using heap allocation (Vec or Box) for
/// large data structures.
///
/// ## The Stack Overflow Problem
///
/// ```text
/// fn process() {
///     let buffer = [0u8; 1024 * 1024];  // 1MB on stack!
///     // ...
/// }
///
/// // In recursive function or deep call stack:
/// // STACK OVERFLOW!
/// ```
///
/// ## Why This Matters
///
/// 1. **Stack overflow**: Program crashes
/// 2. **Limited stack size**: Typically 1-8MB total
/// 3. **Recursive risk**: Each call adds to stack usage
/// 4. **Thread stacks**: Each thread has its own limited stack
///
/// ## The Right Solutions
///
/// ### Option 1: Use Vec for heap allocation
/// ```rust
/// let buffer = vec![0u8; 1024 * 1024];  // On heap
/// ```
///
/// ### Option 2: Use Box for single large values
/// ```rust
/// let buffer = Box::new([0u8; 1024 * 1024]);  // On heap
/// ```
///
/// ### Option 3: Use lazy_static for global buffers
/// ```rust,no_run
/// use once_cell::sync::Lazy;
///
/// static BUFFER: Lazy<Vec<u8>> = Lazy::new(|| vec![0u8; 1024 * 1024]);
/// ```
///
/// Mitigation: Use `Vec` or `Box` for large allocations instead of stack arrays. Keep stack
/// allocations small (< 1KB is safe, > 100KB is risky). Use `#![warn(clippy::large_stack_arrays)]`
/// to detect large stack allocations. Be especially careful in recursive functions.

// ============================================================================
// DANGEROUS PATTERNS
// ============================================================================

/// PROBLEM E1710: Allocating large array on stack
pub fn e1710_bad_large_stack() {
    // PROBLEM E1710: Allocating large array on stack
    let _large_array = [0u8; 1024 * 1024]; // 1MB on stack
}

/// PROBLEM E1710: Large stack in recursive function
pub fn e1710_bad_recursive_stack(depth: usize) {
    let _buffer = [0u8; 10240]; // 10KB per call

    if depth > 0 {
        e1710_bad_recursive_stack(depth - 1); // Stack grows!
    }
}

/// PROBLEM E1710: Multiple large stack allocations
pub fn e1710_bad_multiple_large() {
    let _a = [0u8; 100_000];
    let _b = [0u8; 100_000];
    let _c = [0u8; 100_000];
    // 300KB total on stack
}

/// Entry point for problem demonstration
pub fn e1710_entry() -> Result<(), Box<dyn std::error::Error>> {
    Ok(())
}

// ============================================================================
// GOOD ALTERNATIVES
// ============================================================================

/// GOOD: Use Vec for heap allocation
pub fn e1710_good_vec() -> Vec<u8> {
    vec![0u8; 1024 * 1024] // 1MB on heap
}

/// GOOD: Use Box for single large value
pub fn e1710_good_box() -> Box<[u8; 1024 * 1024]> {
    Box::new([0u8; 1024 * 1024])
}

/// GOOD: Use Vec in recursive function
pub fn e1710_good_recursive_heap(depth: usize) {
    let _buffer = vec![0u8; 10240]; // On heap

    if depth > 0 {
        e1710_good_recursive_heap(depth - 1);
    }
}

/// GOOD: Pre-allocate and pass reference
pub fn e1710_good_preallocate() {
    let buffer = vec![0u8; 1024 * 1024];
    e1710_process_buffer(&buffer);
}

fn e1710_process_buffer(buffer: &[u8]) {
    println!("Buffer size: {}", buffer.len());
}

/// GOOD: Use lazy_static for global buffer
use once_cell::sync::Lazy;

static LARGE_BUFFER: Lazy<Vec<u8>> = Lazy::new(|| vec![0u8; 1024 * 1024]);

pub fn e1710_good_static() {
    println!("Static buffer size: {}", LARGE_BUFFER.len());
}

/// GOOD: Small stack allocation is fine
pub fn e1710_good_small_stack() {
    let _buffer = [0u8; 1024]; // 1KB is fine
}

/// GOOD: Use streaming for large data
pub fn e1710_good_streaming<R: std::io::Read>(reader: &mut R) -> std::io::Result<usize> {
    let mut buffer = [0u8; 8192]; // Small buffer
    let mut total = 0;

    loop {
        let n = reader.read(&mut buffer)?;
        if n == 0 {
            break;
        }
        total += n;
        // Process chunk...
    }

    Ok(total)
}

/// GOOD: Use thread with larger stack if needed
pub fn e1710_good_thread_stack() {
    std::thread::Builder::new()
        .stack_size(8 * 1024 * 1024) // 8MB stack
        .spawn(|| {
            // Can use larger stack allocations here
            let _buffer = [0u8; 1024 * 1024];
        })
        .unwrap()
        .join()
        .unwrap();
}

// ============================================================================
// GOOD EXAMPLES unit tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vec() {
        let buffer = e1710_good_vec();
        assert_eq!(buffer.len(), 1024 * 1024);
    }

    #[test]
    fn test_box() {
        let buffer = e1710_good_box();
        assert_eq!(buffer.len(), 1024 * 1024);
    }

    #[test]
    fn test_recursive_heap() {
        e1710_good_recursive_heap(100); // Would overflow with stack allocation
    }

    #[test]
    fn test_static() {
        e1710_good_static();
    }

    #[test]
    fn test_streaming() {
        let data = vec![1u8; 10000];
        let mut cursor = std::io::Cursor::new(data);
        let total = e1710_good_streaming(&mut cursor).unwrap();
        assert_eq!(total, 10000);
    }
}
