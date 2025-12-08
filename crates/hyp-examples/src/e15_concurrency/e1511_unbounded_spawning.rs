/// E1511: Unbounded task/thread spawning in loops
/// Severity: HIGH
/// LLM confusion: 2 (LOW)
///
/// Description: Spawning threads or async tasks in loops without limits leads to
/// resource exhaustion and potential denial of service. If the input is large or
/// unbounded, this can spawn millions of threads/tasks, exhausting system resources.
///
/// Mitigation: Use bounded concurrency with Semaphore, thread pools, or
/// stream::buffer_unordered(). Process in batches or use work-stealing queues.

use std::sync::Arc;
use std::thread;

/// PROBLEM E1511: Unbounded thread spawning in for loop
pub fn e1511_bad_unbounded_thread_spawn(items: Vec<i32>) {
    for item in items {
        // If items has 1 million elements, this spawns 1 million threads!
        thread::spawn(move || {
            e1511_bad_expensive_computation(item);
        });
    }
}

/// PROBLEM E1511: Spawning in while loop with unknown bounds
pub fn e1511_bad_unbounded_while_spawn(receiver: std::sync::mpsc::Receiver<i32>) {
    while let Ok(item) = receiver.recv() {
        // Spawns unbounded threads based on message rate
        thread::spawn(move || {
            e1511_bad_expensive_computation(item);
        });
    }
}

fn e1511_bad_expensive_computation(item: i32) -> i32 {
    // Simulate work
    thread::sleep(std::time::Duration::from_millis(10));
    item * 2
}

pub fn e1511_entry() -> Result<(), Box<dyn std::error::Error>> {
    e1511_bad_unbounded_thread_spawn(vec![1, 2, 3]);
    Ok(())
}

// ============================================================================
// GOOD EXAMPLES - Bounded concurrency
// ============================================================================

use std::sync::mpsc;

fn e1511_good_expensive_computation(item: i32) -> i32 {
    // Simulate work
    thread::sleep(std::time::Duration::from_millis(10));
    item * 2
}

/// GOOD: Use a thread pool with bounded workers
pub fn e1511_good_threadpool(items: Vec<i32>) {
    let num_workers = num_cpus();
    let (tx, rx) = mpsc::channel();
    let rx = Arc::new(std::sync::Mutex::new(rx));

    // Fixed number of worker threads
    let handles: Vec<_> = (0..num_workers)
        .map(|_| {
            let rx = Arc::clone(&rx);
            thread::spawn(move || {
                while let Ok(item) = rx.lock().unwrap().recv() {
                    e1511_good_expensive_computation(item);
                }
            })
        })
        .collect();

    // Send work to the pool
    for item in items {
        tx.send(item).unwrap();
    }
    drop(tx); // Signal workers to exit

    // Wait for completion
    for handle in handles {
        handle.join().unwrap();
    }
}

fn num_cpus() -> usize {
    4 // Simplified
}

/// GOOD: Process in batches with join
pub fn e1511_good_batched(items: Vec<i32>, batch_size: usize) {
    for batch in items.chunks(batch_size) {
        let handles: Vec<_> = batch
            .iter()
            .map(|&item| {
                thread::spawn(move || {
                    e1511_good_expensive_computation(item);
                })
            })
            .collect();

        // Wait for batch to complete before starting next
        for handle in handles {
            handle.join().unwrap();
        }
    }
}

/// GOOD: Use scoped threads (limited lifetime)
pub fn e1511_good_scoped(items: &[i32]) {
    thread::scope(|s| {
        // Spawn limited number of workers
        let chunk_size = (items.len() / num_cpus()).max(1);
        for chunk in items.chunks(chunk_size) {
            s.spawn(move || {
                for &item in chunk {
                    e1511_good_expensive_computation(item);
                }
            });
        }
    });
}

/// GOOD: Sequential processing for small workloads
pub fn e1511_good_sequential(items: Vec<i32>) -> Vec<i32> {
    items
        .into_iter()
        .map(e1511_good_expensive_computation)
        .collect()
}

// ============================================================================
// GOOD EXAMPLES unit tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_batched_processing() {
        let items: Vec<i32> = (0..100).collect();
        e1511_good_batched(items, 10);
    }

    #[test]
    fn test_scoped_threads() {
        let items: Vec<i32> = (0..100).collect();
        e1511_good_scoped(&items);
    }
}
