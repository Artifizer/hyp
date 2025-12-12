/// E1709: Unnecessary boxing
/// Severity: LOW
/// LLM confusion: 2 (LOW)
///
/// Description: Boxing (heap-allocating) small values like integers is wasteful. Box adds
/// indirection (an extra pointer dereference) and a heap allocation for a value that fits
/// perfectly on the stack. Fix by using the value directly without boxing, unless you specifically
/// need heap allocation or trait objects.
///
/// ## The Unnecessary Indirection Problem
///
/// ```text
/// let x = Box::new(42);  // Heap allocation for 4 bytes!
/// *x + 1                  // Pointer dereference to read 4 bytes
///
/// let x = 42;            // Just 4 bytes on stack
/// x + 1                  // Direct access
/// ```
///
/// ## Why This Matters
///
/// 1. **Allocation overhead**: Heap allocation is expensive
/// 2. **Indirection cost**: Extra pointer dereference
/// 3. **Cache unfriendly**: Heap data may not be in cache
/// 4. **Memory fragmentation**: Many small allocations fragment heap
///
/// ## The Right Solutions
///
/// ### Option 1: Use values directly
/// ```rust
/// let x = 42;  // No Box needed
/// ```
///
/// ### Option 2: Box only when necessary
/// ```rust
/// // Recursive types need Box
/// struct Node {
///     value: i32,
///     next: Option<Box<Node>>,
/// }
///
/// // Trait objects need Box
/// let handler: Box<dyn Fn()> = Box::new(|| println!("hi"));
/// ```
///
/// ### Option 3: Use references for borrowed data
/// ```rust
/// fn process(data: &i32) {  // Reference, not Box
///     println!("{}", data);
/// }
/// ```
///
/// Mitigation: Use `#![warn(clippy::box_default)]` and `#![warn(clippy::boxed_local)]` to catch
/// unnecessary boxing. Only box when you need: trait objects, recursive types, or to move large
/// values. Don't box primitive types or small structs.

// ============================================================================
// WASTEFUL PATTERNS
// ============================================================================

/// PROBLEM E1709: Boxing primitive type unnecessarily
pub fn e1709_bad_box_primitive(x: i32) -> Box<i32> {
    // PROBLEM E1709: Boxing primitive type unnecessarily
    Box::new(x)
}

/// PROBLEM E1709: Boxing small struct
pub fn e1709_bad_box_small_struct() -> Box<(i32, i32)> {
    Box::new((1, 2)) // 8 bytes, no need to box
}

/// PROBLEM E1709: Boxing in loop
pub fn e1709_bad_box_in_loop() -> Vec<Box<i32>> {
    (0..100).map(Box::new).collect()
}

/// Entry point for problem demonstration
pub fn e1709_entry() -> Result<(), Box<dyn std::error::Error>> {
    Ok(())
}

// ============================================================================
// GOOD ALTERNATIVES
// ============================================================================

/// GOOD: Return value directly
pub fn e1709_good_direct(x: i32) -> i32 {
    x
}

/// GOOD: Return tuple directly
pub fn e1709_good_tuple() -> (i32, i32) {
    (1, 2)
}

/// GOOD: Vec of values, not boxes
pub fn e1709_good_vec_values() -> Vec<i32> {
    (0..100).collect()
}

/// GOOD: Box for recursive types (necessary)
pub struct Node {
    value: i32,
    next: Option<Box<Node>>, // Box is necessary here
}

impl Node {
    pub fn new(value: i32) -> Self {
        Node { value, next: None }
    }

    pub fn append(&mut self, value: i32) {
        match &mut self.next {
            Some(next) => next.append(value),
            None => self.next = Some(Box::new(Node::new(value))),
        }
    }
}

/// GOOD: Box for trait objects (necessary)
pub fn e1709_good_trait_object() -> Box<dyn std::fmt::Display> {
    Box::new(42) // Trait object requires Box
}

/// GOOD: Box for large structs to avoid stack overflow
pub struct LargeStruct {
    data: [u8; 1024 * 1024], // 1MB
}

pub fn e1709_good_box_large() -> Box<LargeStruct> {
    // Box is appropriate for very large structs
    Box::new(LargeStruct {
        data: [0; 1024 * 1024],
    })
}

/// GOOD: Use reference instead of Box for borrowing
pub fn e1709_good_reference(data: &i32) {
    println!("{}", data);
}

/// GOOD: Use Rc/Arc for shared ownership instead of Box
pub fn e1709_good_shared() -> std::rc::Rc<i32> {
    std::rc::Rc::new(42)
}

/// GOOD: Use Cow for conditional ownership
pub fn e1709_good_cow(data: &[i32], modify: bool) -> std::borrow::Cow<'_, [i32]> {
    use std::borrow::Cow;

    if modify {
        Cow::Owned(data.iter().map(|x| x * 2).collect())
    } else {
        Cow::Borrowed(data)
    }
}

// ============================================================================
// GOOD EXAMPLES unit tests
// ============================================================================


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_direct() {
        assert_eq!(e1709_good_direct(42), 42);
    }

    #[test]
    fn test_vec_values() {
        let vec = e1709_good_vec_values();
        assert_eq!(vec.len(), 100);
    }

    #[test]
    fn test_node() {
        let mut node = Node::new(1);
        node.append(2);
        node.append(3);
        assert_eq!(node.value, 1);
        assert!(node.next.is_some());
    }

    #[test]
    fn test_trait_object() {
        let obj = e1709_good_trait_object();
        assert_eq!(format!("{}", obj), "42");
    }

    #[test]
    fn test_cow_borrowed() {
        let data = vec![1, 2, 3];
        let cow = e1709_good_cow(&data, false);
        assert!(matches!(cow, std::borrow::Cow::Borrowed(_)));
    }

    #[test]
    fn test_cow_owned() {
        let data = vec![1, 2, 3];
        let cow = e1709_good_cow(&data, true);
        assert!(matches!(cow, std::borrow::Cow::Owned(_)));
    }
}
