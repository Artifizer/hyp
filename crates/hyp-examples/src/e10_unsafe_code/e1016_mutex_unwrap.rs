#![doc(test(attr(ignore)))]

/// E1016: Mutex unwrap - lock poisoning and panic cascades
/// Severity: HIGH
/// LLM confusion: 3 (MED)
///
/// Description: Using `.lock().unwrap()` on a Mutex (or `.read()/.write().unwrap()` on RwLock)
/// is extremely dangerous due to Rust's lock poisoning mechanism. When a thread panics while
/// holding a mutex lock, the mutex becomes "poisoned" to indicate the protected data might be
/// in an inconsistent state. If you use `.unwrap()` on a poisoned lock, YOUR code will ALSO
/// panic, creating a cascade of failures across all threads.
///
/// ## The Panic Cascade Problem
///
/// ```text
/// Thread A: panics while holding lock (bug, assertion failure, etc.)
///     ↓
/// Mutex becomes "poisoned" (Rust's safety mechanism)
///     ↓
/// Thread B: calls mutex.lock().unwrap() → PANIC! (the unwrap fails)
///     ↓
/// Thread C: calls mutex.lock().unwrap() → PANIC!
///     ↓
/// Thread D: calls mutex.lock().unwrap() → PANIC!
///     ↓
/// ... entire application crashes from ONE original error
/// ```
///
/// ## Why This Matters
///
/// 1. **One bug crashes everything**: A single thread's panic spreads to all threads
/// 2. **Hard to debug**: The panic you see is not the original error
/// 3. **No graceful degradation**: Server can't handle the error and continue
/// 4. **Data corruption hidden**: The poisoning exists because data might be invalid!
///
/// ## The Right Solutions
///
/// ### Option 1: Recover despite poisoning (when data is likely fine)
/// ```rust,no_run
/// use std::sync::Mutex;
///
/// let mutex = Mutex::new(1);
/// let guard = mutex.lock().unwrap_or_else(|poisoned| {
///     eprintln!("Lock was poisoned by another thread, recovering");
///     poisoned.into_inner()
/// });
/// assert_eq!(*guard, 1);
/// ```
///
/// ### Option 2: Handle as error (when you need to know about poisoning)
/// ```rust,no_run
/// use std::sync::Mutex;
///
/// #[derive(Debug, PartialEq)]
/// enum MyError {
///     LockPoisoned,
/// }
///
/// fn do_work(mutex: &Mutex<i32>) -> Result<i32, MyError> {
///     let guard = mutex.lock().map_err(|_| MyError::LockPoisoned)?;
///     Ok(*guard)
/// }
///
/// let mutex = Mutex::new(7);
/// assert_eq!(do_work(&mutex), Ok(7));
/// ```
///
/// ### Option 3: Use parking_lot (no poisoning at all)
/// ```rust,no_run
/// use parking_lot::Mutex; // Different crate, no poisoning!
///
/// let mutex = Mutex::new(5);
/// let guard = mutex.lock(); // Returns MutexGuard directly, no Result!
/// assert_eq!(*guard, 5);
/// ```
///
/// Mitigation: Use `#![warn(clippy::unwrap_used)]` and specifically check for lock() patterns.
/// Consider whether lock poisoning is meaningful for your use case - often parking_lot is better.
use std::sync::{Arc, Mutex, RwLock};
use std::thread;

// ============================================================================
// DANGEROUS PATTERNS - NEVER DO THIS
// ============================================================================

/// PROBLEM E1016: Mutex lock().unwrap() - the classic panic cascade trigger
pub fn e1016_mutex_lock_unwrap(mutex: &Mutex<i32>) {
    // If ANY other thread panicked while holding this mutex,
    // this unwrap() will ALSO panic, spreading the failure!
    let guard = mutex.lock().unwrap(); // DANGEROUS!
    println!("Value: {}", *guard);
}

/// PROBLEM E1016: RwLock read().unwrap() - same problem
pub fn e1016_rwlock_read_unwrap(rwlock: &RwLock<String>) {
    let guard = rwlock.read().unwrap(); // DANGEROUS!
    println!("Value: {}", *guard);
}

