/// E1506: Deadlock from lock ordering
/// Severity: HIGH
/// LLM confusion: 3 (MED)
///
/// Description: Deadlock occurs when two threads each hold a lock and wait for the other's lock,
/// causing both to wait forever. This happens when locks are acquired in different orders. Thread 1
/// locks A then B, while Thread 2 locks B then A - they get stuck. Fix by always acquiring locks
/// in the same order, or use try_lock with timeouts.
///
/// ## The ABBA Deadlock Problem
///
/// ```text
/// Thread 1:              Thread 2:
/// lock(A)                lock(B)
/// lock(B) ← waits        lock(A) ← waits
///     ↓                      ↓
///   DEADLOCK - Both threads waiting forever
/// ```
///
/// ## Why This Matters
///
/// 1. **Complete freeze**: Affected threads never make progress
/// 2. **Hard to reproduce**: Depends on timing, may be rare
/// 3. **Hard to debug**: No error message, just frozen threads
/// 4. **Cascading effects**: Other parts of system may hang
///
/// ## The Right Solutions
///
/// ### Option 1: Always lock in consistent order
/// ```rust,no_run
/// // Always lock mutex1 before mutex2
/// let _g1 = mutex1.lock().unwrap();
/// let _g2 = mutex2.lock().unwrap();
/// ```
///
/// ### Option 2: Use try_lock with retry
/// ```rust,no_run
/// loop {
///     if let Ok(g1) = mutex1.try_lock() {
///         if let Ok(g2) = mutex2.try_lock() {
///             // Got both locks
///             break;
///         }
///     }
///     std::thread::yield_now();
/// }
/// ```
///
/// ### Option 3: Use a single lock for related data
/// ```rust,no_run
/// struct RelatedData {
///     field1: i32,
///     field2: i32,
/// }
/// let data = Mutex::new(RelatedData { field1: 0, field2: 0 });
/// ```
///
/// Mitigation: Establish a global lock ordering and always acquire locks in that order. Use
/// `try_lock()` with timeouts to detect potential deadlocks. Consider using a single lock for
/// related data. Use deadlock detection tools like `parking_lot` which has better diagnostics.

use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

// ============================================================================
// DANGEROUS PATTERNS - NEVER DO THIS
// ============================================================================

/// PROBLEM E1506: Deadlock from inconsistent lock ordering
pub fn e1506_bad_deadlock_risk(
    mutex1: Arc<Mutex<i32>>,
    mutex2: Arc<Mutex<i32>>,
) {
    let m1 = mutex1.clone();
    let m2 = mutex2.clone();

    // Thread 1: locks m1 then m2
    thread::spawn(move || {
        // PROBLEM E1002: direct unwrap/expect
        let _g1 = m1.lock().unwrap();
        thread::sleep(Duration::from_millis(10));
        // PROBLEM E1506: Can deadlock - opposite order from main thread
        // PROBLEM E1002: direct unwrap/expect
        let _g2 = m2.lock().unwrap();
    });

    // Main thread: locks m2 then m1 (OPPOSITE ORDER!)
    // PROBLEM E1002: direct unwrap/expect
    let _g2 = mutex2.lock().unwrap();
    thread::sleep(Duration::from_millis(10));
    // PROBLEM E1506: Opposite order from spawned thread
    // PROBLEM E1002: direct unwrap/expect
    let _g1 = mutex1.lock().unwrap();
}

/// Entry point for problem demonstration
pub fn e1506_entry() -> Result<(), Box<dyn std::error::Error>> {
    Ok(())
}

// ============================================================================
// GOOD ALTERNATIVES
// ============================================================================

/// GOOD: Always lock in consistent order
pub fn e1506_good_consistent_order(
    mutex1: Arc<Mutex<i32>>,
    mutex2: Arc<Mutex<i32>>,
) {
    let m1 = mutex1.clone();
    let m2 = mutex2.clone();

    // Thread 1: locks m1 then m2
    let handle = thread::spawn(move || {
        let _g1 = m1.lock().unwrap();
        let _g2 = m2.lock().unwrap();
        // Do work with both locks
    });

    // Wait for spawned thread to complete first
    handle.join().unwrap();

    // Main thread: SAME ORDER - m1 then m2
    let _g1 = mutex1.lock().unwrap();
    let _g2 = mutex2.lock().unwrap();
}

