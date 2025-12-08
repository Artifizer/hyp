/// E1509: Channel lifetime issues
/// Severity: MED
/// LLM confusion: 3 (MED)
///
/// Description: Channels have a sender and receiver. If the sender is dropped before the receiver
/// tries to receive, the receiver gets a disconnection error. This code spawns a thread that
/// sends a message and immediately exits (dropping the sender), which might happen before the
/// receiver tries to receive. Fix by keeping the sender alive until all messages are sent, or
/// handle disconnection errors properly.
///
/// ## The Disconnection Problem
///
/// ```text
/// let (tx, rx) = mpsc::channel();
/// thread::spawn(move || {
///     tx.send(42).unwrap();
/// });  // tx dropped when thread exits
///
/// // If thread exits before we call recv()...
/// rx.recv()  // RecvError: channel disconnected
/// ```
///
/// ## Why This Matters
///
/// 1. **Lost messages**: Messages sent after disconnect are lost
/// 2. **Unexpected errors**: recv() returns error instead of blocking
/// 3. **Race conditions**: Depends on timing
/// 4. **Resource leaks**: Pending messages may be dropped
///
/// ## The Right Solutions
///
/// ### Option 1: Join thread before receiving
/// ```rust
/// let handle = std::thread::spawn(move || { tx.send(42).unwrap(); });
/// handle.join().unwrap();
/// let value = rx.recv().unwrap();
/// ```
///
/// ### Option 2: Handle disconnection gracefully
/// ```rust
/// match rx.recv() {
///     Ok(value) => println!("Got: {}", value),
///     Err(_) => println!("Channel disconnected"),
/// }
/// ```
///
/// ### Option 3: Use sync_channel for backpressure
/// ```rust
/// let (tx, rx) = std::sync::mpsc::sync_channel(1);
/// // Sender blocks if channel is full
/// ```
///
/// Mitigation: Keep senders alive until all messages are sent. Handle `RecvError` properly - it
/// indicates the sender was dropped. Use `sync_channel` for backpressure. Clone senders if
/// multiple threads need to send. Join threads before receiving if order matters.

use std::sync::mpsc;
use std::thread;
use std::time::Duration;

// ============================================================================
// PROBLEMATIC PATTERNS
// ============================================================================

/// PROBLEM E1509: Sender dropped too early
pub fn e1509_bad_early_drop() {
    let (tx, rx) = mpsc::channel();

    thread::spawn(move || {
        // PROBLEM E1002: direct unwrap/expect
        // PROBLEM E1509: Sender dropped too early, receiver will get disconnected error
        tx.send(42).unwrap();
    });

    // May fail if sender dropped before this runs
    let _value = rx.recv();
}

/// PROBLEM E1509: Not handling disconnection
pub fn e1509_bad_no_error_handling() -> i32 {
    let (tx, rx) = mpsc::channel::<i32>();

    thread::spawn(move || {
        // Sender might panic or exit early
        if false {
            tx.send(42).unwrap();
        }
        // tx dropped without sending!
    });

    // PROBLEM E1509: unwrap on potentially disconnected channel
    rx.recv().unwrap() // Will panic if sender dropped without sending
}

/// Entry point for problem demonstration
pub fn e1509_entry() -> Result<(), Box<dyn std::error::Error>> {
    Ok(())
}

// ============================================================================
// GOOD ALTERNATIVES
// ============================================================================

/// GOOD: Join thread before receiving
pub fn e1509_good_join_first() -> i32 {
    let (tx, rx) = mpsc::channel();

    let handle = thread::spawn(move || {
        tx.send(42).unwrap();
    });

    handle.join().unwrap(); // Wait for thread to complete
    rx.recv().unwrap() // Safe - we know message was sent
}

/// GOOD: Handle disconnection gracefully
pub fn e1509_good_handle_disconnect() -> Option<i32> {
    let (tx, rx) = mpsc::channel();

    thread::spawn(move || {
        thread::sleep(Duration::from_millis(10));
        let _ = tx.send(42); // Ignore send error
    });

    match rx.recv_timeout(Duration::from_millis(100)) {
        Ok(value) => Some(value),
        Err(mpsc::RecvTimeoutError::Timeout) => {
            eprintln!("Timeout waiting for message");
            None
        }
        Err(mpsc::RecvTimeoutError::Disconnected) => {
            eprintln!("Channel disconnected");
            None
        }
    }
}

/// GOOD: Use sync_channel for backpressure
pub fn e1509_good_sync_channel() -> Vec<i32> {
    let (tx, rx) = mpsc::sync_channel(2); // Buffer of 2

    let handle = thread::spawn(move || {
        for i in 0..5 {
            tx.send(i).unwrap(); // Blocks if buffer full
        }
    });

    let mut results = vec![];
    while let Ok(value) = rx.recv() {
        results.push(value);
    }

    handle.join().unwrap();
    results
}

/// GOOD: Keep sender alive with explicit drop
pub fn e1509_good_explicit_drop() -> Vec<i32> {
    let (tx, rx) = mpsc::channel();

    for i in 0..3 {
        let tx = tx.clone();
        thread::spawn(move || {
            tx.send(i).unwrap();
        });
    }

    drop(tx); // Drop original sender so rx knows when all are done

    rx.into_iter().collect()
}

/// GOOD: Use iter() for receiving all messages
pub fn e1509_good_iter() -> Vec<i32> {
    let (tx, rx) = mpsc::channel();

    let handle = thread::spawn(move || {
        for i in 0..5 {
            tx.send(i).unwrap();
        }
        // tx dropped here, signaling end
    });

    let results: Vec<i32> = rx.iter().collect();
    handle.join().unwrap();
    results
}

/// GOOD: Multiple senders with proper lifetime management
pub fn e1509_good_multiple_senders() -> Vec<i32> {
    let (tx, rx) = mpsc::channel();
    let mut handles = vec![];

    for i in 0..3 {
        let tx = tx.clone();
        let handle = thread::spawn(move || {
            thread::sleep(Duration::from_millis(i as u64 * 10));
            tx.send(i).unwrap();
        });
        handles.push(handle);
    }

    drop(tx); // Important: drop original sender

    // Wait for all threads
    for handle in handles {
        handle.join().unwrap();
    }

    rx.into_iter().collect()
}

/// GOOD: Bidirectional communication
pub fn e1509_good_bidirectional() -> i32 {
    let (request_tx, request_rx) = mpsc::channel();
    let (response_tx, response_rx) = mpsc::channel();

    let handle = thread::spawn(move || {
        let value: i32 = request_rx.recv().unwrap();
        response_tx.send(value * 2).unwrap();
    });

    request_tx.send(21).unwrap();
    let result = response_rx.recv().unwrap();

    handle.join().unwrap();
    result
}

// ============================================================================
// GOOD EXAMPLES unit tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_join_first() {
        assert_eq!(e1509_good_join_first(), 42);
    }

    #[test]
    fn test_handle_disconnect() {
        let result = e1509_good_handle_disconnect();
        assert_eq!(result, Some(42));
    }

    #[test]
    fn test_sync_channel() {
        let results = e1509_good_sync_channel();
        assert_eq!(results, vec![0, 1, 2, 3, 4]);
    }

    #[test]
    fn test_explicit_drop() {
        let mut results = e1509_good_explicit_drop();
        results.sort();
        assert_eq!(results, vec![0, 1, 2]);
    }

    #[test]
    fn test_iter() {
        let results = e1509_good_iter();
        assert_eq!(results, vec![0, 1, 2, 3, 4]);
    }

    #[test]
    fn test_bidirectional() {
        assert_eq!(e1509_good_bidirectional(), 42);
    }
}
