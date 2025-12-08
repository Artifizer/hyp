/// E1701: Oversized struct passed by value
/// Severity: LOW
/// LLM confusion: 2 (LOW)
///
/// Description: Passing large structs by value (not by reference) copies all the data, which is
/// slow and uses extra stack space. This struct is 4KB, so every function call copies 4KB of data.
/// Fix by passing large structs by reference (`&OversizedStruct`) instead of by value, or use
/// `Box<OversizedStruct>` to pass a pointer.
///
/// ## The Copy Cost
///
/// ```text
/// struct Big { data: [u8; 4096] }  // 4KB
///
/// fn process(s: Big) { ... }  // Copies 4KB on every call!
///
/// let big = Big { data: [0; 4096] };
/// process(big);  // 4KB copied
/// process(big);  // Another 4KB copied (if it were Copy)
/// ```
///
/// ## Why This Matters
///
/// 1. **Stack usage**: Large values can overflow the stack
/// 2. **Performance**: Copying is proportional to size
/// 3. **Cache pollution**: Large copies evict useful data from cache
/// 4. **Memory bandwidth**: Wastes memory bandwidth on copies
///
/// ## The Right Solutions
///
/// ### Option 1: Pass by reference
/// ```rust
/// struct Big { data: [u8; 4096] }
///
/// fn process(s: &Big) {  // Only copies 8 bytes (pointer)
///     println!("{}", s.data[0]);
/// }
/// ```
///
/// ### Option 2: Use Box for heap allocation
/// ```rust
/// fn process(s: Box<Big>) {  // Only copies 8 bytes (pointer)
///     println!("{}", s.data[0]);
/// }
/// ```
///
/// ### Option 3: Use Arc for shared ownership
/// ```rust
/// use std::sync::Arc;
///
/// fn process(s: Arc<Big>) {  // Only copies 8 bytes + atomic increment
///     println!("{}", s.data[0]);
/// }
/// ```
///
/// Mitigation: Use `#![warn(clippy::large_types_passed_by_value)]` to detect large types passed
/// by value. Pass structs larger than ~128 bytes by reference. Use `Box` for very large types.
/// Consider if the struct needs to be that large - maybe split it up.

// ============================================================================
// PROBLEMATIC PATTERNS
// ============================================================================

/// Large struct - 4KB
pub struct OversizedStruct {
    // PROBLEM E1701: Large struct (> 1KB) passed by value
    pub data: [u8; 2048],
    pub more_data: [u8; 2048],
}

/// PROBLEM E1701: Passing large struct by value
pub fn e1701_bad_by_value(s: OversizedStruct) {
    // Copies 4KB on every call!
    println!("First byte: {}", s.data[0]);
}

/// PROBLEM E1701: Returning large struct by value
pub fn e1701_bad_return_by_value() -> OversizedStruct {
    OversizedStruct {
        data: [0; 2048],
        more_data: [0; 2048],
    }
}

/// PROBLEM E1701: Large struct in loop
pub fn e1701_bad_in_loop(structs: Vec<OversizedStruct>) {
    for s in structs {
        // Each iteration copies 4KB!
        e1701_bad_by_value(s);
    }
}

/// Entry point for problem demonstration
pub fn e1701_entry() -> Result<(), Box<dyn std::error::Error>> {
    Ok(())
}

// ============================================================================
// GOOD ALTERNATIVES
// ============================================================================

/// GOOD: Pass by reference
pub fn e1701_good_by_reference(s: &OversizedStruct) {
    // Only copies 8 bytes (pointer)
    println!("First byte: {}", s.data[0]);
}

/// GOOD: Pass by mutable reference
pub fn e1701_good_by_mut_ref(s: &mut OversizedStruct) {
    s.data[0] = 42;
}

/// GOOD: Use Box for heap allocation
pub fn e1701_good_boxed(s: Box<OversizedStruct>) {
    // Box is just a pointer - 8 bytes
    println!("First byte: {}", s.data[0]);
}

/// GOOD: Return Box instead of value
pub fn e1701_good_return_boxed() -> Box<OversizedStruct> {
    Box::new(OversizedStruct {
        data: [0; 2048],
        more_data: [0; 2048],
    })
}

/// GOOD: Iterate by reference
pub fn e1701_good_iter_ref(structs: &[OversizedStruct]) {
    for s in structs {
        // No copy - just borrowing
        e1701_good_by_reference(s);
    }
}

/// GOOD: Use Arc for shared ownership
pub fn e1701_good_arc(s: std::sync::Arc<OversizedStruct>) {
    // Arc clone is cheap (atomic increment)
    println!("First byte: {}", s.data[0]);
}

/// GOOD: Split large struct into smaller parts
pub struct SplitStruct {
    pub header: Header,
    pub body: Box<Body>,
}

pub struct Header {
    pub id: u64,
    pub flags: u32,
}

pub struct Body {
    pub data: [u8; 4096],
}

pub fn e1701_good_split(s: &SplitStruct) {
    // Header is small, can be copied if needed
    // Body is boxed, only pointer is passed
    println!("ID: {}, First byte: {}", s.header.id, s.body.data[0]);
}

// ============================================================================
// GOOD EXAMPLES unit tests
// ============================================================================


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_by_reference() {
        let s = OversizedStruct {
            data: [1; 2048],
            more_data: [2; 2048],
        };
        e1701_good_by_reference(&s);
    }

    #[test]
    fn test_boxed() {
        let s = e1701_good_return_boxed();
        e1701_good_boxed(s);
    }

    #[test]
    fn test_iter_ref() {
        let structs = vec![
            OversizedStruct {
                data: [0; 2048],
                more_data: [0; 2048],
            },
            OversizedStruct {
                data: [1; 2048],
                more_data: [1; 2048],
            },
        ];
        e1701_good_iter_ref(&structs);
    }
}
