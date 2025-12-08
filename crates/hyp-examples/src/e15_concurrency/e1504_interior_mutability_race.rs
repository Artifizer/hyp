/// E1504: Interior mutability race
/// Severity: HIGH
/// LLM confusion: 4 (HIGH)
///
/// Description: Interior mutability types like Cell and RefCell allow mutation through shared
/// references, but they're not thread-safe (not Sync). Cell doesn't use any synchronization,
/// so sharing it across threads would cause data races. Fix by using thread-safe alternatives
/// like Mutex, RwLock, or atomic types.
///
/// ## The Sync Trait Problem
///
/// ```text
/// Cell<T>    → NOT Sync (no synchronization)
/// RefCell<T> → NOT Sync (runtime borrow checking, not thread-safe)
/// Mutex<T>   → Sync (synchronized access)
/// RwLock<T>  → Sync (synchronized access)
/// AtomicI32  → Sync (hardware-level atomics)
/// ```
///
/// ## Why This Matters
///
/// 1. **Compile error**: Rust prevents sharing non-Sync types
/// 2. **Data races**: If bypassed with unsafe, causes undefined behavior
/// 3. **LLM confusion**: LLMs suggest Cell/RefCell for shared state
/// 4. **Subtle distinction**: Interior mutability ≠ thread safety
///
/// ## The Right Solutions
///
/// ### Option 1: Use Mutex for thread-safe interior mutability
/// ```rust
/// use std::sync::Mutex;
///
/// let counter = Mutex::new(0);
/// // Multiple threads can safely access through &counter
/// ```
///
/// ### Option 2: Use atomics for simple values
/// ```rust
/// use std::sync::atomic::{AtomicI32, Ordering};
///
/// let counter = AtomicI32::new(0);
/// counter.fetch_add(1, Ordering::SeqCst);
/// ```
///
/// ### Option 3: Use RwLock for read-heavy workloads
/// ```rust
/// use std::sync::RwLock;
///
/// let data = RwLock::new(vec![1, 2, 3]);
/// // Multiple readers OR one writer
/// ```
///
/// Mitigation: Use `Mutex<T>` or `RwLock<T>` for thread-safe interior mutability. Use atomic
/// types (`AtomicI32`, etc.) for simple values. The compiler prevents this at compile time -
/// Cell and RefCell are not Sync. Understand the difference between Send and Sync traits.

use std::cell::Cell;
use std::sync::atomic::{AtomicI32, Ordering};
use std::sync::{Arc, Mutex, RwLock};
use std::thread;

// ============================================================================
// PROBLEMATIC PATTERNS
// ============================================================================

/// PROBLEM E1504: Cell is not Sync, sharing across threads is unsafe
pub fn e1504_bad_cell_not_sync() {
    let data = Cell::new(0);
    // PROBLEM E1504: Cell is not Sync, sharing across threads is unsafe
    // This won't compile if we try to share &data across threads
    // The following would fail:
    // thread::spawn(|| { data.set(1); });  // Error: Cell is not Sync

    // We can only use it in single-threaded context
    data.set(42);
    let _value = data.get();
}

/// PROBLEM E1504: RefCell is also not Sync
pub fn e1504_bad_refcell_not_sync() {
    use std::cell::RefCell;

    let data = RefCell::new(vec![1, 2, 3]);
    // PROBLEM E1504: RefCell is not Sync
    // Cannot share across threads without Arc<Mutex<...>>

    // Single-threaded use is fine
    data.borrow_mut().push(4);
}

/// Entry point for problem demonstration
pub fn e1504_entry() -> Result<(), Box<dyn std::error::Error>> {
    e1504_bad_cell_not_sync();
    Ok(())
}

// ============================================================================
// GOOD ALTERNATIVES
// ============================================================================

/// GOOD: Use Mutex for thread-safe interior mutability
pub fn e1504_good_mutex() {
    let counter = Arc::new(Mutex::new(0));
    let mut handles = vec![];

    for _ in 0..10 {
        let counter = Arc::clone(&counter);
        let handle = thread::spawn(move || {
            let mut guard = counter.lock().unwrap();
            *guard += 1;
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }

    assert_eq!(*counter.lock().unwrap(), 10);
}

/// GOOD: Use atomics for simple counters
pub fn e1504_good_atomic() {
    let counter = Arc::new(AtomicI32::new(0));
    let mut handles = vec![];

    for _ in 0..10 {
        let counter = Arc::clone(&counter);
        let handle = thread::spawn(move || {
            counter.fetch_add(1, Ordering::SeqCst);
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }

    assert_eq!(counter.load(Ordering::SeqCst), 10);
}

/// GOOD: Use RwLock for read-heavy workloads
pub fn e1504_good_rwlock() {
    let data = Arc::new(RwLock::new(vec![1, 2, 3]));
    let mut handles = vec![];

    // Multiple readers
    for i in 0..5 {
        let data = Arc::clone(&data);
        let handle = thread::spawn(move || {
            let guard = data.read().unwrap();
            println!("Reader {}: {:?}", i, *guard);
        });
        handles.push(handle);
    }

    // One writer
    {
        let data = Arc::clone(&data);
        let handle = thread::spawn(move || {
            let mut guard = data.write().unwrap();
            guard.push(4);
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }
}

/// GOOD: Thread-safe wrapper for Cell-like behavior
pub struct ThreadSafeCell<T> {
    inner: Mutex<T>,
}

impl<T: Copy> ThreadSafeCell<T> {
    pub fn new(value: T) -> Self {
        Self {
            inner: Mutex::new(value),
        }
    }

    pub fn get(&self) -> T {
        *self.inner.lock().unwrap()
    }

    pub fn set(&self, value: T) {
        *self.inner.lock().unwrap() = value;
    }
}

/// GOOD: Use atomic types for flags
pub struct AtomicFlag {
    value: std::sync::atomic::AtomicBool,
}

impl AtomicFlag {
    pub fn new(value: bool) -> Self {
        Self {
            value: std::sync::atomic::AtomicBool::new(value),
        }
    }

    pub fn get(&self) -> bool {
        self.value.load(Ordering::SeqCst)
    }

    pub fn set(&self, value: bool) {
        self.value.store(value, Ordering::SeqCst);
    }

    pub fn toggle(&self) -> bool {
        self.value.fetch_xor(true, Ordering::SeqCst)
    }
}

// ============================================================================
// GOOD EXAMPLES unit tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mutex_counter() {
        e1504_good_mutex();
    }

    #[test]
    fn test_atomic_counter() {
        e1504_good_atomic();
    }

    #[test]
    fn test_thread_safe_cell() {
        let cell = Arc::new(ThreadSafeCell::new(0));
        let mut handles = vec![];

        for i in 0..5 {
            let cell = Arc::clone(&cell);
            let handle = thread::spawn(move || {
                cell.set(i);
            });
            handles.push(handle);
        }

        for handle in handles {
            handle.join().unwrap();
        }

        // Value will be one of 0-4 (last writer wins)
        let value = cell.get();
        assert!(value < 5);
    }

    #[test]
    fn test_atomic_flag() {
        let flag = Arc::new(AtomicFlag::new(false));
        let flag_clone = Arc::clone(&flag);

        let handle = thread::spawn(move || {
            flag_clone.set(true);
        });

        handle.join().unwrap();
        assert!(flag.get());
    }
}
