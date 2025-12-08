/// E1014: Raw pointer arithmetic without bounds checking
/// Severity: HIGH
/// LLM confusion: 4 (HIGH)
///
/// Description: Raw pointer arithmetic lets you manually calculate memory addresses by adding or
/// subtracting offsets. This is extremely unsafe because there's no bounds checking - you can
/// easily create pointers to invalid memory. Dereferencing out-of-bounds pointers causes undefined
/// behavior. It's like array indexing without bounds checks but worse because you're working with
/// raw memory addresses. Fix by using slices and safe indexing instead.
///
/// Mitigation: Avoid raw pointer arithmetic. Use slice methods like `get()`, `split_at()` instead.
/// If pointer arithmetic is necessary, extremely carefully validate bounds. Use `offset()` instead
/// of `add()` to make unsafety explicit. Test with Miri to catch UB.

pub fn e1014_bad_pointer_arithmetic() {
    let data = [1, 2, 3, 4, 5];
    let ptr = data.as_ptr();

    // PROBLEM E1003: Direct use of unsafe code
    unsafe {
        // PROBLEM E1014: Manual pointer arithmetic - no bounds checking
        let second = ptr.add(1);
        let third = ptr.add(2);

        let _val2 = *second;
        let _val3 = *third;

        // PROBLEM E1014: Easy to go out of bounds
        let maybe_invalid = ptr.add(10); // Out of bounds!
                                         // Dereferencing this would be UB
        let _ = maybe_invalid;
    }
}

// Pointer arithmetic with offset
pub fn e1014_bad_offset_arithmetic() {
    let array = [10, 20, 30, 40, 50];
    let ptr = array.as_ptr();

    // PROBLEM E1003: Direct use of unsafe code
    unsafe {
        // PROBLEM E1014: offset can easily go negative or out of bounds
        let ptr_offset = ptr.offset(2);
        let _value = *ptr_offset; // 30

        // PROBLEM E1014: Negative offset
        let ptr_back = ptr_offset.offset(-1);
        let _value_back = *ptr_back; // 20

        // PROBLEM E1014: What if we go too far?
        let dangerous = ptr.offset(100); // Way out of bounds
        let _ = dangerous;
    }
}

// PROBLEM E1014: Pointer arithmetic in a loop
pub fn e1014_bad_loop_pointer_arithmetic(data: &[i32]) -> i32 {
    let mut sum = 0;
    let ptr = data.as_ptr();

    // PROBLEM E1003: Direct use of unsafe code
    unsafe {
        // PROBLEM E1004: No safety documentation
        // PROBLEM E1014: Manual loop with pointer arithmetic instead of iterator
        for i in 0..data.len() {
            let element_ptr = ptr.add(i);
            sum += *element_ptr;
        }
    }

    sum
}

pub fn e1014_entry() -> Result<(), Box<dyn std::error::Error>> {
    e1014_bad_pointer_arithmetic();
    e1014_bad_offset_arithmetic();
    let _ = e1014_bad_loop_pointer_arithmetic(&[1, 2, 3]);
    Ok(())
}

// ============================================================================
// GOOD EXAMPLES - Proper alternatives
// ============================================================================

/// GOOD: Use slice indexing with bounds checking
pub fn e1014_slice_indexing(data: &[i32]) -> i32 {
    let mut sum = 0;
    for i in 0..data.len() {
        sum += data[i]; // Bounds-checked
    }
    sum
}

/// GOOD: Use iterators (safest and most idiomatic)
pub fn e1014_good_iterator(data: &[i32]) -> i32 {
    data.iter().sum() // No indexing at all!
}

/// GOOD: Use get() for fallible access
pub fn e1014_good_get_access(data: &[i32], index: usize) -> Option<i32> {
    data.get(index).copied() // Returns None if out of bounds
}

/// GOOD: Use split_at for safe slicing
pub fn e1014_good_split_at(data: &[i32]) -> (&[i32], &[i32]) {
    let mid = data.len() / 2;
    data.split_at(mid) // Safe, panics only if mid > len
}

/// GOOD: Use chunks for iteration over sections
pub fn e1014_good_chunks(data: &[i32]) {
    for chunk in data.chunks(2) {
        println!("Chunk: {:?}", chunk);
    }
}

/// GOOD: If pointer arithmetic is truly needed, document thoroughly
pub fn e1014_documented_pointer(data: &[i32]) -> i32 {
    if data.is_empty() {
        return 0;
    }

    let mut sum = 0;
    let ptr = data.as_ptr();
    let end = unsafe { ptr.add(data.len()) };

    let mut current = ptr;
    while current != end {
        // SAFETY: current starts at ptr and increments by 1 each iteration.
        // The loop exits when current == end, so we never dereference end.
        // All accesses are within the bounds of the slice.
        unsafe {
            sum += *current;
            current = current.add(1);
        }
    }

    sum
}

// ============================================================================
// GOOD EXAMPLES unit tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn e1014_slice_indexing_sums_values() {
        let data = [1, 2, 3, 4, 5];
        assert_eq!(e1014_slice_indexing(&data), 15);
    }

    #[test]
    fn e1014_good_iterator_collects_sum() {
        let data = [2, 4, 6];
        assert_eq!(e1014_good_iterator(&data), 12);
    }

    #[test]
    fn e1014_good_get_access_handles_oob() {
        let data = [1, 2, 3];
        assert_eq!(e1014_good_get_access(&data, 10), None);
    }

    #[test]
    fn e1014_good_split_at_splits_middle() {
        let data = [1, 2, 3, 4];
        let (left, right) = e1014_good_split_at(&data);
        assert_eq!(left, &[1, 2]);
        assert_eq!(right, &[3, 4]);
    }

    #[test]
    fn e1014_documented_pointer_matches_iterator_sum() {
        let data = [1, 2, 3, 4];
        assert_eq!(e1014_documented_pointer(&data), e1014_good_iterator(&data));
    }
}
