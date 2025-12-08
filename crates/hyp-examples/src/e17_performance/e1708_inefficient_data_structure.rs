/// E1708: Inefficient data structure
/// Severity: LOW
/// LLM confusion: 2 (LOW)
///
/// Description: Different data structures have different performance characteristics. Vec is
/// optimized for indexed access and iteration, but checking membership with `.contains()` is
/// O(N) - it scans the entire vector. For membership testing, HashSet is O(1). Fix by using
/// HashSet when you need fast membership testing.
///
/// ## The Wrong Tool Problem
///
/// ```text
/// let vec = vec![1, 2, 3, ..., 1000000];
/// vec.contains(&999999);  // Scans 999,999 elements!
///
/// let set: HashSet<_> = vec.into_iter().collect();
/// set.contains(&999999);  // O(1) lookup
/// ```
///
/// ## Why This Matters
///
/// 1. **O(N) vs O(1)**: Linear vs constant time
/// 2. **Scalability**: Small data OK, large data slow
/// 3. **Hidden cost**: `.contains()` looks simple but is expensive
/// 4. **Repeated lookups**: Cost multiplies with lookup count
///
/// ## The Right Solutions
///
/// ### Option 1: Use HashSet for membership
/// ```rust
/// use std::collections::HashSet;
///
/// let set: HashSet<_> = data.into_iter().collect();
/// set.contains(&target)  // O(1)
/// ```
///
/// ### Option 2: Use HashMap for key-value
/// ```rust
/// use std::collections::HashMap;
///
/// let map: HashMap<_, _> = data.into_iter().collect();
/// map.get(&key)  // O(1)
/// ```
///
/// ### Option 3: Use BTreeSet for sorted + membership
/// ```rust
/// use std::collections::BTreeSet;
///
/// let set: BTreeSet<_> = data.into_iter().collect();
/// set.contains(&target)  // O(log N)
/// set.range(10..20)      // Efficient range queries
/// ```
///
/// Mitigation: Use HashSet for membership testing, HashMap for key-value lookups, BTreeSet/BTreeMap
/// for sorted data, Vec for indexed access. Understand the time complexity of operations on each
/// data structure. Profile to verify your choice improves performance.

use std::collections::{BTreeSet, HashMap, HashSet};

// ============================================================================
// INEFFICIENT PATTERNS
// ============================================================================

/// PROBLEM E1708: Using Vec for membership testing (should use HashSet)
#[allow(clippy::ptr_arg)]
pub fn e1708_bad_vec_contains(data: &Vec<i32>, target: i32) -> bool {
    // PROBLEM E1708: Using Vec for membership testing (should use HashSet)
    data.contains(&target)
}

/// PROBLEM E1708: Repeated membership checks on Vec
pub fn e1708_bad_repeated_contains(data: &[i32], targets: &[i32]) -> Vec<bool> {
    targets.iter().map(|t| data.contains(t)).collect()
}

/// PROBLEM E1708: Using Vec as lookup table
pub fn e1708_bad_vec_lookup(pairs: &[(String, i32)], key: &str) -> Option<i32> {
    for (k, v) in pairs {
        if k == key {
            return Some(*v);
        }
    }
    None
}

/// Entry point for problem demonstration
pub fn e1708_entry() -> Result<(), Box<dyn std::error::Error>> {
    Ok(())
}

// ============================================================================
// GOOD ALTERNATIVES
// ============================================================================

/// GOOD: Use HashSet for membership testing
pub fn e1708_good_hashset_contains(data: &HashSet<i32>, target: i32) -> bool {
    data.contains(&target)
}

/// GOOD: Convert to HashSet for repeated lookups
pub fn e1708_good_convert_to_set(data: &[i32], targets: &[i32]) -> Vec<bool> {
    let set: HashSet<_> = data.iter().collect();
    targets.iter().map(|t| set.contains(t)).collect()
}

/// GOOD: Use HashMap for key-value lookup
pub fn e1708_good_hashmap_lookup(map: &HashMap<String, i32>, key: &str) -> Option<i32> {
    map.get(key).copied()
}

/// GOOD: Use BTreeSet for sorted membership + range queries
pub fn e1708_good_btreeset(data: &BTreeSet<i32>, target: i32) -> bool {
    data.contains(&target)
}

/// GOOD: BTreeSet range query
pub fn e1708_good_btreeset_range(data: &BTreeSet<i32>, min: i32, max: i32) -> Vec<i32> {
    data.range(min..=max).copied().collect()
}

/// GOOD: Use appropriate structure based on access pattern
pub struct EfficientLookup {
    // For O(1) membership
    set: HashSet<i32>,
    // For O(1) key-value
    map: HashMap<String, i32>,
    // For sorted iteration
    sorted: BTreeSet<i32>,
    // For indexed access
    vec: Vec<i32>,
}

impl EfficientLookup {
    pub fn new(data: Vec<i32>) -> Self {
        Self {
            set: data.iter().copied().collect(),
            map: HashMap::new(),
            sorted: data.iter().copied().collect(),
            vec: data,
        }
    }

    pub fn contains(&self, value: i32) -> bool {
        self.set.contains(&value) // O(1)
    }

    pub fn get_by_index(&self, index: usize) -> Option<i32> {
        self.vec.get(index).copied() // O(1)
    }

    pub fn range(&self, min: i32, max: i32) -> impl Iterator<Item = &i32> {
        self.sorted.range(min..=max) // O(log N) + O(K)
    }
}

/// Decision helper
pub fn e1708_choose_structure() {
    println!("Use Vec when:");
    println!("  - Need indexed access (vec[i])");
    println!("  - Need to maintain insertion order");
    println!("  - Iterating over all elements");
    println!();
    println!("Use HashSet when:");
    println!("  - Membership testing (contains)");
    println!("  - Deduplication");
    println!("  - Set operations (union, intersection)");
    println!();
    println!("Use HashMap when:");
    println!("  - Key-value lookups");
    println!("  - Counting occurrences");
    println!("  - Caching/memoization");
    println!();
    println!("Use BTreeSet/BTreeMap when:");
    println!("  - Need sorted order");
    println!("  - Range queries");
    println!("  - Min/max operations");
}

// ============================================================================
// GOOD EXAMPLES unit tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hashset_contains() {
        let set: HashSet<_> = vec![1, 2, 3, 4, 5].into_iter().collect();
        assert!(e1708_good_hashset_contains(&set, 3));
        assert!(!e1708_good_hashset_contains(&set, 10));
    }

    #[test]
    fn test_convert_to_set() {
        let data = vec![1, 2, 3, 4, 5];
        let targets = vec![1, 10, 3];
        let results = e1708_good_convert_to_set(&data, &targets);
        assert_eq!(results, vec![true, false, true]);
    }

    #[test]
    fn test_hashmap_lookup() {
        let mut map = HashMap::new();
        map.insert("a".to_string(), 1);
        map.insert("b".to_string(), 2);

        assert_eq!(e1708_good_hashmap_lookup(&map, "a"), Some(1));
        assert_eq!(e1708_good_hashmap_lookup(&map, "c"), None);
    }

    #[test]
    fn test_btreeset_range() {
        let set: BTreeSet<_> = vec![1, 3, 5, 7, 9, 11].into_iter().collect();
        let range = e1708_good_btreeset_range(&set, 3, 9);
        assert_eq!(range, vec![3, 5, 7, 9]);
    }

    #[test]
    fn test_efficient_lookup() {
        let lookup = EfficientLookup::new(vec![1, 2, 3, 4, 5]);
        assert!(lookup.contains(3));
        assert_eq!(lookup.get_by_index(2), Some(3));
    }
}
