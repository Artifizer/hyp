/// E1012: Unsafe auto trait implementation
/// Severity: HIGH
/// LLM confusion: 5 (HIGHEST)
///
/// Description: Auto traits (Send, Sync, Unpin) are special traits that the compiler automatically
/// implements based on a type's structure. Manually implementing an unsafe auto trait overrides
/// the compiler's safety checks and promises that your type upholds certain guarantees. If you
/// get it wrong, you cause undefined behavior that's very hard to track down. It's like telling
/// the compiler "trust me, I know what I'm doing" about thread safety or memory layout.
///
/// Mitigation: Never implement Send/Sync manually unless you fully understand the implications.
/// Use safe wrappers like Arc, Mutex instead. If you must implement unsafe auto traits, add
/// extensive safety documentation. Use Miri to test for undefined behavior.
use std::marker::PhantomData;
use std::ptr::NonNull;

// PROBLEM E1012: Manually implementing Send/Sync for a type with raw pointers
pub struct UnsafeWrapper<T> {
    ptr: NonNull<T>,
    _marker: PhantomData<T>,
}

impl<T> UnsafeWrapper<T> {
    pub fn e1012_bad_unsafe_auto_trait(value: T) -> Self {
        let boxed = Box::new(value);
        UnsafeWrapper {
            ptr: NonNull::new(Box::into_raw(boxed)).unwrap(),
            _marker: PhantomData,
        }
    }

    pub unsafe fn e1012_bad_get(&self) -> &T {
        self.ptr.as_ref()
    }
}

// PROBLEM E1008: Implementing unsafe trait without proper safety guarantees
// PROBLEM E1012: Claiming this type is Send/Sync without proper synchronization
// This is UNSAFE and could cause data races
unsafe impl<T> Send for UnsafeWrapper<T> where T: Send {}

// PROBLEM E1008: Implementing unsafe trait without proper safety guarantees
// PROBLEM E1012: Claiming this type is Send/Sync without proper synchronization
// This is UNSAFE and could cause data races
unsafe impl<T> Sync for UnsafeWrapper<T> where T: Sync {}

impl<T> Drop for UnsafeWrapper<T> {
    fn drop(&mut self) {
        // PROBLEM E1003: Direct use of unsafe code
        unsafe {
            // PROBLEM E1004: No safety documentation
            drop(Box::from_raw(self.ptr.as_ptr()));
        }
    }
}

// PROBLEM E1012: Implementing Unpin without understanding its implications
pub struct CustomFuture<T> {
    data: *mut T,
}

// PROBLEM E1012: This claims the future can be moved freely, which might not be safe
impl<T> Unpin for CustomFuture<T> {}

pub fn e1012_bad_wrapper_get() -> Result<(), Box<dyn std::error::Error>> {
    let wrapper = UnsafeWrapper::e1012_bad_unsafe_auto_trait(42);
    // PROBLEM E1003: Direct use of unsafe code
    unsafe {
        // PROBLEM E1004: No safety documentation
        let _ = wrapper.e1012_bad_get();
    }
    Ok(())
}

pub fn e1012_entry() -> Result<(), Box<dyn std::error::Error>> {
    e1012_bad_wrapper_get()
}

// ============================================================================
// GOOD EXAMPLES - Proper alternatives
// ============================================================================

use std::sync::Arc;

/// GOOD: Use Box which automatically implements Send/Sync safely
pub struct GoodWrapper<T> {
    data: Box<T>,
}

impl<T> GoodWrapper<T> {
    pub fn new(value: T) -> Self {
        GoodWrapper {
            data: Box::new(value),
        }
    }

    pub fn e1012_good_get(&self) -> &T {
        &self.data
    }
}
// GoodWrapper is automatically Send+Sync when T is Send+Sync

/// GOOD: Use Arc for shared ownership
pub struct SharedWrapper<T> {
    data: Arc<T>,
}

impl<T> SharedWrapper<T> {
    pub fn new(value: T) -> Self {
        SharedWrapper {
            data: Arc::new(value),
        }
    }

    pub fn e1012_good_get(&self) -> &T {
        &self.data
    }
}
// SharedWrapper is automatically Send+Sync when T is Send+Sync

/// GOOD: If unsafe impl is needed, document thoroughly
pub struct DocumentedWrapper<T: Send + Sync> {
    ptr: NonNull<T>,
    _marker: PhantomData<T>,
}

impl<T: Send + Sync> DocumentedWrapper<T> {
    pub fn new(value: T) -> Self {
        let boxed = Box::new(value);
        DocumentedWrapper {
            ptr: NonNull::new(Box::into_raw(boxed)).unwrap(),
            _marker: PhantomData,
        }
    }
}

// SAFETY: DocumentedWrapper can be sent across threads because:
// 1. The inner T is Send, so its ownership can be transferred
// 2. The pointer is valid and owned by this struct
// 3. No other copies of this pointer exist
unsafe impl<T: Send + Sync> Send for DocumentedWrapper<T> {}

// SAFETY: DocumentedWrapper can be shared across threads because:
// 1. The inner T is Sync, so shared access is safe
// 2. get() returns an immutable reference
// 3. No mutable access is exposed
unsafe impl<T: Send + Sync> Sync for DocumentedWrapper<T> {}

impl<T: Send + Sync> Drop for DocumentedWrapper<T> {
    fn drop(&mut self) {
        // SAFETY: ptr was created from Box::into_raw and is still valid
        unsafe {
            drop(Box::from_raw(self.ptr.as_ptr()));
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
    fn e1012_good_wrapper_gets_value() {
        let wrapper = GoodWrapper::new(5);
        assert_eq!(*wrapper.e1012_good_get(), 5);
    }

    #[test]
    fn e1012_good_shared_wrapper_shares_arc() {
        let wrapper = SharedWrapper::new(7);
        assert_eq!(*wrapper.e1012_good_get(), 7);
    }

    #[test]
    fn e1012_good_documented_wrapper_constructs() {
        let _wrapper = DocumentedWrapper::new(9);
    }
}
