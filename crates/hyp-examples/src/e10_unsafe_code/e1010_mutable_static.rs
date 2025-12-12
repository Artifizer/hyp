/// E1010: Mutable static without synchronization
/// Severity: HIGH
/// LLM confusion: 4 (HIGH)
///
/// Description: Mutable global variables (static mut) can be accessed from any thread without
/// synchronization, causing data races. If two threads read and write the same global variable
/// simultaneously, the results are unpredictable - you might get corrupted data, crashes, or
/// security vulnerabilities. This is why accessing mutable statics requires unsafe code - it's
/// inherently dangerous. Use thread-safe alternatives instead.
///
/// Mitigation: Avoid mutable statics entirely. Use `static` with `Mutex<T>` or `RwLock<T>` for
/// thread-safe global state. Use `AtomicXxx` types for simple counters/flags. Use `thread_local!`
/// for thread-local state. If mutable statics are necessary, document the synchronization strategy.

static mut COUNTER: i32 = 0;

pub fn e1010_bad_mutable_static() {
    // PROBLEM E1003: Direct use of unsafe code
    unsafe {
        // PROBLEM E1004: No safety documentation
        // PROBLEM E1010: Accessing mutable static without synchronization
        COUNTER += 1;
    }
}

pub fn e1010_entry() -> Result<(), Box<dyn std::error::Error>> {
    e1010_bad_mutable_static();
    Ok(())
}

// ============================================================================
// GOOD EXAMPLES - Proper alternatives
// ============================================================================

use std::sync::atomic::{AtomicI32, Ordering};
use std::sync::Mutex;

/// GOOD: Use atomic types for simple counters
static GOOD_COUNTER: AtomicI32 = AtomicI32::new(0);

/// Safe alternative using atomic operations instead of mutable static
pub fn e1010_good_atomic() {
    GOOD_COUNTER.fetch_add(1, Ordering::SeqCst);
    let _value = GOOD_COUNTER.load(Ordering::SeqCst);
}

/// GOOD: Use Mutex for complex state
use std::sync::LazyLock;

static GOOD_STATE: LazyLock<Mutex<i32>> = LazyLock::new(|| Mutex::new(0));

/// Safe alternative using Mutex for synchronized mutable state
pub fn e1010_mutex() -> i32 {
    let mut guard = GOOD_STATE.lock().unwrap();
    *guard += 1;
    *guard
}

// GOOD: Use thread_local for per-thread state
thread_local! {
    static THREAD_COUNTER: std::cell::Cell<i32> = const { std::cell::Cell::new(0) };
}

/// Safe alternative using thread_local for per-thread mutable state
pub fn e1010_good_thread_local() -> i32 {
    THREAD_COUNTER.with(|c| {
        let new_val = c.get().saturating_add(1);
        c.set(new_val);
        new_val
    })
}

/// GOOD: Use const for immutable globals
const MAX_VALUE: i32 = 100;

/// Safe alternative using const for immutable global values
pub fn e1010_good_const() -> i32 {
    MAX_VALUE // No synchronization needed for immutable data
}

// ============================================================================
// GOOD EXAMPLES unit tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn e1010_good_atomic_increments() {
        e1010_good_atomic();
    }

    #[test]
    fn e1010_good_mutex_updates_state() {
        let result = e1010_mutex();
        assert!(result > 0);
    }

    #[test]
    fn e1010_good_thread_local_per_thread() {
        let result = e1010_good_thread_local();
        assert!(result > 0);
    }

    #[test]
    fn e1010_good_const_returns_max() {
        assert_eq!(e1010_good_const(), 100);
    }
}
