/// E1409: Partial initialization
/// Severity: MED
/// LLM confusion: 2 (LOW)
///
/// Description: This code only initializes some elements of an array, leaving others with default
/// values (zeros). While this is safe, it might indicate a logic error if all elements were supposed
/// to be initialized. It's like filling out only half of a form and leaving the rest blank - might
/// be intentional, might be a mistake. Fix by initializing all elements explicitly or using a
/// different data structure that makes partial initialization clearer.
///
/// ## The Ambiguity Problem
///
/// ```text
/// let mut data = [0u8; 4];
/// data[0] = 1;
/// data[1] = 2;
/// // data[2] and data[3] are still 0 - is this intentional?
/// ```
///
/// ## Why This Matters
///
/// 1. **Unclear intent**: Was the partial init intentional or a bug?
/// 2. **Silent bugs**: Missing initializations go unnoticed
/// 3. **Maintenance risk**: Future changes might miss elements
/// 4. **Review difficulty**: Reviewers can't tell if it's correct
///
/// ## The Right Solutions
///
/// ### Option 1: Initialize all elements explicitly
/// ```rust
/// let data = [1, 2, 0, 0];  // Explicit that 0s are intentional
/// ```
///
/// ### Option 2: Use Vec for dynamic initialization
/// ```rust
/// let mut data = Vec::new();
/// data.push(1);
/// data.push(2);
/// // Clear that only 2 elements exist
/// ```
///
/// ### Option 3: Use Option for optional elements
/// ```rust
/// let mut data: [Option<u8>; 4] = [None; 4];
/// data[0] = Some(1);
/// data[1] = Some(2);
/// // Clear which elements are set
/// ```
///
/// Mitigation: Use array initialization syntax like `[value; size]` or `[a, b, c, d]` to make
/// initialization explicit. Consider using `Vec` if you're building up elements incrementally.
/// Add comments if partial initialization is intentional.

// ============================================================================
// AMBIGUOUS PATTERNS
// ============================================================================

/// PROBLEM E1409: Only initializing some fields
pub fn e1409_bad_partial() {
    // PROBLEM E1409: Only initializing some fields
    let mut data = [0u8; 4];
    data[0] = 1;
    data[1] = 2;
    // data[2] and data[3] remain 0 - might be unintended
    let _sum: u8 = data.iter().sum(); // Use the data to avoid warnings
}

/// PROBLEM E1409: Loop doesn't initialize all elements
pub fn e1409_bad_partial_loop() {
    let mut data = [0i32; 10];
    for i in 0..5 {
        // Only initializes first 5 elements!
        data[i] = i as i32 * 2;
    }
    let _sum: i32 = data.iter().sum();
}

/// PROBLEM E1409: Conditional initialization may leave gaps
pub fn e1409_bad_conditional(condition: bool) -> [i32; 4] {
    let mut data = [0i32; 4];
    data[0] = 1;
    if condition {
        data[1] = 2;
    }
    // data[1] might be 0 or 2 depending on condition
    data
}

/// Entry point for problem demonstration
pub fn e1409_entry() -> Result<(), Box<dyn std::error::Error>> {
    e1409_bad_partial();
    Ok(())
}

// ============================================================================
// GOOD ALTERNATIVES
// ============================================================================

/// GOOD: Explicit full initialization
pub fn e1409_good_explicit() -> [u8; 4] {
    [1, 2, 0, 0] // Clear that 0s are intentional
}

/// GOOD: Use array initialization with from_fn
pub fn e1409_good_from_fn() -> [i32; 10] {
    std::array::from_fn(|i| if i < 5 { i as i32 * 2 } else { 0 })
}

/// GOOD: Use Vec for dynamic initialization
pub fn e1409_good_vec() -> Vec<u8> {
    let mut data = Vec::new();
    data.push(1);
    data.push(2);
    // Clear that only 2 elements exist
    data
}

/// GOOD: Use Option to indicate which elements are set
pub fn e1409_good_option() -> [Option<u8>; 4] {
    let mut data: [Option<u8>; 4] = [None; 4];
    data[0] = Some(1);
    data[1] = Some(2);
    // Clear which elements are set
    data
}

/// GOOD: Document intentional partial initialization
pub fn e1409_good_documented() -> [i32; 4] {
    let mut data = [0i32; 4];
    // Only first two elements are used; rest are padding for alignment
    data[0] = 1;
    data[1] = 2;
    data
}

/// GOOD: Use Default trait
#[derive(Default)]
pub struct Config {
    pub port: u16,
    pub host: String,
    pub timeout: u32,
}

pub fn e1409_good_default() -> Config {
    Config {
        port: 8080,
        // host and timeout use Default values explicitly
        ..Default::default()
    }
}

/// GOOD: Builder pattern for complex initialization
pub struct DataBuilder {
    data: Vec<i32>,
    capacity: usize,
}

impl DataBuilder {
    pub fn new(capacity: usize) -> Self {
        Self {
            data: Vec::with_capacity(capacity),
            capacity,
        }
    }

    pub fn add(&mut self, value: i32) -> &mut Self {
        if self.data.len() < self.capacity {
            self.data.push(value);
        }
        self
    }

    pub fn build(self) -> Vec<i32> {
        self.data
    }
}

/// GOOD: Use collect for initialization from iterator
pub fn e1409_good_collect() -> Vec<i32> {
    (0..5).map(|i| i * 2).collect()
}

// ============================================================================
// GOOD EXAMPLES unit tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_explicit_init() {
        let data = e1409_good_explicit();
        assert_eq!(data, [1, 2, 0, 0]);
    }

    #[test]
    fn test_from_fn() {
        let data = e1409_good_from_fn();
        assert_eq!(&data[..5], &[0, 2, 4, 6, 8]);
        assert_eq!(&data[5..], &[0, 0, 0, 0, 0]);
    }

    #[test]
    fn test_option_init() {
        let data = e1409_good_option();
        assert_eq!(data[0], Some(1));
        assert_eq!(data[1], Some(2));
        assert_eq!(data[2], None);
        assert_eq!(data[3], None);
    }

    #[test]
    fn test_builder() {
        let mut builder = DataBuilder::new(5);
        builder.add(1).add(2).add(3);
        let data = builder.build();
        assert_eq!(data, vec![1, 2, 3]);
    }

    #[test]
    fn test_collect() {
        let data = e1409_good_collect();
        assert_eq!(data, vec![0, 2, 4, 6, 8]);
    }
}
