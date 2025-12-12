/// E1608: Double free
/// Severity: HIGH
/// LLM confusion: 5 (HIGHEST)
///
/// Description: Double free occurs when you try to deallocate the same memory twice. ManuallyDrop
/// prevents automatic dropping, but calling `ManuallyDrop::drop()` twice manually drops the same
/// value twice. This causes undefined behavior - the second drop operates on freed memory. Fix by
/// ensuring each value is dropped at most once.
///
/// ## The Double Free Problem
///
/// ```text
/// let mut data = ManuallyDrop::new(vec![1, 2, 3]);
///
/// unsafe {
///     ManuallyDrop::drop(&mut data);  // First drop - OK
///     ManuallyDrop::drop(&mut data);  // Second drop - UB!
/// }
/// // Second drop tries to free already-freed memory
/// ```
///
/// ## Why This Matters
///
/// 1. **Undefined behavior**: Memory corruption, crashes, exploits
/// 2. **Security vulnerability**: Classic exploit vector
/// 3. **Hard to debug**: May not crash immediately
/// 4. **Allocator corruption**: Can corrupt heap metadata
///
/// ## The Right Solutions
///
/// ### Option 1: Use ManuallyDrop::take() to extract value
/// ```rust
/// use std::mem::ManuallyDrop;
///
/// let mut data = ManuallyDrop::new(vec![1, 2, 3]);
/// let owned = unsafe { ManuallyDrop::take(&mut data) };
/// // owned will be dropped normally, data is now invalid
/// ```
///
/// ### Option 2: Track drop state with Option
/// ```rust
/// let mut data = Some(vec![1, 2, 3]);
///
/// if let Some(v) = data.take() {
///     drop(v);  // Only drops once
/// }
/// // data is now None, can't double-drop
/// ```
///
/// ### Option 3: Use RAII (let Rust handle it)
/// ```rust
/// let data = vec![1, 2, 3];
/// // Automatically dropped once at end of scope
/// ```
///
/// Mitigation: Be extremely careful with `ManuallyDrop` - it's easy to create double-free bugs.
/// Use `ManuallyDrop::take()` to extract the value, which prevents double-drop. Avoid manual
/// memory management when possible. Use Miri to detect double-free bugs.

use std::mem::ManuallyDrop;

// ============================================================================
// DANGEROUS PATTERNS - NEVER DO THIS
// ============================================================================

/// PROBLEM E1608: Dropping twice causes undefined behavior
pub fn e1608_bad_double_free() {
    let data = vec![1, 2, 3];
    let mut manual = ManuallyDrop::new(data);

    // PROBLEM E1003: Direct use of unsafe code
    unsafe {
        // PROBLEM E1904: No safety documentation
        // PROBLEM E1608: Dropping twice
        ManuallyDrop::drop(&mut manual);
        ManuallyDrop::drop(&mut manual);
    }
}

/// PROBLEM E1608: Drop in loop can double-drop
pub fn e1608_bad_loop_drop() {
    let data = vec![1, 2, 3];
    let mut manual = ManuallyDrop::new(data);

    for _ in 0..2 {
        unsafe {
            // PROBLEM E1608: Called multiple times!
            ManuallyDrop::drop(&mut manual);
        }
    }
}

/// Entry point for problem demonstration
pub fn e1608_entry() -> Result<(), Box<dyn std::error::Error>> {
    Ok(())
}

// ============================================================================
// GOOD ALTERNATIVES
// ============================================================================

/// GOOD: Use take() to extract value (prevents double-drop)
pub fn e1608_good_take() {
    let mut manual = ManuallyDrop::new(vec![1, 2, 3]);

    // Take ownership - ManuallyDrop is now in undefined state
    let owned = unsafe { ManuallyDrop::take(&mut manual) };

    // owned will be dropped normally
    drop(owned);

    // Cannot accidentally double-drop - manual is consumed
}

/// GOOD: Use Option to track drop state
pub fn e1608_good_option() {
    let mut data = Some(vec![1, 2, 3]);

    // First drop
    if let Some(v) = data.take() {
        drop(v);
    }

    // Second attempt - data is None, nothing happens
    if let Some(v) = data.take() {
        drop(v);
    }
}

/// GOOD: Let Rust handle dropping (RAII)
pub fn e1608_good_raii() {
    let data = vec![1, 2, 3];
    println!("{:?}", data);
    // Automatically dropped once at end of scope
}

/// GOOD: Use a wrapper that tracks drop state
pub struct SafeManualDrop<T> {
    inner: Option<T>,
}

impl<T> SafeManualDrop<T> {
    pub fn new(value: T) -> Self {
        Self { inner: Some(value) }
    }

    pub fn take(&mut self) -> Option<T> {
        self.inner.take()
    }

    pub fn drop_inner(&mut self) {
        self.inner.take(); // Drop by taking
    }

    pub fn is_dropped(&self) -> bool {
        self.inner.is_none()
    }
}

impl<T> std::ops::Deref for SafeManualDrop<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.inner.as_ref().expect("Already dropped")
    }
}

/// GOOD: Use scope to ensure single drop
pub fn e1608_good_scoped() {
    {
        let data = [1, 2, 3];
        // Use data...
    } // Dropped exactly once here
}

/// GOOD: Conditional drop with flag
pub fn e1608_good_conditional() {
    let mut data = ManuallyDrop::new(vec![1, 2, 3]);
    let mut dropped = false;

    // Only drop if not already dropped
    if !dropped {
        unsafe {
            ManuallyDrop::drop(&mut data);
        }
        dropped = true;
    }

    // Second attempt - flag prevents double drop
    if !dropped {
        unsafe {
            ManuallyDrop::drop(&mut data);
        }
    }
}

// ============================================================================
// GOOD EXAMPLES unit tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_option_pattern() {
        e1608_good_option();
    }

    #[test]
    fn test_safe_manual_drop() {
        let mut safe = SafeManualDrop::new(vec![1, 2, 3]);
        assert!(!safe.is_dropped());

        safe.drop_inner();
        assert!(safe.is_dropped());

        // Safe to call again
        safe.drop_inner();
        assert!(safe.is_dropped());
    }

    #[test]
    fn test_safe_manual_drop_take() {
        let mut safe = SafeManualDrop::new(42);
        let value = safe.take();
        assert_eq!(value, Some(42));
        assert_eq!(safe.take(), None);
    }
}
