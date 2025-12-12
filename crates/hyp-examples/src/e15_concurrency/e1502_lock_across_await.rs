/// E1502: Lock held across await
/// Severity: HIGH
/// LLM confusion: 4 (HIGH)
///
/// Description: In async code, holding a lock (Mutex guard) across an `.await` point is dangerous
/// because the task might be moved to a different thread while waiting. This can cause deadlocks
/// or prevent other tasks from accessing the locked data. Fix by dropping the lock before awaiting,
/// or use async-aware locks like `tokio::sync::Mutex`.
///
/// ## The Async Lock Problem
///
/// ```text
/// async fn bad() {
///     let guard = mutex.lock().unwrap();  // Lock acquired
///     some_async_op().await;               // Task suspended here
///     // Guard is still held! Other tasks can't access mutex
///     *guard += 1;
/// }
/// ```
///
/// ## Why This Matters
///
/// 1. **Deadlocks**: Other tasks waiting for the lock block forever
/// 2. **Thread starvation**: Lock held across thread boundaries
/// 3. **Performance**: Locks held much longer than necessary
/// 4. **Non-Send futures**: std::sync::MutexGuard is not Send
///
/// ## The Right Solutions
///
/// ### Option 1: Drop lock before await
/// ```rust,no_run
/// async fn good() {
///     let value = {
///         let guard = mutex.lock().unwrap();
///         *guard  // Copy value
///     };  // Guard dropped here
///     some_async_op().await;  // Safe!
/// }
/// ```
///
/// ### Option 2: Use tokio::sync::Mutex
/// ```rust,no_run
/// use tokio::sync::Mutex;
///
/// async fn good() {
///     let guard = mutex.lock().await;  // Async-aware lock
///     some_async_op().await;            // Safe with tokio Mutex
///     *guard += 1;
/// }
/// ```
///
/// ### Option 3: Restructure to minimize lock scope
/// ```rust,no_run
/// async fn good() {
///     let data = get_data_from_mutex();
///     let result = process_async(data).await;
///     store_result_in_mutex(result);
/// }
/// ```
///
/// Mitigation: Use async-aware locks like `tokio::sync::Mutex` or `async_std::sync::Mutex`. Always
/// drop lock guards before `.await` points. Use `#![warn(clippy::await_holding_lock)]` to detect
/// this pattern. Restructure code to minimize lock scope.

use std::sync::{Arc, Mutex};

// ============================================================================
// DANGEROUS PATTERNS - NEVER DO THIS
// ============================================================================

/// PROBLEM E1502: Holding lock across await point can cause deadlocks
#[allow(clippy::await_holding_lock)]
pub async fn e1502_bad_lock_across_await(mutex: Arc<Mutex<i32>>) {
    // PROBLEM E1002: direct unwrap/expect
    let mut guard = mutex.lock().unwrap();
    *guard += 1;

    // PROBLEM E1502: Holding lock across await point can cause deadlocks
    e1502_some_async_operation().await;

    *guard += 1;
}

/// PROBLEM E1502: Lock held in async loop
#[allow(clippy::await_holding_lock)]
pub async fn e1502_bad_lock_in_loop(mutex: Arc<Mutex<Vec<i32>>>) {
    let mut guard = mutex.lock().unwrap();

    for i in 0..5 {
        guard.push(i);
        // PROBLEM E1502: Lock held across multiple awaits
        e1502_some_async_operation().await;
    }
}

async fn e1502_some_async_operation() {
    tokio::time::sleep(std::time::Duration::from_millis(10)).await;
}

/// Entry point for problem demonstration
pub fn e1502_entry() -> Result<(), Box<dyn std::error::Error>> {
    Ok(())
}

// ============================================================================
// GOOD ALTERNATIVES
// ============================================================================

/// GOOD: Drop lock before await
pub async fn e1502_good_drop_before_await(mutex: Arc<Mutex<i32>>) {
    // Scope the lock acquisition
    {
        let mut guard = mutex.lock().unwrap();
        *guard += 1;
    } // Guard dropped here

    // Now safe to await
    e1502_some_async_operation().await;

    // Reacquire if needed
    {
        let mut guard = mutex.lock().unwrap();
        *guard += 1;
    }
}

/// GOOD: Extract value, release lock, then await
pub async fn e1502_good_extract_value(mutex: Arc<Mutex<i32>>) -> i32 {
    let current = {
        let guard = mutex.lock().unwrap();
        *guard // Copy the value
    }; // Guard dropped

    e1502_some_async_operation().await;

    current + 1
}

/// GOOD: Use tokio::sync::Mutex for async code
pub async fn e1502_good_tokio_mutex(mutex: Arc<tokio::sync::Mutex<i32>>) {
    let mut guard = mutex.lock().await; // Async-aware lock
    *guard += 1;

    // Safe to await with tokio Mutex
    e1502_some_async_operation().await;

    *guard += 1;
}

/// GOOD: Minimize lock scope in loops
pub async fn e1502_good_minimal_scope(mutex: Arc<Mutex<Vec<i32>>>) {
    for i in 0..5 {
        // Lock only for the push operation
        {
            let mut guard = mutex.lock().unwrap();
            guard.push(i);
        } // Released before await

        e1502_some_async_operation().await;
    }
}

/// GOOD: Use RwLock for read-heavy workloads
pub async fn e1502_good_rwlock(rwlock: Arc<tokio::sync::RwLock<i32>>) -> i32 {
    // Multiple readers can proceed concurrently
    let value = *rwlock.read().await;

    e1502_some_async_operation().await;

    value
}

/// GOOD: Use try_lock for non-blocking attempt
pub async fn e1502_good_try_lock(mutex: Arc<Mutex<i32>>) -> Option<i32> {
    match mutex.try_lock() {
        Ok(guard) => Some(*guard),
        Err(_) => {
            // Lock is held, do something else
            e1502_some_async_operation().await;
            None
        }
    }
}

// ============================================================================
// GOOD EXAMPLES unit tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_good_drop_before_await() {
        let mutex = Arc::new(Mutex::new(0));
        e1502_good_drop_before_await(mutex.clone()).await;
        assert_eq!(*mutex.lock().unwrap(), 2);
    }

    #[tokio::test]
    async fn test_good_extract_value() {
        let mutex = Arc::new(Mutex::new(41));
        let result = e1502_good_extract_value(mutex).await;
        assert_eq!(result, 42);
    }

    #[tokio::test]
    async fn test_good_tokio_mutex() {
        let mutex = Arc::new(tokio::sync::Mutex::new(0));
        e1502_good_tokio_mutex(mutex.clone()).await;
        assert_eq!(*mutex.lock().await, 2);
    }

    #[tokio::test]
    async fn test_good_minimal_scope() {
        let mutex = Arc::new(Mutex::new(Vec::new()));
        e1502_good_minimal_scope(mutex.clone()).await;
        assert_eq!(*mutex.lock().unwrap(), vec![0, 1, 2, 3, 4]);
    }
}
