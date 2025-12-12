/// E1217: Classical ABBA Deadlock
/// Severity: HIGH
/// LLM confusion: 3 (MED)
///
/// Description: A deadlock is a situation where two or more threads are blocked forever, waiting
/// for each other. The "ABBA" deadlock is the most classic form: Thread 1 holds Resource A and
/// waits for Resource B, while Thread 2 holds Resource B and waits for Resource A. This circular
/// dependency prevents either from proceeding. This often happens in code that tries to acquire
/// multiple locks without a strict global ordering.
///
/// Mitigation: Always acquire locks in a consistent global order (e.g., sort locks by ID before
/// acquiring). Use `try_lock()` with timeouts/backoff to recover from potential deadlocks.
/// Use a higher-level synchronization primitive that handles multiple resources atomically.
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

#[derive(Debug)]
pub struct Account {
    id: u32,
    balance: Mutex<f64>,
}

// This function simulates a transfer between two bank accounts.
// It takes two accounts (from and to) and an amount.
// To safely update balances, we must lock both accounts to prevent other threads from interfering.
// The code blindly locks 'from' then 'to'.
// If one thread transfers A -> B, and another transfers B -> A strictly at the same time,
// we get a deadlock because they acquire locks in different orders.
//
// PROBLEM E1217: Classical ABBA deadlock pattern
pub fn e1217_bad_abba_deadlock(skip: bool, from: Arc<Account>, to: Arc<Account>, amount: f64) {
    // Prevent self-transfer which would deadlock immediately on the first lock
    if Arc::ptr_eq(&from, &to) {
        println!("Skipping self-transfer to avoid self-deadlock");
        return;
    }

    // Thread 1: Transfer from -> to
    let from_1 = from.clone();
    let to_1 = to.clone();
    let t1 = thread::spawn(move || {
        // Lock the source account
        // PROBLEM E1002: direct unwrap/expect
        let mut from_guard = from_1.balance.lock().unwrap();
        println!("Thread 1: Locked account {}", from_1.id);

        // Simulate some processing time (makes deadlock more likely)
        thread::sleep(Duration::from_millis(50));

        println!("Thread 1: Waiting for account {}", to_1.id);
        // Lock the destination account
        // PROBLEM E1002: direct unwrap/expect
        let mut to_guard = to_1.balance.lock().unwrap(); // DEADLOCK HERE if Thread 2 holds this
        println!("Thread 1: Locked account {}", to_1.id);

        *from_guard -= amount;
        *to_guard += amount;
    });

    // Needed to avoid real hangup
    if skip {
        let _ = t1.join();
        return;
    }

    // Thread 2: Transfer to -> from (Opposite direction!)
    let from_2 = from.clone();
    let to_2 = to.clone();
    let t2 = thread::spawn(move || {
        // Note: We use 'to' as source here, so we lock 'to' first!
        // This creates the AB vs BA inconsistent ordering.
        // PROBLEM E1002: direct unwrap/expect
        let mut from_guard = to_2.balance.lock().unwrap();
        println!("Thread 2: Locked account {}", to_2.id);

        thread::sleep(Duration::from_millis(50));

        println!("Thread 2: Waiting for account {}", from_2.id);
        // PROBLEM E1002: direct unwrap/expect
        let mut to_guard = from_2.balance.lock().unwrap(); // DEADLOCK HERE if Thread 1 holds this
        println!("Thread 2: Locked account {}", from_2.id);

        *from_guard -= amount;
        *to_guard += amount;
    });

    // In a real deadlock, these joins would never return
    let _ = t1.join();
    let _ = t2.join();
}

pub fn e1217_entry() -> Result<(), Box<dyn std::error::Error>> {
    let from = Arc::new(Account {
        id: 1,
        balance: Mutex::new(100.0),
    });
    let to = Arc::new(Account {
        id: 2,
        balance: Mutex::new(100.0),
    });
    e1217_bad_abba_deadlock(true, from, to, 10.0);
    Ok(())
}
