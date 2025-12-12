/// E1508: Sleep instead of synchronization
/// Severity: LOW
/// LLM confusion: 2 (LOW)
///
/// Description: Using `thread::sleep` to wait for another thread is unreliable and inefficient.
/// The sleep duration is arbitrary - too short and you'll read before the other thread finishes,
/// too long and you waste time. Fix by using proper synchronization primitives like channels,
/// condition variables, or joining the thread.
///
/// ## The Sleep Problem
///
/// ```text
/// thread::spawn(|| { do_work(); });
/// thread::sleep(Duration::from_millis(100));  // Hope 100ms is enough?
/// read_result();  // Might not be ready!
/// ```
///
/// ## Why This Matters
///
/// 1. **Race condition**: Sleep might not be long enough
/// 2. **Wasted time**: Sleep might be too long
/// 3. **Non-deterministic**: Works sometimes, fails others
/// 4. **Poor practice**: Indicates misunderstanding of concurrency
///
/// ## The Right Solutions
///
/// ### Option 1: Use thread::join
/// ```rust
/// let handle = std::thread::spawn(|| { do_work(); });
/// handle.join().unwrap();  // Waits until thread completes
/// ```
///
/// ### Option 2: Use channels
/// ```rust
/// let (tx, rx) = std::sync::mpsc::channel();
/// std::thread::spawn(move || {
///     do_work();
///     tx.send(result).unwrap();
/// });
/// let result = rx.recv().unwrap();  // Blocks until ready
/// ```
///
/// ### Option 3: Use condition variables
/// ```rust,no_run
/// use std::sync::{Condvar, Mutex};
///
/// let pair = (Mutex::new(false), Condvar::new());
/// // Worker sets flag and notifies
/// // Main thread waits on condition
/// ```
///
/// Mitigation: Use `thread::join()` to wait for thread completion. Use channels (`mpsc::channel`)
/// to communicate between threads. Use condition variables (`Condvar`) for complex waiting
/// scenarios. Never rely on sleep for synchronization - it's a code smell.

use std::sync::{mpsc, Arc, Condvar, Mutex};
use std::thread;
use std::time::Duration;

// ============================================================================
// BAD PATTERNS - NEVER DO THIS
// ============================================================================

/// PROBLEM E1508: Using sleep instead of proper synchronization
pub fn e1508_bad_sleep_sync() {
    let data = Arc::new(Mutex::new(0));
    let data_clone = data.clone();

    thread::spawn(move || {
        // PROBLEM E1002: direct unwrap/expect
        let mut guard = data_clone.lock().unwrap();
        *guard = 42;
    });

    // PROBLEM E1508: Using sleep instead of proper synchronization
    thread::sleep(Duration::from_millis(100));

    // Might read before thread finishes!
    // PROBLEM E1002: direct unwrap/expect
    let _value = *data.lock().unwrap();
}

/// PROBLEM E1508: Sleep in a polling loop
pub fn e1508_bad_polling() {
    let flag = Arc::new(Mutex::new(false));
    let flag_clone = flag.clone();

    thread::spawn(move || {
        thread::sleep(Duration::from_millis(50));
        *flag_clone.lock().unwrap() = true;
    });

    // PROBLEM E1508: Polling with sleep
    while !*flag.lock().unwrap() {
        thread::sleep(Duration::from_millis(10));
    }
}

/// Entry point for problem demonstration
pub fn e1508_entry() -> Result<(), Box<dyn std::error::Error>> {
    Ok(())
}

// ============================================================================
// GOOD ALTERNATIVES
// ============================================================================

/// GOOD: Use thread::join to wait for completion
pub fn e1508_good_join() -> i32 {
    let handle = thread::spawn(|| {
        thread::sleep(Duration::from_millis(10));
        42
    });

    handle.join().unwrap() // Waits until thread completes
}

/// GOOD: Use channels for communication
pub fn e1508_good_channel() -> i32 {
    let (tx, rx) = mpsc::channel();

    thread::spawn(move || {
        thread::sleep(Duration::from_millis(10));
        tx.send(42).unwrap();
    });

    rx.recv().unwrap() // Blocks until message received
}

/// GOOD: Use condition variable for complex waiting
pub fn e1508_good_condvar() -> i32 {
    let pair = Arc::new((Mutex::new(None::<i32>), Condvar::new()));
    let pair_clone = Arc::clone(&pair);

    thread::spawn(move || {
        let (lock, cvar) = &*pair_clone;
        thread::sleep(Duration::from_millis(10));

        let mut value = lock.lock().unwrap();
        *value = Some(42);
        cvar.notify_one();
    });

    let (lock, cvar) = &*pair;
    let mut value = lock.lock().unwrap();

    while value.is_none() {
        value = cvar.wait(value).unwrap();
    }

    value.unwrap()
}

/// GOOD: Use barrier for synchronizing multiple threads
pub fn e1508_good_barrier() {
    use std::sync::Barrier;

    let barrier = Arc::new(Barrier::new(3));
    let mut handles = vec![];

    for i in 0..3 {
        let barrier = Arc::clone(&barrier);
        let handle = thread::spawn(move || {
            println!("Thread {} before barrier", i);
            barrier.wait(); // All threads wait here
            println!("Thread {} after barrier", i);
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }
}

/// GOOD: Use oneshot channel for single value
pub fn e1508_good_oneshot() -> i32 {
    let (tx, rx) = mpsc::sync_channel(1);

    thread::spawn(move || {
        thread::sleep(Duration::from_millis(10));
        tx.send(42).unwrap();
    });

    rx.recv().unwrap()
}

/// GOOD: Use recv_timeout for bounded waiting
pub fn e1508_good_timeout() -> Option<i32> {
    let (tx, rx) = mpsc::channel();

    thread::spawn(move || {
        thread::sleep(Duration::from_millis(10));
        let _ = tx.send(42);
    });

    rx.recv_timeout(Duration::from_millis(100)).ok()
}

/// GOOD: Scoped threads don't need explicit join
pub fn e1508_good_scoped() -> i32 {
    let result = Mutex::new(0);

    thread::scope(|s| {
        s.spawn(|| {
            thread::sleep(Duration::from_millis(10));
            *result.lock().unwrap() = 42;
        });
        // Thread automatically joined at end of scope
    });

    let value = *result.lock().unwrap();
    value
}

// ============================================================================
// GOOD EXAMPLES unit tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_join() {
        assert_eq!(e1508_good_join(), 42);
    }

    #[test]
    fn test_channel() {
        assert_eq!(e1508_good_channel(), 42);
    }

    #[test]
    fn test_condvar() {
        assert_eq!(e1508_good_condvar(), 42);
    }

    #[test]
    fn test_oneshot() {
        assert_eq!(e1508_good_oneshot(), 42);
    }

    #[test]
    fn test_timeout() {
        assert_eq!(e1508_good_timeout(), Some(42));
    }

    #[test]
    fn test_scoped() {
        assert_eq!(e1508_good_scoped(), 42);
    }
}
