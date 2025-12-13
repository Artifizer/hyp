/// E1006: Transmute without size/alignment checks
/// Severity: HIGH
/// LLM confusion: 5 (HIGHEST)
///
/// Description: Transmute reinterprets the raw bits of one type as another type without any
/// validation. This is extremely dangerous because it bypasses all type safety - you're telling
/// the compiler "trust me, these bytes represent this other type" without checking if that's true.
/// If the types have different sizes or alignment requirements, this causes undefined behavior.
/// It's like casting a pointer to any type in C without checking - it might work, might crash,
/// or might silently corrupt data.
///
/// Mitigation: Avoid transmute unless absolutely necessary. Use safe conversion methods like
/// `from_ne_bytes()`, `to_ne_bytes()`, or implement proper conversion traits. If transmute is
/// needed, add compile-time size/alignment assertions and extensive safety documentation.
/// Use `#![warn(clippy::transmute_ptr_to_ptr)]` to catch suspicious transmutes.

#[allow(unnecessary_transmutes)] // Intentionally demonstrating the bad pattern
pub fn e1006_bad_unsafe_transmute() {
    let x: u32 = 42;

    // PROBLEM E1003: Direct use of unsafe code
    // PROBLEM E1908: No safety documentation
    // PROBLEM E1006: transmute without verifying size/alignment compatibility
    let _y: f32 = unsafe { std::mem::transmute(x) };
}

pub fn e1006_entry() -> Result<(), Box<dyn std::error::Error>> {
    e1006_bad_unsafe_transmute();
    Ok(())
}

// ============================================================================
// GOOD EXAMPLES - Proper alternatives
// ============================================================================

use crate::test_constants::IEEE_754_FORTY_TWO;
use crate::test_constants::MAGIC_F32;
use crate::test_constants::MAGIC_U32;
use crate::test_constants::MAGIC_U64;

/// GOOD: Use safe byte conversion methods
pub fn e1006_good_from_bytes() -> f32 {
    let x: u32 = IEEE_754_FORTY_TWO; // IEEE 754 representation of 42.0
    f32::from_bits(x) // Safe, explicit bit reinterpretation
}

/// GOOD: Use to_bytes/from_bytes for conversions
pub fn e1006_good_byte_conversion() -> u32 {
    let x: f32 = MAGIC_F32;
    x.to_bits() // Convert float to its bit representation
}

/// GOOD: If transmute is necessary, verify and document
pub fn e1006_good_documented_transmute() {
    let x: u32 = MAGIC_U32;

    // Compile-time assertion that sizes match
    const _: () = assert!(std::mem::size_of::<u32>() == std::mem::size_of::<f32>());

    // SAFETY (if using transmute): u32 and f32 have the same size (4 bytes) and alignment.
    // All bit patterns are valid for both types. Prefer from_bits as the safe alternative:
    let _y: f32 = f32::from_bits(x);
}

/// GOOD: Use TryFrom/TryInto for fallible conversions
pub fn e1006_good_try_from() -> Result<i32, std::num::TryFromIntError> {
    let x: u64 = MAGIC_U64;
    i32::try_from(x) // Returns Err if value doesn't fit
}

// ============================================================================
// GOOD EXAMPLES unit tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn e1006_good_from_bytes_converts_bits() {
        let value = e1006_good_from_bytes();
        assert!((value - 42.0).abs() < f32::EPSILON);
    }

    #[test]
    fn e1006_good_byte_conversion_round_trips() {
        let bits = e1006_good_byte_conversion();
        assert_eq!(bits, 0x42280000);
    }

    #[test]
    fn e1006_good_try_from_fits_in_i32() {
        let result = e1006_good_try_from();
        assert!(result.is_ok());
    }
}
