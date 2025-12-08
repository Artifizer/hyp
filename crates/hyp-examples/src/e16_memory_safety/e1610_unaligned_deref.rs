/// E1610: Unaligned dereference
/// Severity: HIGH
/// LLM confusion: 4 (HIGH)
///
/// Description: Dereferencing an unaligned pointer causes undefined behavior on many architectures.
/// A u64 typically requires 8-byte alignment, but this code creates a u64 pointer at offset 1
/// (not aligned to 8 bytes). Reading from it can crash or return garbage. Fix by ensuring pointers
/// are properly aligned, or use `read_unaligned()` for unaligned access.
///
/// ## The Alignment Problem
///
/// ```text
/// let data = [0u8; 10];
/// let ptr = data.as_ptr().add(1);  // Offset 1, not aligned to 8
/// let ptr64 = ptr as *const u64;
///
/// unsafe { *ptr64 }  // UB! u64 requires 8-byte alignment
/// ```
///
/// ## Why This Matters
///
/// 1. **Crashes**: SIGBUS on some architectures
/// 2. **Undefined behavior**: Wrong values, corruption
/// 3. **Performance**: Even when it "works", it's slow
/// 4. **Architecture-dependent**: Works on x86, crashes on ARM
///
/// ## The Right Solutions
///
/// ### Option 1: Use read_unaligned for potentially unaligned data
/// ```rust
/// let data = [0u8; 10];
/// let ptr = data.as_ptr().add(1) as *const u64;
///
/// unsafe {
///     let value = ptr.read_unaligned();  // Safe!
/// }
/// ```
///
/// ### Option 2: Ensure proper alignment
/// ```rust
/// #[repr(align(8))]
/// struct AlignedData([u8; 16]);
///
/// let data = AlignedData([0; 16]);
/// // Now aligned for u64 access
/// ```
///
/// ### Option 3: Use byte-by-byte access
/// ```rust
/// fn read_u64_le(bytes: &[u8]) -> u64 {
///     u64::from_le_bytes(bytes[..8].try_into().unwrap())
/// }
/// ```
///
/// Mitigation: Use `ptr.read_unaligned()` for potentially unaligned reads. Check alignment with
/// `ptr.align_offset()`. Use `#[repr(packed)]` carefully as it creates unaligned fields. Understand
/// your target architecture's alignment requirements.

// ============================================================================
// DANGEROUS PATTERNS - NEVER DO THIS
// ============================================================================

/// PROBLEM E1610: Unaligned dereference
pub fn e1610_bad_unaligned() {
    let data = [0u8; 10];
    let ptr = data.as_ptr();

    // PROBLEM E1003: Direct use of unsafe code
    unsafe {
        // PROBLEM E1004: No safety documentation
        // PROBLEM E1610: Casting to u64 pointer may not be aligned
        let ptr64 = ptr.add(1) as *const u64;
        let _value = *ptr64;
    }
}

/// PROBLEM E1610: Packed struct field access
#[repr(packed)]
struct PackedStruct {
    a: u8,
    b: u64, // Unaligned!
}

pub fn e1610_bad_packed() {
    let packed = PackedStruct { a: 1, b: 42 };

    // PROBLEM E1610: Taking reference to unaligned field
    // let _ref = &packed.b;  // This would be UB!

    // Even reading directly can be problematic
    let _value = packed.b; // May work on x86, UB on other archs
}

/// Entry point for problem demonstration
pub fn e1610_entry() -> Result<(), Box<dyn std::error::Error>> {
    Ok(())
}

// ============================================================================
// GOOD ALTERNATIVES
// ============================================================================

/// GOOD: Use read_unaligned
pub fn e1610_good_read_unaligned() {
    let data = [1u8, 2, 3, 4, 5, 6, 7, 8, 9, 10];
    let ptr = data.as_ptr();

    unsafe {
        let ptr64 = ptr.add(1) as *const u64;
        let value = ptr64.read_unaligned(); // Safe!
        println!("Value: {}", value);
    }
}

/// GOOD: Use from_le_bytes for portable byte access
pub fn e1610_good_from_bytes(bytes: &[u8]) -> Option<u64> {
    if bytes.len() >= 8 {
        Some(u64::from_le_bytes(bytes[..8].try_into().unwrap()))
    } else {
        None
    }
}

/// GOOD: Use repr(C) with proper alignment
#[repr(C)]
struct AlignedStruct {
    a: u8,
    _padding: [u8; 7], // Explicit padding
    b: u64,            // Now aligned!
}

pub fn e1610_good_aligned_struct() {
    let aligned = AlignedStruct {
        a: 1,
        _padding: [0; 7],
        b: 42,
    };

    let _ref = &aligned.b; // Safe - properly aligned
}

/// GOOD: Check alignment before access
pub fn e1610_good_check_alignment(ptr: *const u8) -> Option<u64> {
    let ptr64 = ptr as *const u64;

    // Check if aligned
    if ptr64.align_offset(std::mem::align_of::<u64>()) == 0 {
        Some(unsafe { *ptr64 })
    } else {
        // Fall back to unaligned read
        Some(unsafe { ptr64.read_unaligned() })
    }
}

/// GOOD: Use bytemuck for safe transmutes
#[cfg(all(feature = "bytemuck", not(feature = "bytemuck")))]
pub fn e1610_good_bytemuck() {
    use bytemuck::{Pod, Zeroable};

    #[derive(Copy, Clone, Pod, Zeroable)]
    #[repr(C)]
    struct Data {
        value: u64,
    }

    let bytes = [0u8; 8];
    let data: Data = bytemuck::pod_read_unaligned(&bytes);
    println!("{}", data.value);
}

/// GOOD: Use zerocopy for safe byte reinterpretation
pub fn e1610_good_manual_read(bytes: &[u8], offset: usize) -> Option<u64> {
    if offset + 8 > bytes.len() {
        return None;
    }

    let slice = &bytes[offset..offset + 8];
    Some(u64::from_ne_bytes(slice.try_into().unwrap()))
}

// ============================================================================
// GOOD EXAMPLES unit tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_read_unaligned() {
        e1610_good_read_unaligned();
    }

    #[test]
    fn test_from_bytes() {
        let bytes = [1, 0, 0, 0, 0, 0, 0, 0];
        assert_eq!(e1610_good_from_bytes(&bytes), Some(1));
    }

    #[test]
    fn test_from_bytes_too_short() {
        let bytes = [1, 2, 3];
        assert_eq!(e1610_good_from_bytes(&bytes), None);
    }

    #[test]
    fn test_manual_read() {
        let bytes = [0, 1, 0, 0, 0, 0, 0, 0, 0];
        let value = e1610_good_manual_read(&bytes, 1);
        assert!(value.is_some());
    }
}
