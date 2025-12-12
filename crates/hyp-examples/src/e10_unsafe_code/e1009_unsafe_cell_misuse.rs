/// E1009: UnsafeCell misuse and interior mutability violations
/// Severity: HIGH
/// LLM confusion: 5 (HIGHEST)
///
/// Description: UnsafeCell is Rust's primitive for interior mutability, allowing mutation through
/// shared references. However, it provides NO safety guarantees - you must manually ensure that
/// mutable access is exclusive. Using UnsafeCell incorrectly can create multiple mutable aliases
/// to the same data, violating Rust's aliasing rules and causing undefined behavior. This is like
/// having two mutable references to the same memory location, which Rust normally prevents.
///
/// Mitigation: Use safe wrappers like `Cell<T>`, `RefCell<T>`, `Mutex<T>`, or `RwLock<T>` instead
/// of UnsafeCell directly. If you must use UnsafeCell, carefully document your synchronization
/// strategy and ensure exclusive access. Use `#![warn(clippy::undocumented_unsafe_blocks)]` and
/// add detailed safety comments explaining why each unsafe block is sound.
use std::cell::UnsafeCell;

pub struct BadCell<T> {
    value: UnsafeCell<T>,
}

impl<T> BadCell<T> {
    pub fn new(value: T) -> Self {
        BadCell {
            value: UnsafeCell::new(value),
        }
    }

    // PROBLEM E1009: Returns multiple mutable references to the same data!
    // This violates Rust's aliasing rules and causes undefined behavior
    pub fn e1009_bad_get_mut(&self) -> &mut T {
        // PROBLEM E1003: Direct use of unsafe code
        unsafe {
            // PROBLEM E1904: No safety documentation
            // DANGER: This creates aliasing mutable references
            &mut *self.value.get()
        }
    }
}

pub fn e1009_bad_unsafe_cell_misuse() {
    let cell = BadCell::new(42);

    // PROBLEM E1009: Two mutable references to the same memory!
    let ref1 = cell.e1009_bad_get_mut();
    let ref2 = cell.e1009_bad_get_mut();

    // Undefined behavior: modifying through both references
    *ref1 = 100;
    *ref2 = 200;

    // Which value is it? Undefined!
    println!("Value: {}", *ref1);
}

pub fn e1009_entry() -> Result<(), Box<dyn std::error::Error>> {
    e1009_bad_unsafe_cell_misuse();
    Ok(())
}

// ============================================================================
// GOOD EXAMPLES - Proper alternatives
// ============================================================================

use std::cell::{Cell, RefCell};
use std::sync::{Mutex, RwLock};
use crate::test_constants::MAGIC_I32;

/// GOOD: Use Cell for simple Copy types
pub fn e1009_good_use_cell() {
    let cell = Cell::new(MAGIC_I32);
    cell.set(100); // Safe interior mutability
    let value = cell.get();
    println!("Value: {}", value);
}

/// GOOD: Use RefCell for runtime borrow checking
pub fn e1009_good_use_refcell() {
    let cell = RefCell::new(MAGIC_I32);

    // Borrow mutably - panics if already borrowed
    {
        let mut borrow = cell.borrow_mut();
        *borrow = MAGIC_I32;
    } // Mutable borrow ends here

    // Now we can borrow again
    let value = cell.borrow();
    println!("Value: {}", *value);
}

/// GOOD: Use RwLock for read-heavy workloads with thread-safe interior mutability
pub fn e1009_good_use_rwlock() -> Result<i32, Box<dyn std::error::Error>> {
    let rwlock = RwLock::new(MAGIC_I32);

    // Write lock for modification
    {
        let mut writer = rwlock.write().map_err(|e| format!("Lock poisoned: {:?}", e))?;
        *writer = MAGIC_I32.saturating_mul(MAGIC_I32);
    } // Write lock released here

    // Read lock for reading (multiple readers can access concurrently)
    let reader = rwlock.read().map_err(|e| format!("Lock poisoned: {:?}", e))?;
    Ok(*reader)
}

/// GOOD: Use Mutex for write-dominated workloads with atomic operations
///
/// This demonstrates a counter/accumulator pattern where EVERY access modifies state.
/// RwLock is inappropriate here because:
/// 1. Every operation is a write (100% write rate)
/// 2. We need atomic read-modify-write semantics
/// 3. RwLock adds overhead for read/write distinction we don't need
/// 4. Multiple sequential modifications must be atomic together
///
/// This is a legitimate Mutex use case - not every shared state benefits from RwLock.
pub fn e1009_good_use_mutex() -> Result<i32, Box<dyn std::error::Error>> {
    // Simulating an accumulator that processes multiple operations atomically
    let accumulator = Mutex::new(0_i32);

    // Operation 1: Add base value
    {
        let mut guard = accumulator.lock().map_err(|e| format!("Lock poisoned: {:?}", e))?;
        *guard = guard.saturating_add(MAGIC_I32);  // Write operation (using checked arithmetic)
    }

    // Operation 2: Multiply by factor (atomic with read-modify-write)
    {
        let mut guard = accumulator.lock().map_err(|e| format!("Lock poisoned: {:?}", e))?;
        let current = *guard;  // Read current state
        *guard = current.saturating_mul(2);  // Write new state based on read (using checked arithmetic)
        // With RwLock, another thread could modify between read and write locks
    }

    // Operation 3: Add offset
    {
        let mut guard = accumulator.lock().map_err(|e| format!("Lock poisoned: {:?}", e))?;
        *guard = guard.saturating_add(10);  // Write operation (using checked arithmetic)
    }

    // Final read
    let guard = accumulator.lock().map_err(|e| format!("Lock poisoned: {:?}", e))?;
    Ok(*guard)
}

/// GOOD: use atomic variables when possible
pub fn e1009_good_use_atomic() -> i32 {
    use std::sync::atomic::{AtomicI32, Ordering};

    let atomic = AtomicI32::new(MAGIC_I32);
    atomic.store(100, Ordering::SeqCst);
    atomic.load(Ordering::SeqCst)
}

/// GOOD: Use try_borrow to handle conflicts
pub fn e1009_good_try_borrow() {
    let cell = RefCell::new(MAGIC_I32);

    let borrow1 = cell.borrow_mut();
    match cell.try_borrow_mut() {
        Ok(_) => println!("Got second borrow"),
        Err(_) => println!("Already borrowed - handled gracefully"),
    }
    drop(borrow1);
}

// ============================================================================
// GOOD EXAMPLES unit tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn e1009_good_use_cell_allows_updates() {
    e1009_good_use_cell();
    }

    #[test]
    fn e1009_good_use_refcell_borrows_mutably() {
    e1009_good_use_refcell();
    }

    #[test]
    fn e1009_good_use_rwlock_updates_value() {
        let result = e1009_good_use_rwlock();
        assert!(result.is_ok());
        const EXPECTED: i32 = 1764; // 42 * 42
        assert_eq!(result.unwrap(), EXPECTED);
    }

    #[test]
    fn e1009_good_use_mutex_updates_value() {
        let result = e1009_good_use_mutex();
        assert!(result.is_ok());
        // Accumulator: 0 + 42 = 42, 42 * 2 = 84, 84 + 10 = 94
        const EXPECTED: i32 = 94;
        assert_eq!(result.unwrap(), EXPECTED);
    }

    #[test]
    fn e1009_good_use_atomic_updates_value() {
        const EXPECTED: i32 = 100;
        assert_eq!(e1009_good_use_atomic(), EXPECTED);
    }

    #[test]
    fn e1009_good_try_borrow_handles_conflict() {
        e1009_good_try_borrow();
    }
}
