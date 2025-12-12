/// E1503: Lock poisoning mishandled
/// Severity: MED
/// LLM confusion: 3 (MED)
///
/// Description: When a thread panics while holding a Mutex, the Mutex becomes "poisoned" to
/// indicate that the protected data might be in an inconsistent state. Using `.unwrap()` on
/// lock() will panic if the Mutex is poisoned. Fix by handling poisoned locks explicitly with
/// `into_inner()` or by using a different synchronization primitive.
///
/// ## The Poison Cascade Problem
///
/// ```text
/// Thread A: panics while holding lock
///     ↓
/// Mutex becomes "poisoned"
///     ↓
/// Thread B: calls mutex.lock().unwrap()
///     ↓
/// PANIC! The poison spreads
/// ```
///
/// ## Why This Matters
///
/// 1. **Cascade failures**: One panic causes all threads to panic
/// 2. **Hidden bugs**: Original panic masked by poison panic
/// 3. **No recovery**: Application can't gracefully handle the error
/// 4. **Data might be fine**: Often the data is still usable
///
/// ## The Right Solutions
///
/// ### Option 1: Recover despite poisoning
/// ```rust
/// use std::sync::Mutex;
///
/// let mutex = Mutex::new(42);
/// let guard = mutex.lock().unwrap_or_else(|poisoned| {
///     eprintln!("Lock was poisoned, recovering");
///     poisoned.into_inner()
/// });
/// ```
///
/// ### Option 2: Handle as error
/// ```rust
/// use std::sync::Mutex;
///
/// fn get_value(mutex: &Mutex<i32>) -> Result<i32, &'static str> {
///     mutex.lock()
///         .map(|guard| *guard)
///         .map_err(|_| "Lock poisoned")
/// }
/// ```
///
/// ### Option 3: Use parking_lot (no poisoning)
/// ```rust,no_run
/// use parking_lot::Mutex;
///
/// let mutex = Mutex::new(42);
/// let guard = mutex.lock();  // No Result, no poisoning!
/// ```
///
/// Mitigation: Handle poisoned locks explicitly: use `lock().unwrap_or_else(|e| e.into_inner())`
/// to recover the data despite poisoning. Consider whether your data can be safely used after a
/// panic. Use RwLock or other primitives if lock poisoning is problematic.

use std::sync::{Arc, Mutex};

// ============================================================================
// DANGEROUS PATTERNS - NEVER DO THIS
// ============================================================================

/// PROBLEM E1503: Not handling poisoned lock properly
pub fn e1503_bad_unwrap(mutex: Arc<Mutex<i32>>) {
    // PROBLEM E1503: Not handling poisoned lock properly
    // PROBLEM E1002: direct unwrap/expect
    let mut guard = mutex.lock().unwrap(); // Can panic if poisoned
    *guard += 1;
}

/// PROBLEM E1503: expect doesn't help with poisoning
pub fn e1503_bad_expect(mutex: Arc<Mutex<i32>>) {
    // PROBLEM E1503: expect is just as bad
    let mut guard = mutex.lock().expect("Failed to acquire lock");
    *guard += 1;
}

/// PROBLEM E1503: Multiple locks with unwrap - cascade risk
pub fn e1503_bad_multiple_locks(m1: Arc<Mutex<i32>>, m2: Arc<Mutex<i32>>) {
    let mut g1 = m1.lock().unwrap();
    let mut g2 = m2.lock().unwrap(); // If m1 was poisoned, we already panicked
    *g1 += *g2;
}

/// Entry point for problem demonstration
pub fn e1503_entry() -> Result<(), Box<dyn std::error::Error>> {
    Ok(())
}

// ============================================================================
// GOOD ALTERNATIVES
// ============================================================================

/// GOOD: Recover data despite poisoning
pub fn e1503_good_recover(mutex: &Mutex<i32>) -> i32 {
    let guard = mutex.lock().unwrap_or_else(|poisoned| {
        eprintln!("Warning: Lock was poisoned, recovering data");
        poisoned.into_inner()
    });
    *guard
}

/// GOOD: Handle poisoning as an error
pub fn e1503_good_result(mutex: &Mutex<i32>) -> Result<i32, &'static str> {
    mutex
        .lock()
        .map(|guard| *guard)
        .map_err(|_| "Lock was poisoned")
}

/// GOOD: Check poison status first
pub fn e1503_good_check_poison(mutex: &Mutex<i32>) -> Option<i32> {
    if mutex.is_poisoned() {
        eprintln!("Mutex is poisoned, cannot safely access");
        return None;
    }
    // Still use proper error handling even after checking!
    mutex.lock().ok().map(|guard| *guard)
}

/// GOOD: Clear the poison when appropriate
pub fn e1503_good_clear_poison(mutex: &Mutex<i32>) {
    mutex.clear_poison();
    // Now the mutex is no longer poisoned
    if let Ok(guard) = mutex.lock() {
        println!("Value after clearing poison: {}", *guard);
    }
}

/// GOOD: Use map_err for custom error handling
pub fn e1503_good_map_err(mutex: &Mutex<i32>) -> Result<i32, String> {
    mutex
        .lock()
        .map(|guard| *guard)
        .map_err(|e| format!("Lock poisoned by panic: {:?}", e))
}

/// GOOD: Recover and log details
pub fn e1503_good_recover_with_logging(mutex: &Mutex<i32>) -> i32 {
    match mutex.lock() {
        Ok(guard) => *guard,
        Err(poisoned) => {
            eprintln!(
                "Lock was poisoned. Recovering data. \
                 Consider investigating the original panic."
            );
            *poisoned.into_inner()
        }
    }
}

/// GOOD: Wrapper that handles poisoning internally
pub struct SafeCounter {
    inner: Mutex<i32>,
}

impl SafeCounter {
    pub fn new(value: i32) -> Self {
        Self {
            inner: Mutex::new(value),
        }
    }

    pub fn get(&self) -> i32 {
        *self.inner
            .lock()
            .unwrap_or_else(|p| p.into_inner())
    }

    pub fn increment(&self) {
        let mut guard = self.inner.lock().unwrap_or_else(|p| p.into_inner());
        *guard += 1;
    }
}

// ============================================================================
// GOOD EXAMPLES unit tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use std::panic;

    #[test]
    fn test_good_recover_from_poison() {
        let mutex = Arc::new(Mutex::new(42));
        let mutex_clone = Arc::clone(&mutex);

        // Poison the mutex by panicking while holding it
        let result = panic::catch_unwind(move || {
            let _guard = mutex_clone.lock().unwrap();
            panic!("intentional panic to poison mutex");
        });

        assert!(result.is_err()); // Thread panicked
        assert!(mutex.is_poisoned()); // Mutex is now poisoned

        // Our safe function should still work!
        let value = e1503_good_recover(&mutex);
        assert_eq!(value, 42);
    }

    #[test]
    fn test_good_result_returns_error_when_poisoned() {
        let mutex = Arc::new(Mutex::new(42));
        let mutex_clone = Arc::clone(&mutex);

        let _ = panic::catch_unwind(move || {
            let _guard = mutex_clone.lock().unwrap();
            panic!("poison");
        });

        let result = e1503_good_result(&mutex);
        assert!(result.is_err());
    }

    #[test]
    fn test_safe_counter() {
        let counter = SafeCounter::new(0);
        counter.increment();
        counter.increment();
        assert_eq!(counter.get(), 2);
    }
}
