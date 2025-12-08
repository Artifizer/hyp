/// E1309: Panic in Drop implementation
/// Severity: HIGH
/// LLM confusion: 4 (HIGH)
///
/// Description: The Drop trait runs cleanup code when a value is destroyed (like a destructor or
/// finally block). Panicking (crashing) in Drop is extremely dangerous because if Drop runs while
/// already handling another crash, it causes a double-crash which immediately terminates the entire
/// program with no chance to recover. It's like throwing an exception in a finally block while
/// already unwinding from another exception. Fix by handling errors in Drop gracefully - log them
/// instead of crashing.
///
/// ## The Double Panic Problem
///
/// ```text
/// fn process() {
///     let resource = PanickingDrop::new();
///     panic!("First panic!");  // Starts unwinding
///         ↓
///     // During unwinding, resource.drop() is called
///         ↓
///     drop() calls panic!("Cleanup failed!")  // Second panic!
///         ↓
///     ABORT! Program immediately terminates, no recovery possible
/// }
/// ```
///
/// ## Why This Matters
///
/// 1. **Immediate abort**: Double panic = immediate process termination
/// 2. **No cleanup**: Other destructors don't run, resources leak
/// 3. **No error handling**: catch_unwind can't help with double panics
/// 4. **Data loss**: Buffered writes, transactions, etc. are lost
///
/// ## The Right Solutions
///
/// ### Option 1: Log errors instead of panicking
/// ```rust
/// impl Drop for Resource {
///     fn drop(&mut self) {
///         if let Err(e) = self.cleanup() {
///             eprintln!("Warning: cleanup failed: {}", e);
///         }
///     }
/// }
/// ```
///
/// ### Option 2: Store error for later retrieval
/// ```rust
/// struct Resource {
///     cleanup_error: Option<std::io::Error>,
/// }
///
/// impl Drop for Resource {
///     fn drop(&mut self) {
///         if let Err(e) = self.do_cleanup() {
///             self.cleanup_error = Some(e);
///         }
///     }
/// }
/// ```
///
/// ### Option 3: Provide explicit close method
/// ```rust
/// impl Resource {
///     pub fn close(self) -> Result<(), Error> {
///         // Fallible cleanup here
///         self.flush()?;
///         Ok(())
///     }
/// }
/// ```
///
/// Mitigation: Never panic in Drop implementations. Log errors instead, or store them for later
/// retrieval. Use `std::panic::catch_unwind` if you must handle panics. Consider providing an
/// explicit cleanup method for fallible operations.

// ============================================================================
// DANGEROUS PATTERNS - NEVER DO THIS
// ============================================================================

/// PROBLEM E1309: Panic in Drop can cause double panic
pub struct PanickingDrop {
    value: i32,
}

impl Drop for PanickingDrop {
    fn drop(&mut self) {
        if self.value > 0 {
            // PROBLEM E1001: Direct panic in production code
            panic!("PROBLEM: Panic in Drop can cause double panic");
        }
    }
}

/// PROBLEM E1309: unwrap in Drop
pub struct UnwrappingDrop {
    file: Option<std::fs::File>,
}

impl Drop for UnwrappingDrop {
    fn drop(&mut self) {
        if let Some(mut file) = self.file.take() {
            use std::io::Write;
            // PROBLEM E1309: unwrap in drop can panic
            file.flush().unwrap();
        }
    }
}

/// PROBLEM E1309: expect in Drop
pub struct ExpectingDrop {
    path: String,
}

impl Drop for ExpectingDrop {
    fn drop(&mut self) {
        // PROBLEM E1309: expect in drop can panic
        std::fs::remove_file(&self.path).expect("cleanup failed");
    }
}

/// Entry point for problem demonstration
pub fn e1309_entry() -> Result<(), Box<dyn std::error::Error>> {
    let _resource = PanickingDrop { value: 0 };
    Ok(())
}

// ============================================================================
// GOOD ALTERNATIVES
// ============================================================================

/// GOOD: Log errors in Drop instead of panicking
pub struct LoggingDrop {
    path: String,
}

impl LoggingDrop {
    pub fn new(path: &str) -> Self {
        Self {
            path: path.to_string(),
        }
    }
}

impl Drop for LoggingDrop {
    fn drop(&mut self) {
        if let Err(e) = std::fs::remove_file(&self.path) {
            // Log instead of panic - this is safe
            eprintln!("Warning: Failed to clean up {}: {}", self.path, e);
        }
    }
}

/// GOOD: Store error for later inspection
pub struct ErrorStoringDrop {
    value: i32,
    cleanup_error: std::cell::Cell<Option<String>>,
}

impl ErrorStoringDrop {
    pub fn new(value: i32) -> Self {
        Self {
            value,
            cleanup_error: std::cell::Cell::new(None),
        }
    }

    pub fn cleanup_error(&self) -> Option<String> {
        self.cleanup_error.take()
    }
}

impl Drop for ErrorStoringDrop {
    fn drop(&mut self) {
        if self.value < 0 {
            self.cleanup_error
                .set(Some("Cleanup failed: negative value".to_string()));
        }
    }
}

/// GOOD: Provide explicit close method for fallible cleanup
pub struct ExplicitClose {
    file: Option<std::fs::File>,
}

impl ExplicitClose {
    pub fn new(file: std::fs::File) -> Self {
        Self { file: Some(file) }
    }

    /// Explicit close that can return errors
    pub fn close(mut self) -> std::io::Result<()> {
        use std::io::Write;
        if let Some(mut file) = self.file.take() {
            file.flush()?;
        }
        Ok(())
    }
}

impl Drop for ExplicitClose {
    fn drop(&mut self) {
        // Best-effort cleanup if close() wasn't called
        if let Some(mut file) = self.file.take() {
            use std::io::Write;
            if let Err(e) = file.flush() {
                eprintln!("Warning: flush failed in drop: {}", e);
            }
        }
    }
}

/// GOOD: Use catch_unwind for risky operations
pub struct CatchingDrop {
    callback: Option<Box<dyn FnOnce()>>,
}

impl CatchingDrop {
    pub fn new(callback: Box<dyn FnOnce()>) -> Self {
        Self {
            callback: Some(callback),
        }
    }
}

impl Drop for CatchingDrop {
    fn drop(&mut self) {
        if let Some(callback) = self.callback.take() {
            // Catch any panics from the callback
            if let Err(e) = std::panic::catch_unwind(std::panic::AssertUnwindSafe(callback)) {
                eprintln!("Warning: cleanup callback panicked: {:?}", e);
            }
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
    fn test_logging_drop_handles_missing_file() {
        let _drop = LoggingDrop::new("nonexistent_file_12345.txt");
        // Should not panic, just log
    }

    #[test]
    fn test_error_storing_drop() {
        let drop = ErrorStoringDrop::new(-1);
        // Error will be stored during drop
        std::mem::drop(drop);
    }

    #[test]
    fn test_catching_drop_handles_panic() {
        let drop = CatchingDrop::new(Box::new(|| {
            panic!("This panic should be caught");
        }));
        // Should not propagate panic
        std::mem::drop(drop);
    }
}