/// PROBLEM E1016: RwLock write().unwrap() - same problem
pub fn e1016_rwlock_write_unwrap(rwlock: &RwLock<String>) {
    let mut guard = rwlock.write().unwrap(); // DANGEROUS!
    *guard = "new value".to_string();
}

/// PROBLEM E1016: Multiple threads with unwrap - demonstrates the cascade
pub fn e1016_panic_cascade_demo() {
    let data = Arc::new(Mutex::new(0));

    // Spawn a thread that will panic while holding the lock
    let data_clone = Arc::clone(&data);
    let panicking_thread = thread::spawn(move || {
        let _guard = data_clone.lock().unwrap();
        // Simulate work that panics
        panic!("Original error - maybe a bug, assertion failure, etc.");
    });

    // Wait for the panicking thread
    let _ = panicking_thread.join();

    // Now the mutex is poisoned - ANY thread using unwrap() will fail!
    // This line would panic with: "PoisonError { .. }"
    // let _guard = data.lock().unwrap(); // This would CASCADE the panic!

    println!("Demonstration: mutex is now poisoned");
}

// ============================================================================
// GOOD ALTERNATIVES
// ============================================================================

/// GOOD: Recover data despite poisoning
pub fn good_recover_from_poison(mutex: &Mutex<i32>) -> i32 {
    let guard = mutex.lock().unwrap_or_else(|poisoned| {
        eprintln!("Warning: Lock was poisoned, recovering data");
        poisoned.into_inner()
    });
    *guard
}

/// GOOD: Propagate poisoning as an error
pub fn good_propagate_as_error(mutex: &Mutex<i32>) -> Result<i32, &'static str> {
    let guard = mutex.lock().map_err(|_| "Lock was poisoned")?;
    Ok(*guard)
}

/// GOOD: Use is_poisoned() to check first
pub fn good_check_poison_status(mutex: &Mutex<i32>) -> Option<i32> {
    if mutex.is_poisoned() {
        eprintln!("Mutex is poisoned, cannot safely access");
        return None;
    }
    // Still use proper error handling even after checking!
    mutex.lock().ok().map(|guard| *guard)
}

/// GOOD: Clear the poison (rare, but sometimes needed)
pub fn good_clear_poison(mutex: &Mutex<i32>) {
    mutex.clear_poison();
    // Now the mutex is no longer poisoned, lock() will succeed
    if let Ok(guard) = mutex.lock() {
        println!("Value after clearing poison: {}", *guard);
    }
}

/// Entry point for problem demonstration
pub fn e1016_entry() -> Result<(), Box<dyn std::error::Error>> {
    // Demonstrate safe alternatives
    let mutex = Mutex::new(42);

    let recovered = good_recover_from_poison(&mutex);
    println!("Recovered value: {}", recovered);

    let result = good_propagate_as_error(&mutex);
    println!("Propagated result: {:?}", result);

    let checked = good_check_poison_status(&mutex);
    println!("Checked value: {:?}", checked);

    // Note: We don't run e1016_panic_cascade_demo() as it would create
    // a poisoned mutex that affects other code.

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_recover_from_poison_works_on_clean_mutex() {
        let mutex = Mutex::new(42);
        assert_eq!(good_recover_from_poison(&mutex), 42);
    }

    #[test]
    fn test_propagate_as_error_works() {
        let mutex = Mutex::new(42);
        assert_eq!(good_propagate_as_error(&mutex), Ok(42));
    }

    #[test]
    fn test_check_poison_status_on_clean_mutex() {
        let mutex = Mutex::new(42);
        assert_eq!(good_check_poison_status(&mutex), Some(42));
    }

    #[test]
    fn test_recover_from_poisoned_mutex() {
        let mutex = Arc::new(Mutex::new(42));
        let mutex_clone = Arc::clone(&mutex);

        // Poison the mutex by panicking while holding it
        let result = std::panic::catch_unwind(move || {
            let _guard = mutex_clone.lock().unwrap();
            panic!("intentional panic to poison mutex");
        });

        assert!(result.is_err()); // Thread panicked
        assert!(mutex.is_poisoned()); // Mutex is now poisoned

        // Our safe function should still work!
        let value = good_recover_from_poison(&mutex);
        assert_eq!(value, 42);
    }
}
