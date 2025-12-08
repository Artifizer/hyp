/// E1505: Non-Send future
/// Severity: HIGH
/// LLM confusion: 4 (HIGH)
///
/// Description: Async futures must be Send if they need to be moved between threads (which most
/// async runtimes do). Using non-Send types like Rc in an async function makes the entire future
/// non-Send, preventing it from being used with multi-threaded executors. Fix by using Send
/// alternatives like Arc, or ensure the non-Send value is dropped before await points.
///
/// ## The Non-Send Future Problem
///
/// ```text
/// async fn bad() {
///     let rc = Rc::new(42);        // Rc is not Send
///     some_async_op().await;        // Future suspended here
///     println!("{}", rc);           // rc held across await
/// }
/// // This future is NOT Send - can't spawn on multi-threaded runtime
/// ```
///
/// ## Why This Matters
///
/// 1. **Can't spawn**: tokio::spawn requires Send futures
/// 2. **Runtime limitations**: Can only use current_thread runtime
/// 3. **Composition issues**: One non-Send type infects the whole future
/// 4. **Confusing errors**: Compiler errors about Send are cryptic
///
/// ## The Right Solutions
///
/// ### Option 1: Use Arc instead of Rc
/// ```rust,no_run
/// use std::sync::Arc;
///
/// async fn good() {
///     let arc = Arc::new(42);       // Arc is Send
///     some_async_op().await;
///     println!("{}", arc);          // Safe!
/// }
/// ```
///
/// ### Option 2: Drop non-Send before await
/// ```rust,no_run
/// use std::rc::Rc;
///
/// async fn good() {
///     let value = {
///         let rc = Rc::new(42);
///         *rc  // Extract value
///     };  // rc dropped here
///     some_async_op().await;        // Safe - no Rc held
///     println!("{}", value);
/// }
/// ```
///
/// ### Option 3: Use spawn_local for non-Send futures
/// ```rust,no_run
/// use tokio::task::spawn_local;
///
/// // Must be in a LocalSet context
/// spawn_local(async {
///     let rc = std::rc::Rc::new(42);
///     some_async_op().await;
///     println!("{}", rc);
/// });
/// ```
///
/// Mitigation: Use `Arc` instead of `Rc` in async code. Drop non-Send values before `.await`
/// points. Use `#![warn(clippy::future_not_send)]` to detect non-Send futures. Test that your
/// futures are Send by spawning them on multi-threaded runtimes.

use std::rc::Rc;
use std::sync::Arc;

// ============================================================================
// PROBLEMATIC PATTERNS
// ============================================================================

/// PROBLEM E1505: Future is not Send due to Rc
pub async fn e1505_bad_rc_across_await() {
    let data = Rc::new(42);

    // PROBLEM E1505: Future is not Send due to Rc
    e1505_some_async_operation().await;
    println!("{}", data);
}

/// PROBLEM E1505: RefCell makes future non-Send
pub async fn e1505_bad_refcell_across_await() {
    use std::cell::RefCell;

    let data = Rc::new(RefCell::new(vec![1, 2, 3]));

    e1505_some_async_operation().await;

    data.borrow_mut().push(4);
}

async fn e1505_some_async_operation() {
    tokio::time::sleep(std::time::Duration::from_millis(10)).await;
}

/// Entry point for problem demonstration
pub fn e1505_entry() -> Result<(), Box<dyn std::error::Error>> {
    Ok(())
}

// ============================================================================
// GOOD ALTERNATIVES
// ============================================================================

/// GOOD: Use Arc instead of Rc
pub async fn e1505_good_arc() {
    let data = Arc::new(42);

    e1505_some_async_operation().await;

    println!("{}", data);
}

/// GOOD: Drop Rc before await
pub async fn e1505_good_drop_before_await() {
    let value = {
        let rc = Rc::new(42);
        *rc // Extract the value
    }; // rc dropped here

    e1505_some_async_operation().await;

    println!("{}", value);
}

/// GOOD: Use Arc<Mutex> for shared mutable state
pub async fn e1505_good_arc_mutex() {
    use std::sync::Mutex;

    let data = Arc::new(Mutex::new(vec![1, 2, 3]));

    e1505_some_async_operation().await;

    data.lock().unwrap().push(4);
}

/// GOOD: Use tokio's Mutex for async-friendly locking
pub async fn e1505_good_tokio_mutex() {
    let data = Arc::new(tokio::sync::Mutex::new(vec![1, 2, 3]));

    e1505_some_async_operation().await;

    data.lock().await.push(4);
}

/// GOOD: Clone Arc for spawned tasks
pub async fn e1505_good_spawn_with_arc() {
    let data = Arc::new(42);
    let data_clone = Arc::clone(&data);

    let handle = tokio::spawn(async move {
        e1505_some_async_operation().await;
        *data_clone
    });

    let result = handle.await.unwrap();
    println!("Result: {}, Original: {}", result, data);
}

/// Helper to verify a future is Send
fn _assert_send<T: Send>(_: T) {}

/// GOOD: Verify your future is Send at compile time
pub async fn e1505_good_verified_send() {
    let data = Arc::new(42);
    e1505_some_async_operation().await;
    println!("{}", data);
}

// This would fail to compile if the future wasn't Send
fn _verify_send() {
    _assert_send(e1505_good_verified_send());
}

/// GOOD: Use channels for communication instead of shared state
pub async fn e1505_good_channel_pattern() -> i32 {
    let (tx, rx) = tokio::sync::oneshot::channel();

    tokio::spawn(async move {
        e1505_some_async_operation().await;
        let _ = tx.send(42);
    });

    rx.await.unwrap()
}

// ============================================================================
// GOOD EXAMPLES unit tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_good_arc() {
        e1505_good_arc().await;
    }

    #[tokio::test]
    async fn test_good_arc_mutex() {
        e1505_good_arc_mutex().await;
    }

    #[tokio::test]
    async fn test_good_spawn_with_arc() {
        e1505_good_spawn_with_arc().await;
    }

    #[tokio::test]
    async fn test_good_channel_pattern() {
        let result = e1505_good_channel_pattern().await;
        assert_eq!(result, 42);
    }

    // Compile-time test: these functions should be callable from tokio::spawn
    #[tokio::test]
    async fn test_futures_are_send() {
        let h1 = tokio::spawn(e1505_good_arc());
        let h2 = tokio::spawn(e1505_good_arc_mutex());
        let h3 = tokio::spawn(e1505_good_tokio_mutex());

        h1.await.unwrap();
        h2.await.unwrap();
        h3.await.unwrap();
    }
}
