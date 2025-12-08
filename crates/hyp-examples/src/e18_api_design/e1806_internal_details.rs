/// E1806: Exposing internal details
/// Severity: MED
/// LLM confusion: 2 (LOW)
///
/// Description: Exposing internal implementation details (like a Vec used as a cache) in your
/// public API makes it hard to change the implementation later. Users might depend on Vec-specific
/// behavior, preventing you from switching to a different data structure. Fix by hiding
/// implementation details behind methods or opaque types.
///
/// ## The Leaky Abstraction Problem
///
/// ```text
/// pub struct Cache {
///     pub internal_cache: Vec<i32>,  // Exposed!
/// }
///
/// // User code:
/// cache.internal_cache.push(42);  // Depends on Vec
/// cache.internal_cache.sort();    // Now you can't change to HashMap!
/// ```
///
/// ## Why This Matters
///
/// 1. **Can't change implementation**: Users depend on internals
/// 2. **Invariant violations**: Users can corrupt internal state
/// 3. **API bloat**: Vec methods become part of your API
/// 4. **Semver hazard**: Changing internals becomes breaking change
///
/// ## The Right Solutions
///
/// ### Option 1: Private fields with methods
/// ```rust
/// pub struct Cache {
///     cache: Vec<i32>,  // Private
/// }
///
/// impl Cache {
///     pub fn add(&mut self, value: i32) {
///         self.cache.push(value);
///     }
///
///     pub fn get(&self, index: usize) -> Option<i32> {
///         self.cache.get(index).copied()
///     }
/// }
/// ```
///
/// ### Option 2: Return iterators instead of collections
/// ```rust
/// impl Cache {
///     pub fn iter(&self) -> impl Iterator<Item = &i32> {
///         self.cache.iter()
///     }
/// }
/// ```
///
/// Mitigation: Make internal fields private. Provide methods for necessary operations instead of
/// direct field access. Use opaque types or trait objects to hide implementation. This allows you
/// to change internals without breaking users.

// ============================================================================
// PROBLEMATIC PATTERNS
// ============================================================================

/// PROBLEM E1806: Exposing internal Vec directly
pub struct BadExposedCache {
    pub internal_cache: Vec<i32>,
}

impl BadExposedCache {
    pub fn new() -> Self {
        Self {
            internal_cache: Vec::new(),
        }
    }
}

impl Default for BadExposedCache {
    fn default() -> Self {
        Self::new()
    }
}

/// PROBLEM E1806: Exposing implementation-specific types
pub struct BadExposedHashMap {
    pub data: std::collections::HashMap<String, i32>,
}

/// Entry point for problem demonstration
pub fn e1806_entry() -> Result<(), Box<dyn std::error::Error>> {
    let mut cache = BadExposedCache::new();
    // Users can manipulate internals directly!
    cache.internal_cache.push(42);
    cache.internal_cache.clear(); // Bypasses any validation
    Ok(())
}

// ============================================================================
// GOOD ALTERNATIVES
// ============================================================================

/// GOOD: Private fields with controlled access
pub struct GoodCache {
    cache: Vec<i32>, // Private!
    max_size: usize,
}

impl GoodCache {
    /// Creates a new cache with the given maximum size.
    pub fn new(max_size: usize) -> Self {
        Self {
            cache: Vec::with_capacity(max_size),
            max_size,
        }
    }

    /// Adds a value to the cache.
    ///
    /// Returns false if the cache is full.
    pub fn add(&mut self, value: i32) -> bool {
        if self.cache.len() >= self.max_size {
            return false;
        }
        self.cache.push(value);
        true
    }

    /// Gets a value by index.
    pub fn get(&self, index: usize) -> Option<i32> {
        self.cache.get(index).copied()
    }

    /// Returns the number of items in the cache.
    pub fn len(&self) -> usize {
        self.cache.len()
    }

    /// Returns true if the cache is empty.
    pub fn is_empty(&self) -> bool {
        self.cache.is_empty()
    }

    /// Clears the cache.
    pub fn clear(&mut self) {
        self.cache.clear();
    }

    /// Returns an iterator over the cached values.
    ///
    /// This hides the internal Vec - we could switch to HashMap later.
    pub fn iter(&self) -> impl Iterator<Item = &i32> {
        self.cache.iter()
    }

