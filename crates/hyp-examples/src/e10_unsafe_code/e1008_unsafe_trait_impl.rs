/// E1008: Unsafe trait implementation
/// Severity: HIGH
/// LLM confusion: 4 (HIGH)
///
/// Description: Implementing an unsafe trait (like Send or Sync) promises the compiler that your
/// type meets certain safety requirements. If you implement these traits incorrectly, you can cause
/// data races, memory corruption, or other undefined behavior - and the compiler can't check if
/// you're right. It's like promising "this type is thread-safe" when it actually isn't. The compiler
/// trusts your promise and allows multi-threaded access, which then causes bugs.
///
/// Mitigation: Only implement unsafe traits when you fully understand their safety requirements.
/// Document why your implementation is safe. Use `#![warn(clippy::undocumented_unsafe_blocks)]`.
/// Prefer using safe wrappers (Arc, Mutex) instead of implementing Send/Sync manually. Test
/// thoroughly with thread sanitizers.

struct MyType {
    _data: *const i32,
}

// PROBLEM E1008: Implementing unsafe trait without proper safety guarantees
unsafe impl Send for MyType {}

pub fn e1008_bad_unsafe_trait_impl() {
    let _ = MyType {
        _data: std::ptr::null(),
    };
}

pub fn e1008_entry() -> Result<(), Box<dyn std::error::Error>> {
    e1008_bad_unsafe_trait_impl();
    Ok(())
}

// ============================================================================
// GOOD EXAMPLES - Proper alternatives
// ============================================================================

use std::sync::Arc;

/// GOOD: Use safe wrappers that implement Send/Sync correctly
pub struct GoodType {
    data: Arc<i32>, // Arc is Send + Sync when T: Send + Sync
}

impl GoodType {
    pub fn new(value: i32) -> Self {
        GoodType {
            data: Arc::new(value),
        }
    }
}

/// GOOD: If implementing unsafe traits, document thoroughly
struct DocumentedSendType {
    /// This raw pointer is only used for reading, never dereferenced
    /// across threads. The pointed-to data is actually owned by an
    /// Arc elsewhere, ensuring it lives long enough.
    _data: *const i32,
}

// SAFETY: DocumentedSendType only contains a raw pointer that is:
// 1. Never dereferenced (used only for identification)
// 2. Points to data with 'static lifetime (leaked Box)
// 3. Never written to from any thread
// Therefore, sending this type across threads is safe.
unsafe impl Send for DocumentedSendType {}

/// GOOD: Use interior mutability with proper synchronization
use std::sync::Mutex;

pub struct ThreadSafeType {
    data: Mutex<i32>,
}

impl ThreadSafeType {
    pub fn new(value: i32) -> Self {
        ThreadSafeType {
            data: Mutex::new(value),
        }
    }
}

// ThreadSafeType is automatically Send + Sync because Mutex<T> is
//
// ============================================================================
// GOOD EXAMPLES unit tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn e1008_good_type_constructs() {
        let value = GoodType::new(10);
        let cloned = value.data.clone();
        assert_eq!(*cloned, 10);
    }

    #[test]
    fn e1008_good_thread_safe_type_locks() {
        let wrapper = ThreadSafeType::new(5);
        {
            let mut guard = wrapper.data.lock().unwrap();
            *guard = 6;
        }
        assert_eq!(*wrapper.data.lock().unwrap(), 6);
    }
}
