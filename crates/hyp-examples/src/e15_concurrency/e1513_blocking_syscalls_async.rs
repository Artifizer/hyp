/// E1513: Blocking syscalls inside async code
/// Severity: MEDIUM
/// LLM confusion: 3 (MEDIUM) - LLMs often suggest stdlib instead of async alternatives
///
/// Description: Blocking syscalls in async functions block the Tokio executor thread,
/// preventing other tasks from making progress. With a limited thread pool (typically
/// equal to CPU cores), even a single blocking call can cause cascading timeouts.
///
/// Mitigation: Use async alternatives:
/// - tokio::fs::* for file I/O
/// - tokio::net::* for networking
/// - tokio::time::sleep for sleeping
use std::time::Duration;

// ============================================================================
// BAD EXAMPLES - Blocking syscalls in async code
// ============================================================================

/// PROBLEM E1513: Using std::fs::read_to_string in async
pub async fn e1513_bad_fs_read() -> String {
    // This blocks the executor thread!
    std::fs::read_to_string("config.json").unwrap_or_default()
}

/// PROBLEM E1513: Using std::fs::write in async
pub async fn e1513_bad_fs_write(data: &[u8]) {
    // This blocks the executor thread!
    std::fs::write("output.txt", data).unwrap();
}

/// PROBLEM E1513: Using std::thread::sleep in async
pub async fn e1513_bad_thread_sleep() {
    // This blocks the executor thread for 1 second!
    // Other tasks waiting for this executor thread will stall.
    std::thread::sleep(Duration::from_secs(1));
}

/// PROBLEM E1513: Using std::net::TcpStream in async
pub async fn e1513_bad_tcp_connect() {
    // This blocks the executor thread during connection!
    use std::net::TcpStream;
    let _stream = TcpStream::connect("127.0.0.1:8080");
}

/// PROBLEM E1513: Multiple blocking calls cascade the problem
pub async fn e1513_bad_multiple_blocking() {
    let _data = std::fs::read("a.txt");
    let _content = std::fs::read_to_string("b.txt");
    std::fs::write("c.txt", "data").ok();
}

pub fn e1513_entry() -> Result<(), Box<dyn std::error::Error>> {
    // Note: These are intentionally not awaited for the example
    Ok(())
}

// ============================================================================
// GOOD EXAMPLES - Using async alternatives
// ============================================================================

/// GOOD: Use tokio::fs::read_to_string
pub async fn e1513_good_fs_read() -> String {
    // Non-blocking async file read
    tokio::fs::read_to_string("config.json")
        .await
        .unwrap_or_default()
}

/// GOOD: Use tokio::fs::write
pub async fn e1513_good_fs_write(data: &[u8]) {
    // Non-blocking async file write
    tokio::fs::write("output.txt", data).await.unwrap();
}

/// GOOD: Use tokio::time::sleep
pub async fn e1513_good_async_sleep() {
    // Non-blocking sleep - other tasks can run during this
    tokio::time::sleep(Duration::from_secs(1)).await;
}

/// GOOD: Use tokio::net::TcpStream
pub async fn e1513_good_tcp_connect() -> std::io::Result<()> {
    // Non-blocking async connection
    let _stream = tokio::net::TcpStream::connect("127.0.0.1:8080").await?;
    Ok(())
}

/// GOOD: For blocking operations that don't have async versions,
/// use spawn_blocking
pub async fn e1513_good_spawn_blocking() -> String {
    // Move blocking work to a dedicated blocking thread pool
    tokio::task::spawn_blocking(|| {
        // This DNS lookup is blocking but runs on the blocking pool
        std::net::ToSocketAddrs::to_socket_addrs(&"example.com:80")
            .map(|mut addrs| addrs.next().map(|a| a.to_string()).unwrap_or_default())
            .unwrap_or_default()
    })
    .await
    .unwrap_or_default()
}

/// GOOD: Use async file operations with proper error handling
pub async fn e1513_good_config_read() -> Result<String, Box<dyn std::error::Error>> {
    let content = tokio::fs::read_to_string("config.json").await?;
    Ok(content)
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_async_sleep() {
        tokio::time::sleep(Duration::from_millis(10)).await;
        // This works correctly without blocking
    }

    #[tokio::test]
    async fn test_spawn_blocking() {
        let result = tokio::task::spawn_blocking(|| 42).await;
        assert_eq!(result.unwrap(), 42);
    }
}
