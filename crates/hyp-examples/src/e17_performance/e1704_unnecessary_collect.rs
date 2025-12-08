/// E1704: Unnecessary collect()
/// Severity: LOW
/// LLM confusion: 2 (LOW)
///
/// Description: Calling `.collect()` creates a new collection by consuming an iterator. This code
/// collects into a Vec, then immediately creates another iterator from it. The collect is
/// unnecessary - you can chain iterator operations without collecting intermediate results. Fix
/// by removing the unnecessary collect and chaining operations directly.
///
/// ## The Unnecessary Collection Problem
///
/// ```text
/// data.iter()
///     .filter(|x| **x > 0)
///     .collect::<Vec<_>>()  // Allocates a Vec
///     .iter()               // Creates new iterator
///     .sum()                // Consumes it
/// // The Vec was never needed!
/// ```
///
/// ## Why This Matters
///
/// 1. **Unnecessary allocation**: Vec allocated then discarded
/// 2. **Extra iteration**: Data traversed twice
/// 3. **Memory pressure**: Temporary collection uses memory
/// 4. **Lazy evaluation lost**: Iterators are lazy, collect forces evaluation
///
/// ## The Right Solutions
///
/// ### Option 1: Chain operations directly
/// ```rust
/// data.iter()
///     .filter(|x| **x > 0)
///     .sum()  // No intermediate collection
/// ```
///
/// ### Option 2: Use iterator adapters
/// ```rust
/// data.iter()
///     .filter(|x| **x > 0)
///     .map(|x| x * 2)
///     .take(10)
///     .collect()  // Only collect at the end if needed
/// ```
///
/// ### Option 3: Use for_each for side effects
/// ```rust
/// data.iter()
///     .filter(|x| **x > 0)
///     .for_each(|x| println!("{}", x));
/// ```
///
/// Mitigation: Use `#![warn(clippy::needless_collect)]` to detect unnecessary collections. Chain
/// iterator operations instead of collecting intermediate results. Only collect when you actually
/// need the final collection. Iterators are lazy and efficient.

// ============================================================================
// WASTEFUL PATTERNS
// ============================================================================

/// PROBLEM E1704: Collecting when not needed
pub fn e1704_bad_unnecessary_collect(data: Vec<i32>) -> i32 {
    // PROBLEM E1704: Collecting when not needed
    data.iter()
        .filter(|x| **x > 0)
        .collect::<Vec<_>>()
        .iter()
        .copied()
        .sum()
}

/// PROBLEM E1704: Collect then iterate again
pub fn e1704_bad_collect_then_iter(data: &[i32]) -> Vec<i32> {
    let filtered: Vec<_> = data.iter().filter(|x| **x > 0).collect();
    filtered.iter().map(|x| *x * 2).collect()
}

/// PROBLEM E1704: Collect just to get length
pub fn e1704_bad_collect_for_len(data: &[i32]) -> usize {
    data.iter().filter(|x| **x > 0).collect::<Vec<_>>().len()
}

/// Entry point for problem demonstration
pub fn e1704_entry() -> Result<(), Box<dyn std::error::Error>> {
    Ok(())
}

// ============================================================================
// GOOD ALTERNATIVES
// ============================================================================

/// GOOD: Chain operations directly
pub fn e1704_good_chain(data: &[i32]) -> i32 {
    data.iter().filter(|x| **x > 0).sum()
}

/// GOOD: Single collect at the end
pub fn e1704_good_single_collect(data: &[i32]) -> Vec<i32> {
    data.iter()
        .filter(|x| **x > 0)
        .map(|x| x * 2)
        .collect()
}

/// GOOD: Use count() instead of collect().len()
pub fn e1704_good_count(data: &[i32]) -> usize {
    data.iter().filter(|x| **x > 0).count()
}

/// GOOD: Use any() instead of collect().is_empty()
pub fn e1704_good_any(data: &[i32]) -> bool {
    data.iter().any(|x| *x > 0)
}

/// GOOD: Use find() instead of collect().first()
pub fn e1704_good_find(data: &[i32]) -> Option<i32> {
    data.iter().find(|x| **x > 0).copied()
}

/// GOOD: Use fold for complex accumulation
pub fn e1704_good_fold(data: &[i32]) -> (i32, usize) {
    data.iter()
        .filter(|x| **x > 0)
        .fold((0, 0), |(sum, count), x| (sum + x, count + 1))
}

/// GOOD: Use for_each for side effects
pub fn e1704_good_for_each(data: &[i32]) {
    data.iter()
        .filter(|x| **x > 0)
        .for_each(|x| println!("{}", x));
}

/// GOOD: Collect only when truly needed (e.g., random access)
pub fn e1704_good_when_needed(data: &[i32]) -> Option<i32> {
    let filtered: Vec<_> = data.iter().filter(|x| **x > 0).collect();
    // Need random access, so collect is justified
    let mid = filtered.len() / 2;
    filtered.get(mid).copied().copied()
}

/// GOOD: Use partition for splitting
pub fn e1704_good_partition(data: &[i32]) -> (Vec<i32>, Vec<i32>) {
    data.iter().partition(|x| **x > 0)
}

// ============================================================================
// GOOD EXAMPLES unit tests
// ============================================================================


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chain() {
        let data = vec![1, -2, 3, -4, 5];
        assert_eq!(e1704_good_chain(&data), 9);
    }

    #[test]
    fn test_single_collect() {
        let data = vec![1, -2, 3];
        assert_eq!(e1704_good_single_collect(&data), vec![2, 6]);
    }

    #[test]
    fn test_count() {
        let data = vec![1, -2, 3, -4, 5];
        assert_eq!(e1704_good_count(&data), 3);
    }

    #[test]
    fn test_any() {
        assert!(e1704_good_any(&[1, -2, 3]));
        assert!(!e1704_good_any(&[-1, -2, -3]));
    }

    #[test]
    fn test_find() {
        let data = vec![-1, -2, 3, 4];
        assert_eq!(e1704_good_find(&data), Some(3));
    }

    #[test]
    fn test_fold() {
        let data = vec![1, -2, 3, -4, 5];
        assert_eq!(e1704_good_fold(&data), (9, 3));
    }

    #[test]
    fn test_partition() {
        let data = vec![1, -2, 3, -4];
        let (pos, neg) = e1704_good_partition(&data);
        assert_eq!(pos.len(), 2);
        assert_eq!(neg.len(), 2);
        assert_eq!(pos[0], 1);
        assert_eq!(pos[1], 3);
        assert_eq!(neg[0], -2);
        assert_eq!(neg[1], -4);
    }
}
