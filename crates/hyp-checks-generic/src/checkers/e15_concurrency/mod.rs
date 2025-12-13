//! E15 - Concurrency checkers.

pub mod e1502_lock_across_await;
pub mod e1503_lock_poisoning;
pub mod e1506_deadlock_lock_ordering;
pub mod e1508_sleep_instead_of_sync;
pub mod e1509_channel_lifetime;
pub mod e1510_mutex_instead_of_rwlock;
pub mod e1511_unbounded_spawning;
pub mod e1512_prohibit_std_thread_spawn;
pub mod e1513_blocking_syscalls_async;
pub mod registry;

pub use e1502_lock_across_await::{E1502Config, E1502LockAcrossAwait};
pub use e1503_lock_poisoning::{E1503Config, E1503LockPoisoning};
pub use e1506_deadlock_lock_ordering::{E1506Config, E1506DeadlockLockOrdering};
pub use e1508_sleep_instead_of_sync::{E1508Config, E1508SleepInsteadOfSync};
pub use e1509_channel_lifetime::{E1509ChannelLifetime, E1509Config};
pub use e1510_mutex_instead_of_rwlock::{E1510Config, E1510MutexInsteadOfRwLock};
pub use e1511_unbounded_spawning::{E1511Config, E1511UnboundedSpawning};
pub use e1512_prohibit_std_thread_spawn::{E1512Config, E1512ProhibitStdThreadSpawn};
pub use e1513_blocking_syscalls_async::{E1513BlockingSyscallsAsync, E1513Config};