    /// Returns a slice of the cached values.
    pub fn as_slice(&self) -> &[i32] {
        &self.cache
    }
}

/// GOOD: Opaque type that hides implementation
pub struct OpaqueStore {
    // Could be Vec, HashMap, or anything else
    data: std::collections::HashMap<String, i32>,
}

impl OpaqueStore {
    pub fn new() -> Self {
        Self {
            data: std::collections::HashMap::new(),
        }
    }

    pub fn insert(&mut self, key: &str, value: i32) {
        self.data.insert(key.to_string(), value);
    }

    pub fn get(&self, key: &str) -> Option<i32> {
        self.data.get(key).copied()
    }

    pub fn remove(&mut self, key: &str) -> Option<i32> {
        self.data.remove(key)
    }

    pub fn contains(&self, key: &str) -> bool {
        self.data.contains_key(key)
    }

    /// Returns keys - could be Vec, HashSet, or iterator
    pub fn keys(&self) -> impl Iterator<Item = &str> {
        self.data.keys().map(|s| s.as_str())
    }
}

impl Default for OpaqueStore {
    fn default() -> Self {
        Self::new()
    }
}

/// GOOD: Use traits to hide implementation
pub trait Store {
    fn insert(&mut self, key: &str, value: i32);
    fn get(&self, key: &str) -> Option<i32>;
    fn remove(&mut self, key: &str) -> Option<i32>;
}

/// Implementation A - uses HashMap
pub struct HashMapStore {
    data: std::collections::HashMap<String, i32>,
}

impl HashMapStore {
    pub fn new() -> Self {
        Self {
            data: std::collections::HashMap::new(),
        }
    }
}

impl Default for HashMapStore {
    fn default() -> Self {
        Self::new()
    }
}

impl Store for HashMapStore {
    fn insert(&mut self, key: &str, value: i32) {
        self.data.insert(key.to_string(), value);
    }

    fn get(&self, key: &str) -> Option<i32> {
        self.data.get(key).copied()
    }

    fn remove(&mut self, key: &str) -> Option<i32> {
        self.data.remove(key)
    }
}

/// Implementation B - uses BTreeMap (could swap without breaking API)
pub struct BTreeStore {
    data: std::collections::BTreeMap<String, i32>,
}

impl BTreeStore {
    pub fn new() -> Self {
        Self {
            data: std::collections::BTreeMap::new(),
        }
    }
}

impl Default for BTreeStore {
    fn default() -> Self {
        Self::new()
    }
}

impl Store for BTreeStore {
    fn insert(&mut self, key: &str, value: i32) {
        self.data.insert(key.to_string(), value);
    }

    fn get(&self, key: &str) -> Option<i32> {
        self.data.get(key).copied()
    }

    fn remove(&mut self, key: &str) -> Option<i32> {
        self.data.remove(key)
    }
}

/// GOOD: Factory function returns trait object
pub fn create_store(sorted: bool) -> Box<dyn Store> {
    if sorted {
        Box::new(BTreeStore::new())
    } else {
        Box::new(HashMapStore::new())
    }
}

// ============================================================================
// GOOD EXAMPLES unit tests
// ============================================================================


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_good_cache() {
        let mut cache = GoodCache::new(3);
        assert!(cache.add(1));
        assert!(cache.add(2));
        assert!(cache.add(3));
        assert!(!cache.add(4)); // Full!

        assert_eq!(cache.get(0), Some(1));
        assert_eq!(cache.len(), 3);
    }

    #[test]
    fn test_opaque_store() {
        let mut store = OpaqueStore::new();
        store.insert("key", 42);
        assert_eq!(store.get("key"), Some(42));
        assert!(store.contains("key"));
    }

    #[test]
    fn test_trait_store() {
        let mut store: Box<dyn Store> = create_store(false);
        store.insert("key", 42);
        assert_eq!(store.get("key"), Some(42));
    }

    #[test]
    fn test_cache_iter() {
        let mut cache = GoodCache::new(3);
        cache.add(1);
        cache.add(2);
        cache.add(3);

        let sum: i32 = cache.iter().sum();
        assert_eq!(sum, 6);
    }
}
