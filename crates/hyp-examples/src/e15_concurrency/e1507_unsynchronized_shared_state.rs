/// E1507: Unsynchronized shared state
/// Severity: HIGH
/// LLM confusion: 4 (HIGH)
///
/// Description: A data race occurs when multiple threads access the same memory location
/// concurrently, at least one is writing, and there's no synchronization. This code uses a
/// static mutable variable accessed from multiple threads without any locks or atomic operations,
/// causing undefined behavior. Fix by using Mutex, RwLock, or atomic types.
///
/// ## The Data Race Problem
///
/// ```text
/// static mut COUNTER: i32 = 0;
///
/// Thread 1: reads COUNTER (sees 0)
/// Thread 2: reads COUNTER (sees 0)
/// Thread 1: writes COUNTER = 1
/// Thread 2: writes COUNTER = 1  // Lost update!
///
/// Expected: 2, Actual: 1 (or worse - undefined behavior)
/// ```
///
/// ## Why This Matters
///
/// 1. **Undefined behavior**: Anything can happen
/// 2. **Lost updates**: Increments get lost
/// 3. **Torn reads/writes**: Partially updated values
/// 4. **Memory corruption**: Can corrupt other data
///
/// ## The Right Solutions
///
/// ### Option 1: Use Mutex for complex data
/// ```rust
/// use std::sync::Mutex;
/// use once_cell::sync::Lazy;
///
/// static COUNTER: Lazy<Mutex<i32>> = Lazy::new(|| Mutex::new(0));
/// ```
///
/// ### Option 2: Use atomics for simple values
/// ```rust
/// use std::sync::atomic::{AtomicI32, Ordering};
///
/// static COUNTER: AtomicI32 = AtomicI32::new(0);
/// COUNTER.fetch_add(1, Ordering::SeqCst);
/// ```
///
/// ### Option 3: Use thread-local storage
/// ```rust
/// thread_local! {
///     static COUNTER: std::cell::Cell<i32> = const { std::cell::Cell::new(0) };
/// }
/// ```
///
/// Mitigation: Never use `static mut` in multi-threaded code. Use `static` with `Mutex<T>`,
/// `RwLock<T>`, or atomic types. Use `lazy_static` or `once_cell` for safe static initialization.
/// Rust's type system prevents most data races, but `unsafe` code can bypass these protections.

use std::sync::atomic::{AtomicI32, Ordering};
use std::sync::{Arc, Mutex, RwLock};
use std::thread;

// ============================================================================
// DANGEROUS PATTERNS - NEVER DO THIS
// ============================================================================

/// PROBLEM E1507: Data race on static mut
pub fn e1507_bad_static_mut() {
    static mut COUNTER: i32 = 0;

    for _ in 0..10 {
        thread::spawn(|| {
            // PROBLEM E1003: Direct use of unsafe code
            unsafe {
                // PROBLEM E1904: No safety documentation
                // PROBLEM E1507: Data race on COUNTER
                COUNTER += 1;
            }
        });
    }
}

/// Entry point for problem demonstration
pub fn e1507_entry() -> Result<(), Box<dyn std::error::Error>> {
    Ok(())
}

// ============================================================================
// GOOD ALTERNATIVES
// ============================================================================

/// GOOD: Use AtomicI32 for simple counters
static ATOMIC_COUNTER: AtomicI32 = AtomicI32::new(0);

pub fn e1507_good_atomic() {
    let mut handles = vec![];

    for _ in 0..10 {
        let handle = thread::spawn(|| {
            ATOMIC_COUNTER.fetch_add(1, Ordering::SeqCst);
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }

    assert_eq!(ATOMIC_COUNTER.load(Ordering::SeqCst), 10);

    // Reset for next test
    ATOMIC_COUNTER.store(0, Ordering::SeqCst);
}

/// GOOD: Use Mutex with once_cell for complex data
use once_cell::sync::Lazy;

static MUTEX_DATA: Lazy<Mutex<Vec<i32>>> = Lazy::new(|| Mutex::new(Vec::new()));

pub fn e1507_good_lazy_mutex() {
    let mut handles = vec![];

    for i in 0..10 {
        let handle = thread::spawn(move || {
            MUTEX_DATA.lock().unwrap().push(i);
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }

    let data = MUTEX_DATA.lock().unwrap();
    assert_eq!(data.len(), 10);
}

/// GOOD: Use RwLock for read-heavy workloads
static RWLOCK_DATA: Lazy<RwLock<i32>> = Lazy::new(|| RwLock::new(0));

pub fn e1507_good_rwlock() {
    let mut handles = vec![];

    // Multiple readers
    for _ in 0..5 {
        let handle = thread::spawn(|| {
            let value = *RWLOCK_DATA.read().unwrap();
            println!("Read: {}", value);
        });
        handles.push(handle);
    }

    // One writer
    handles.push(thread::spawn(|| {
        *RWLOCK_DATA.write().unwrap() += 1;
    }));

    for handle in handles {
        handle.join().unwrap();
    }
}

// GOOD: Use thread-local storage when sharing isn't needed
thread_local! {
    static THREAD_LOCAL_COUNTER: std::cell::Cell<i32> = const { std::cell::Cell::new(0) };
}

pub fn e1507_good_thread_local() {
    let mut handles = vec![];

    for _ in 0..3 {
        let handle = thread::spawn(|| {
            // Each thread has its own counter
            THREAD_LOCAL_COUNTER.with(|c| {
                for _ in 0..10 {
                    c.set(c.get() + 1);
                }
                c.get()
            })
        });
        handles.push(handle);
    }

    let results: Vec<i32> = handles.into_iter().map(|h| h.join().unwrap()).collect();

    // Each thread counted to 10 independently
    assert!(results.iter().all(|&r| r == 10));
}

/// GOOD: Pass shared state explicitly via Arc
pub fn e1507_good_arc_mutex() {
    let counter = Arc::new(Mutex::new(0));
    let mut handles = vec![];

    for _ in 0..10 {
        let counter = Arc::clone(&counter);
        let handle = thread::spawn(move || {
            *counter.lock().unwrap() += 1;
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }

    assert_eq!(*counter.lock().unwrap(), 10);
}

/// GOOD: Use channels instead of shared state
pub fn e1507_good_channels() -> i32 {
    use std::sync::mpsc;

    let (tx, rx) = mpsc::channel();

    for i in 0..10 {
        let tx = tx.clone();
        thread::spawn(move || {
            tx.send(i).unwrap();
        });
    }

    drop(tx); // Close sender

    rx.into_iter().sum()
}

// ============================================================================
// GOOD EXAMPLES unit tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_atomic() {
        e1507_good_atomic();
    }

    #[test]
    fn test_arc_mutex() {
        e1507_good_arc_mutex();
    }

    #[test]
    fn test_thread_local() {
        e1507_good_thread_local();
    }

    #[test]
    fn test_channels() {
        let sum = e1507_good_channels();
        assert_eq!(sum, 45); // 0 + 1 + 2 + ... + 9
    }
}
