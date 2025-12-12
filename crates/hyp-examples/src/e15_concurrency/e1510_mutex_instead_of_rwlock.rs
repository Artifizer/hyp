/// E1510: Mutex instead of RwLock
/// Severity: LOW
/// LLM confusion: 2 (LOW)
///
/// Description: Mutex allows only one reader OR one writer at a time. RwLock allows multiple
/// simultaneous readers but only one writer. For read-heavy workloads, using Mutex is inefficient
/// because readers block each other unnecessarily. Fix by using RwLock when you have many reads
/// and few writes.
///
/// ## The Contention Problem
///
/// ```text
/// Mutex: Readers block each other
/// Thread 1: lock() → reading...
/// Thread 2: lock() → BLOCKED (waiting for Thread 1)
/// Thread 3: lock() → BLOCKED (waiting for Thread 1)
///
/// RwLock: Readers can proceed in parallel
/// Thread 1: read() → reading...
/// Thread 2: read() → reading... (parallel!)
/// Thread 3: read() → reading... (parallel!)
/// ```
///
/// ## Why This Matters
///
/// 1. **Performance**: Readers unnecessarily block each other
/// 2. **Scalability**: Contention increases with thread count
/// 3. **Throughput**: Lower overall throughput for read-heavy workloads
/// 4. **Latency**: Higher latency for read operations
///
/// ## The Right Solutions
///
/// ### Option 1: Use RwLock for read-heavy workloads
/// ```rust
/// use std::sync::RwLock;
///
/// let data = RwLock::new(vec![1, 2, 3]);
/// let guard = data.read().unwrap();  // Multiple readers OK
/// ```
///
/// ### Option 2: Keep Mutex for write-heavy workloads
/// ```rust
/// use std::sync::Mutex;
///
/// let data = Mutex::new(0);
/// // When writes are frequent, Mutex is simpler and faster
/// ```
///
/// ### Option 3: Use atomic types for simple values
/// ```rust
/// use std::sync::atomic::AtomicI32;
///
/// let counter = AtomicI32::new(0);
/// // No locking needed at all!
/// ```
///
/// Mitigation: Use `RwLock` for read-heavy workloads - it allows multiple concurrent readers.
/// Use `Mutex` for write-heavy workloads or when simplicity is preferred. Profile to verify that
/// RwLock improves performance - it has higher overhead than Mutex for writes.

use std::sync::{Arc, Mutex, RwLock};
use std::thread;
use std::time::{Duration, Instant};

// ============================================================================
// SUBOPTIMAL PATTERNS
// ============================================================================

/// PROBLEM E1510: Using Mutex for read-heavy workload
pub fn e1510_bad_mutex_for_reads() {
    let data = Arc::new(Mutex::new(vec![1, 2, 3]));

    // PROBLEM E1510: Using Mutex for read-heavy workload (RwLock would be better)
    for _ in 0..10 {
        let data_clone = data.clone();
        thread::spawn(move || {
            // PROBLEM E1002: direct unwrap/expect
            let guard = data_clone.lock().unwrap();
            let _sum: i32 = guard.iter().sum();
            // All readers block each other!
        });
    }
}

/// Entry point for problem demonstration
pub fn e1510_entry() -> Result<(), Box<dyn std::error::Error>> {
    Ok(())
}

// ============================================================================
// GOOD ALTERNATIVES
// ============================================================================

