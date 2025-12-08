/// E1501: Non-Send types across threads
/// Severity: HIGH
/// LLM confusion: 4 (HIGH)
///
/// Description: Some types in Rust are not safe to send between threads (not "Send"). This example
/// shows code that compiles but demonstrates the confusion around thread safety. The code uses
/// channels and Arc to work around the Send requirement, but the pattern is confusing because it's
/// not immediately obvious why we need Arc instead of Rc. For non-Rust developers, understanding
/// which types are Send and which aren't is very confusing. Fix by using thread-safe types (Arc,
/// Mutex) from the start and understanding the Send/Sync traits.
///
/// ## The Send Trait Problem
///
/// ```text
/// Rc<T>  → NOT Send (reference counting not atomic)
/// Arc<T> → Send (atomic reference counting)
///
/// Cell<T>    → NOT Sync (interior mutability not thread-safe)
/// Mutex<T>   → Sync (synchronized access)
///
/// *mut T → NOT Send/Sync (raw pointers are unsafe)
/// ```
///
/// ## Why This Matters
///
/// 1. **Compile errors**: Non-Send types can't be moved to threads
/// 2. **Data races**: Non-Sync types can cause data races if shared
/// 3. **LLM confusion**: LLMs often suggest Rc when Arc is needed
/// 4. **Subtle bugs**: Wrong type choice leads to hard-to-understand errors
///
/// ## The Right Solutions
///
/// ### Option 1: Use Arc instead of Rc
/// ```rust
/// use std::sync::Arc;
/// use std::thread;
///
/// let data = Arc::new(vec![1, 2, 3]);
/// let data_clone = Arc::clone(&data);
/// thread::spawn(move || {
///     println!("{:?}", data_clone);
/// });
/// ```
///
/// ### Option 2: Use Mutex for shared mutable state
/// ```rust
/// use std::sync::{Arc, Mutex};
///
/// let counter = Arc::new(Mutex::new(0));
/// let counter_clone = Arc::clone(&counter);
/// std::thread::spawn(move || {
///     *counter_clone.lock().unwrap() += 1;
/// });
/// ```
///
/// ### Option 3: Clone data instead of sharing
/// ```rust
/// let data = vec![1, 2, 3];
/// let data_clone = data.clone();  // Each thread owns its copy
/// std::thread::spawn(move || {
///     println!("{:?}", data_clone);
/// });
/// ```
///
/// Mitigation: Use `Arc` instead of `Rc` for multi-threaded code. The compiler will prevent this
/// error at compile time. Understand the Send and Sync traits - they mark types safe for threading.
/// Use thread-safe alternatives: Arc instead of Rc, Mutex/RwLock for shared mutable state.

use std::sync::{mpsc, Arc};
use std::thread;

// ============================================================================
// CONFUSING PATTERNS - UNDERSTAND WHY THESE ARE NEEDED
// ============================================================================

/// PROBLEM E1501: This pattern is confusing - why do we need Arc here?
pub fn e1501_bad_arc_confusion() {
    // PROBLEM E1501: This pattern is confusing - why do we need Arc here?
    // Non-Rust developers won't understand the Send requirement
    let data = Arc::new(vec![1, 2, 3, 4, 5]);
    let (tx, rx) = mpsc::channel();

    // Spawn multiple threads that share the data
    for i in 0..3 {
        let data_clone = Arc::clone(&data);
        let tx_clone = tx.clone();

        thread::spawn(move || {
            // PROBLEM E1501: The need for Arc instead of Rc is not obvious
            // LLMs often suggest Rc here, which won't compile
            let sum: i32 = data_clone.iter().sum();
            // PROBLEM E1002: direct unwrap/expect
            tx_clone.send((i, sum)).unwrap();
        });
    }

    drop(tx);

    // Collect results
    for (thread_id, sum) in rx {
        let _ = (thread_id, sum);
    }
}

// PROBLEM E1501: This struct is NOT Send because it contains Rc
// But this isn't obvious without understanding Send trait
use std::rc::Rc;

/// Struct containing Rc - NOT Send
pub struct NotSendData {
    // PROBLEM E1501: Rc is not Send - can't be sent across threads
    // But this compiles fine as long as we don't try to send it
    data: Rc<Vec<i32>>,
}