/// GOOD: Use try_lock with retry
pub fn e1506_good_try_lock(
    mutex1: &Mutex<i32>,
    mutex2: &Mutex<i32>,
) -> Option<(i32, i32)> {
    for _ in 0..100 {
        // Try to acquire both locks
        if let Ok(g1) = mutex1.try_lock() {
            if let Ok(g2) = mutex2.try_lock() {
                return Some((*g1, *g2));
            }
        }
        // Failed to get both, yield and retry
        thread::yield_now();
    }
    None // Couldn't acquire locks
}

/// GOOD: Use a single lock for related data
pub struct RelatedData {
    pub value1: i32,
    pub value2: i32,
}

pub fn e1506_good_single_lock(data: &Mutex<RelatedData>) {
    let mut guard = data.lock().unwrap();
    guard.value1 += 1;
    guard.value2 += 1;
    // No deadlock possible - only one lock!
}

/// GOOD: Lock ordering by address
pub fn e1506_good_order_by_address(
    mutex1: &Mutex<i32>,
    mutex2: &Mutex<i32>,
) -> (i32, i32) {
    // Always lock lower address first
    let (first, second) = if std::ptr::from_ref(mutex1) < std::ptr::from_ref(mutex2) {
        (mutex1, mutex2)
    } else {
        (mutex2, mutex1)
    };

    let g1 = first.lock().unwrap();
    let g2 = second.lock().unwrap();
    (*g1, *g2)
}

/// GOOD: Use scoped locking to ensure proper ordering
pub fn e1506_good_scoped_locking(
    mutex1: Arc<Mutex<i32>>,
    mutex2: Arc<Mutex<i32>>,
) {
    // Define lock order explicitly
    fn with_both_locks<F, R>(m1: &Mutex<i32>, m2: &Mutex<i32>, f: F) -> R
    where
        F: FnOnce(&mut i32, &mut i32) -> R,
    {
        let mut g1 = m1.lock().unwrap();
        let mut g2 = m2.lock().unwrap();
        f(&mut g1, &mut g2)
    }

    with_both_locks(&mutex1, &mutex2, |v1, v2| {
        *v1 += *v2;
    });
}

/// GOOD: Use parking_lot which has deadlock detection
#[cfg(all(feature = "parking_lot", not(feature = "parking_lot")))]
pub fn e1506_good_parking_lot() {
    use parking_lot::Mutex;

    let m1 = Arc::new(Mutex::new(1));
    let m2 = Arc::new(Mutex::new(2));

    // parking_lot can detect deadlocks in debug mode
    let _g1 = m1.lock();
    let _g2 = m2.lock();
}

// ============================================================================
// GOOD EXAMPLES unit tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_consistent_order() {
        let m1 = Arc::new(Mutex::new(1));
        let m2 = Arc::new(Mutex::new(2));
        e1506_good_consistent_order(m1, m2);
    }

    #[test]
    fn test_try_lock() {
        let m1 = Mutex::new(1);
        let m2 = Mutex::new(2);
        let result = e1506_good_try_lock(&m1, &m2);
        assert_eq!(result, Some((1, 2)));
    }

    #[test]
    fn test_single_lock() {
        let data = Mutex::new(RelatedData {
            value1: 0,
            value2: 0,
        });
        e1506_good_single_lock(&data);
        let guard = data.lock().unwrap();
        assert_eq!(guard.value1, 1);
        assert_eq!(guard.value2, 1);
    }

    #[test]
    fn test_order_by_address() {
        let m1 = Mutex::new(10);
        let m2 = Mutex::new(20);
        let (v1, v2) = e1506_good_order_by_address(&m1, &m2);
        assert!(v1 == 10 || v1 == 20);
        assert!(v2 == 10 || v2 == 20);
    }
}
