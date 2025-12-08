/// E1808: Mutable getter
/// Severity: MED
/// LLM confusion: 2 (LOW)
///
/// Description: Returning a mutable reference to internal data breaks encapsulation - callers can
/// modify the internal state in ways that violate invariants. This getter returns `&mut Vec`,
/// allowing callers to clear it, add invalid data, etc. Fix by providing specific methods for
/// allowed operations instead of exposing mutable internals.
///
/// ## The Broken Encapsulation Problem
///
/// ```text
/// impl Container {
///     pub fn get_data_mut(&mut self) -> &mut Vec<i32> {
///         &mut self.data
///     }
/// }
///
/// container.get_data_mut().clear();  // Bypasses any invariant checks!
/// container.get_data_mut().push(-1); // Could violate "positive only" rule
/// ```
///
/// ## Why This Matters
///
/// 1. **Invariant violations**: Users can corrupt internal state
/// 2. **No validation**: Mutations bypass your checks
/// 3. **Tight coupling**: Users depend on internal representation
/// 4. **Hard to debug**: State changes happen outside your control
///
/// ## The Right Solutions
///
/// ### Option 1: Specific mutation methods
/// ```rust
/// impl Container {
///     pub fn add(&mut self, value: i32) -> Result<(), &'static str> {
///         if value < 0 {
///             return Err("Value must be non-negative");
///         }
///         self.data.push(value);
///         Ok(())
///     }
///
///     pub fn remove(&mut self, index: usize) -> Option<i32> {
///         if index < self.data.len() {
///             Some(self.data.remove(index))
///         } else {
///             None
///         }
///     }
/// }
/// ```
///
/// ### Option 2: Return iterators
/// ```rust
/// impl Container {
///     pub fn iter(&self) -> impl Iterator<Item = &i32> {
///         self.data.iter()
///     }
/// }
/// ```
///
/// Mitigation: Avoid mutable getters. Provide specific methods for allowed mutations (e.g.,
/// `add_item()`, `remove_item()`). If you must expose collections, return iterators or slices
/// instead of mutable references. Use `#![warn(clippy::mut_from_ref)]` to catch suspicious patterns.

// ============================================================================
// PROBLEMATIC PATTERNS
// ============================================================================

/// PROBLEM E1808: Returns mutable reference to internal data
pub struct BadMutableGetter {
    data: Vec<i32>,
    max_size: usize,
}

impl BadMutableGetter {
    pub fn new(max_size: usize) -> Self {
        Self {
            data: Vec::new(),
            max_size,
        }
    }

    // PROBLEM E1808: Returning mutable reference breaks encapsulation
    pub fn get_data_mut(&mut self) -> &mut Vec<i32> {
        &mut self.data
    }

    pub fn max_size(&self) -> usize {
        self.max_size
    }
}

/// Entry point for problem demonstration
pub fn e1808_entry() -> Result<(), Box<dyn std::error::Error>> {
    let mut container = BadMutableGetter::new(3);

    // User can violate invariants!
    container.get_data_mut().push(1);
    container.get_data_mut().push(2);
    container.get_data_mut().push(3);
    container.get_data_mut().push(4); // Exceeds max_size, but allowed!
    container.get_data_mut().clear(); // Bypasses any cleanup logic

    Ok(())
}

// ============================================================================
// GOOD ALTERNATIVES
// ============================================================================

/// GOOD: Specific methods for allowed operations
pub struct GoodControlledAccess {
    data: Vec<i32>,
    max_size: usize,
}

impl GoodControlledAccess {
    pub fn new(max_size: usize) -> Self {
        Self {
            data: Vec::with_capacity(max_size),
            max_size,
        }
    }