impl NotSendData {
    pub fn new(data: Vec<i32>) -> Self {
        NotSendData {
            data: Rc::new(data),
        }
    }

    pub fn get_sum(&self) -> i32 {
        self.data.iter().sum()
    }
}

/// This function compiles but demonstrates the confusion
pub fn e1501_bad_send_confusion() {
    let not_send = NotSendData::new(vec![1, 2, 3]);

    // PROBLEM E1501: Can't spawn thread with not_send because it's not Send
    // We have to work around it, which is confusing
    let sum = not_send.get_sum();

    // Instead of sending the struct, we send just the data
    thread::spawn(move || {
        println!("Sum: {}", sum);
    });
}

/// Entry point for problem demonstration
pub fn e1501_entry() -> Result<(), Box<dyn std::error::Error>> {
    e1501_bad_arc_confusion();
    Ok(())
}

// ============================================================================
// GOOD ALTERNATIVES
// ============================================================================

/// GOOD: Struct designed for thread safety
pub struct SendSafeData {
    data: Arc<Vec<i32>>,
}

impl SendSafeData {
    pub fn new(data: Vec<i32>) -> Self {
        SendSafeData {
            data: Arc::new(data),
        }
    }

    pub fn get_sum(&self) -> i32 {
        self.data.iter().sum()
    }

    pub fn clone_for_thread(&self) -> Self {
        SendSafeData {
            data: Arc::clone(&self.data),
        }
    }
}

/// GOOD: Clear thread-safe pattern
pub fn e1501_good_arc_pattern() {
    let data = Arc::new(vec![1, 2, 3, 4, 5]);
    let mut handles = vec![];

    for i in 0..3 {
        let data_clone = Arc::clone(&data);
        let handle = thread::spawn(move || {
            let sum: i32 = data_clone.iter().sum();
            (i, sum)
        });
        handles.push(handle);
    }

    for handle in handles {
        let (id, sum) = handle.join().expect("Thread panicked");
        println!("Thread {}: sum = {}", id, sum);
    }
}

/// GOOD: Use scoped threads when data doesn't need to outlive threads
pub fn e1501_good_scoped_threads() {
    let data = [1, 2, 3, 4, 5];

    thread::scope(|s| {
        // Spawn threads that can borrow data
        s.spawn(|| {
            let sum: i32 = data.iter().sum();
            println!("Thread 0: sum = {}", sum);
        });
        s.spawn(|| {
            let sum: i32 = data.iter().sum();
            println!("Thread 1: sum = {}", sum);
        });
        s.spawn(|| {
            let sum: i32 = data.iter().sum();
            println!("Thread 2: sum = {}", sum);
        });
    });
}

/// GOOD: Clone data when each thread needs its own copy
pub fn e1501_good_clone_pattern() {
    let original_data = vec![1, 2, 3, 4, 5];

    for i in 0..3 {
        let data_copy = original_data.clone();
        thread::spawn(move || {
            // Each thread owns its own copy
            let sum: i32 = data_copy.iter().sum();
            println!("Thread {}: sum = {}", i, sum);
        });
    }
}

/// GOOD: Use channels for communication
pub fn e1501_good_channel_pattern() -> Vec<i32> {
    let (tx, rx) = mpsc::channel();

    for i in 0..3 {
        let tx = tx.clone();
        thread::spawn(move || {
            let result = i * 10;
            tx.send(result).expect("Receiver dropped");
        });
    }

    drop(tx); // Close sender so receiver knows when done

    rx.into_iter().collect()
}

// ============================================================================
// GOOD EXAMPLES unit tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_send_safe_data() {
        let data = SendSafeData::new(vec![1, 2, 3]);
        assert_eq!(data.get_sum(), 6);

        let cloned = data.clone_for_thread();
        let handle = thread::spawn(move || cloned.get_sum());
        assert_eq!(handle.join().unwrap(), 6);
    }

    #[test]
    fn test_channel_pattern() {
        let results = e1501_good_channel_pattern();
        assert_eq!(results.len(), 3);
        assert!(results.contains(&0));
        assert!(results.contains(&10));
        assert!(results.contains(&20));
    }
}
