/// E1512: Prohibit std::thread::spawn in async codebases
/// Severity: HIGH
/// LLM confusion: 3 (MEDIUM) - LLMs often suggest std::thread::spawn for blocking work
///
/// Description: In tokio-based microservices, using std::thread::spawn instead of
/// tokio::task::spawn_blocking breaks the async runtime's cooperative scheduling model.
/// This causes thread pool exhaustion, context loss, and poor observability.
///
/// Why this matters for microservices:
/// 1. Thread Pool Exhaustion - OS threads outside tokio's control
/// 2. Context Loss - Spawned threads can't use tokio primitives
/// 3. Poor Observability - External threads bypass tracing integration
/// 4. Resource Waste - Duplicates tokio's blocking pool
/// 5. Graceful Shutdown - Can't await OS threads for clean shutdown
///
/// Mitigation: Use tokio::task::spawn_blocking for blocking operations.
use std::thread;

// ============================================================================
// BAD EXAMPLES - std::thread::spawn in async codebase
// ============================================================================

/// PROBLEM E1512: Using std::thread::spawn for blocking I/O
pub fn e1512_bad_direct_thread_spawn() {
    // This creates an OS thread outside tokio's control
    std::thread::spawn(|| {
        // Blocking I/O operation
        std::fs::read_to_string("/etc/hosts").ok();
    });
}

/// PROBLEM E1512: Using imported thread::spawn
pub fn e1512_bad_imported_thread_spawn() {
    // Same problem, just with an import
    thread::spawn(|| {
        expensive_blocking_computation();
    });
}

/// PROBLEM E1512: Multiple spawns in a function
pub fn e1512_bad_multiple_spawns() {
    std::thread::spawn(|| {
        work1();
    });
    std::thread::spawn(|| {
        work2();
    });
    std::thread::spawn(|| {
        work3();
    });
}

/// PROBLEM E1512: Trying to use tokio inside std::thread
pub fn e1512_bad_tokio_context_loss() {
    std::thread::spawn(|| {
        // PANIC! This will fail because we're outside tokio's runtime
        // tokio::time::sleep(std::time::Duration::from_secs(1)).await;

        // Even trying to spawn async tasks fails
        // tokio::spawn(async { ... }); // Runtime not available!
    });
}

fn expensive_blocking_computation() {
    thread::sleep(std::time::Duration::from_millis(100));
}

fn work1() {}
fn work2() {}
fn work3() {}

pub fn e1512_entry() -> Result<(), Box<dyn std::error::Error>> {
    e1512_bad_direct_thread_spawn();
    Ok(())
}

// ============================================================================
// GOOD EXAMPLES - Using tokio::task::spawn_blocking
// ============================================================================

/// GOOD: Use tokio::task::spawn_blocking for blocking operations
pub async fn e1512_good_spawn_blocking() {
    // This schedules blocking work on tokio's blocking thread pool
    let result = tokio::task::spawn_blocking(|| {
        // Blocking I/O is fine here
        std::fs::read_to_string("/etc/hosts")
    })
    .await
    .unwrap();

    println!("Read {} bytes", result.unwrap_or_default().len());
}

/// GOOD: spawn_blocking preserves tokio context for the caller
pub async fn e1512_good_preserves_context() {
    // Before spawn_blocking, we're in async context
    tokio::time::sleep(std::time::Duration::from_millis(10)).await;

    // spawn_blocking for CPU-intensive work
    let computed = tokio::task::spawn_blocking(|| expensive_cpu_computation())
        .await
        .unwrap();

    // After spawn_blocking, we're still in async context
    tokio::time::sleep(std::time::Duration::from_millis(10)).await;

    println!("Computed: {}", computed);
}

/// GOOD: Using tokio::spawn for async work (not blocking)
pub async fn e1512_good_tokio_spawn_for_async() {
    // tokio::spawn is correct for async tasks
    tokio::spawn(async {
        // This is async work, not blocking
        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
        println!("Async task completed");
    });
}

/// GOOD: Proper blocking pool for parallel I/O
pub async fn e1512_good_parallel_blocking_io(paths: Vec<String>) -> Vec<String> {
    let handles: Vec<_> = paths
        .into_iter()
        .map(|path| {
            tokio::task::spawn_blocking(move || std::fs::read_to_string(&path).unwrap_or_default())
        })
        .collect();

    let mut results = Vec::new();
    for handle in handles {
        results.push(handle.await.unwrap());
    }
    results
}

fn expensive_cpu_computation() -> i64 {
    // Simulate CPU work
    (0..1_000_000).sum()
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    #[tokio::test]
    async fn test_spawn_blocking_works() {
        let result = tokio::task::spawn_blocking(|| 42).await;
        assert_eq!(result.unwrap(), 42);
    }
}