/// GOOD: Use RwLock for read-heavy workload
pub fn e1510_good_rwlock_for_reads() {
    let data = Arc::new(RwLock::new(vec![1, 2, 3]));
    let mut handles = vec![];

    // Multiple readers can proceed in parallel
    for _ in 0..10 {
        let data = Arc::clone(&data);
        let handle = thread::spawn(move || {
            let guard = data.read().unwrap();
            let _sum: i32 = guard.iter().sum();
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }
}

/// GOOD: Use Mutex when writes are frequent
pub fn e1510_good_mutex_for_writes() {
    let counter = Arc::new(Mutex::new(0));
    let mut handles = vec![];

    // When mostly writing, Mutex is simpler and faster
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

/// GOOD: Mixed read/write with RwLock
pub fn e1510_good_mixed_workload() {
    let data = Arc::new(RwLock::new(vec![1, 2, 3]));
    let mut handles = vec![];

    // Readers (majority)
    for i in 0..8 {
        let data = Arc::clone(&data);
        let handle = thread::spawn(move || {
            let guard = data.read().unwrap();
            println!("Reader {}: sum = {}", i, guard.iter().sum::<i32>());
        });
        handles.push(handle);
    }

    // Writers (minority)
    for i in 0..2 {
        let data = Arc::clone(&data);
        let handle = thread::spawn(move || {
            let mut guard = data.write().unwrap();
            guard.push(i + 10);
            println!("Writer {}: added {}", i, i + 10);
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }
}

/// GOOD: Benchmark to choose between Mutex and RwLock
pub fn e1510_good_benchmark() -> (Duration, Duration) {
    const ITERATIONS: usize = 1000;
    const READERS: usize = 10;

    // Benchmark Mutex
    let mutex_data = Arc::new(Mutex::new(vec![1, 2, 3, 4, 5]));
    let start = Instant::now();
    let mut handles = vec![];

    for _ in 0..READERS {
        let data = Arc::clone(&mutex_data);
        let handle = thread::spawn(move || {
            for _ in 0..ITERATIONS {
                let guard = data.lock().unwrap();
                let _: i32 = guard.iter().sum();
            }
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }
    let mutex_time = start.elapsed();

    // Benchmark RwLock
    let rwlock_data = Arc::new(RwLock::new(vec![1, 2, 3, 4, 5]));
    let start = Instant::now();
    let mut handles = vec![];

    for _ in 0..READERS {
        let data = Arc::clone(&rwlock_data);
        let handle = thread::spawn(move || {
            for _ in 0..ITERATIONS {
                let guard = data.read().unwrap();
                let _: i32 = guard.iter().sum();
            }
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }
    let rwlock_time = start.elapsed();

    (mutex_time, rwlock_time)
}

/// GOOD: Use atomics for simple read-heavy counters
pub fn e1510_good_atomic_for_reads() {
    use std::sync::atomic::{AtomicI32, Ordering};

    let counter = Arc::new(AtomicI32::new(42));
    let mut handles = vec![];

    // Readers - no locking at all!
    for _ in 0..10 {
        let counter = Arc::clone(&counter);
        let handle = thread::spawn(move || {
            let value = counter.load(Ordering::SeqCst);
            println!("Read: {}", value);
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }
}

/// Decision helper: when to use which
pub fn e1510_decision_guide() {
    println!("Use Mutex when:");
    println!("  - Writes are frequent (> 20% of operations)");
    println!("  - Lock hold times are very short");
    println!("  - Simplicity is preferred over performance");
    println!();
    println!("Use RwLock when:");
    println!("  - Reads vastly outnumber writes (> 80% reads)");
    println!("  - Multiple readers need concurrent access");
    println!("  - Read operations take significant time");
    println!();
    println!("Use Atomics when:");
    println!("  - Data is a simple value (integer, bool, pointer)");
    println!("  - Operations are simple (load, store, fetch_add)");
    println!("  - Maximum performance is needed");
}

// ============================================================================
// GOOD EXAMPLES unit tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rwlock_for_reads() {
        e1510_good_rwlock_for_reads();
    }

    #[test]
    fn test_mutex_for_writes() {
        e1510_good_mutex_for_writes();
    }

    #[test]
    fn test_benchmark() {
        let (mutex_time, rwlock_time) = e1510_good_benchmark();
        println!("Mutex: {:?}, RwLock: {:?}", mutex_time, rwlock_time);
        // RwLock should generally be faster for read-heavy workloads
    }
}