    /// Adds a value if there's room.
    pub fn add(&mut self, value: i32) -> Result<(), &'static str> {
        if self.data.len() >= self.max_size {
            return Err("Container is full");
        }
        self.data.push(value);
        Ok(())
    }

    /// Removes and returns the last value.
    pub fn pop(&mut self) -> Option<i32> {
        self.data.pop()
    }

    /// Removes value at index.
    pub fn remove(&mut self, index: usize) -> Option<i32> {
        if index < self.data.len() {
            Some(self.data.remove(index))
        } else {
            None
        }
    }

    /// Clears with optional callback for cleanup.
    pub fn clear(&mut self) {
        // Could add logging, cleanup, etc.
        self.data.clear();
    }

    /// Read-only access to data.
    pub fn get(&self, index: usize) -> Option<i32> {
        self.data.get(index).copied()
    }

    /// Read-only slice access.
    pub fn as_slice(&self) -> &[i32] {
        &self.data
    }

    /// Iterator over values.
    pub fn iter(&self) -> impl Iterator<Item = &i32> {
        self.data.iter()
    }

    pub fn len(&self) -> usize {
        self.data.len()
    }

    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    pub fn is_full(&self) -> bool {
        self.data.len() >= self.max_size
    }
}

/// GOOD: Validated mutations only
pub struct GoodValidatedMutations {
    data: Vec<i32>,
}

impl GoodValidatedMutations {
    pub fn new() -> Self {
        Self { data: Vec::new() }
    }

    /// Only allows positive values.
    pub fn add_positive(&mut self, value: i32) -> Result<(), &'static str> {
        if value <= 0 {
            return Err("Value must be positive");
        }
        self.data.push(value);
        Ok(())
    }

    /// Allows updating with validation.
    pub fn update(&mut self, index: usize, value: i32) -> Result<(), &'static str> {
        if value <= 0 {
            return Err("Value must be positive");
        }
        if index >= self.data.len() {
            return Err("Index out of bounds");
        }
        self.data[index] = value;
        Ok(())
    }

    pub fn as_slice(&self) -> &[i32] {
        &self.data
    }
}

impl Default for GoodValidatedMutations {
    fn default() -> Self {
        Self::new()
    }
}

/// GOOD: Use interior mutability when needed
use std::cell::RefCell;

pub struct GoodInteriorMutability {
    data: RefCell<Vec<i32>>,
}

impl GoodInteriorMutability {
    pub fn new() -> Self {
        Self {
            data: RefCell::new(Vec::new()),
        }
    }

    /// Allows mutation through shared reference, but controlled.
    pub fn add(&self, value: i32) {
        self.data.borrow_mut().push(value);
    }

    /// Returns a copy, not a reference.
    pub fn get(&self, index: usize) -> Option<i32> {
        self.data.borrow().get(index).copied()
    }

    pub fn len(&self) -> usize {
        self.data.borrow().len()
    }

    pub fn is_empty(&self) -> bool {
        self.data.borrow().is_empty()
    }
}

impl Default for GoodInteriorMutability {
    fn default() -> Self {
        Self::new()
    }
}

/// GOOD: Entry API for complex mutations
use std::collections::HashMap;

pub struct GoodEntryApi {
    data: HashMap<String, i32>,
}

impl GoodEntryApi {
    pub fn new() -> Self {
        Self {
            data: HashMap::new(),
        }
    }

    /// Increment value or insert default.
    pub fn increment(&mut self, key: &str) {
        *self.data.entry(key.to_string()).or_insert(0) += 1;
    }

    /// Get or compute value.
    pub fn get_or_compute(&mut self, key: &str, compute: impl FnOnce() -> i32) -> i32 {
        *self.data.entry(key.to_string()).or_insert_with(compute)
    }

    pub fn get(&self, key: &str) -> Option<i32> {
        self.data.get(key).copied()
    }
}

impl Default for GoodEntryApi {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// GOOD EXAMPLES unit tests
// ============================================================================


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_controlled_access() {
        let mut container = GoodControlledAccess::new(3);
        assert!(container.add(1).is_ok());
        assert!(container.add(2).is_ok());
        assert!(container.add(3).is_ok());
        assert!(container.add(4).is_err()); // Full!
    }

    #[test]
    fn test_validated_mutations() {
        let mut container = GoodValidatedMutations::new();
        assert!(container.add_positive(1).is_ok());
        assert!(container.add_positive(0).is_err());
        assert!(container.add_positive(-1).is_err());
    }

    #[test]
    fn test_interior_mutability() {
        let container = GoodInteriorMutability::new();
        container.add(1);
        container.add(2);
        assert_eq!(container.len(), 2);
    }

    #[test]
    fn test_entry_api() {
        let mut container = GoodEntryApi::new();
        container.increment("key");
        container.increment("key");
        assert_eq!(container.get("key"), Some(2));
    }
}
