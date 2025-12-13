/// E1017: Prohibit std::mem::transmute unconditionally
/// Severity: HIGH
/// LLM confusion: 5 (HIGHEST)
///
/// Description: `std::mem::transmute` is one of the most dangerous operations in Rust,
/// allowing arbitrary reinterpretation of bits from one type to another without any safety
/// checks whatsoever. It completely bypasses Rust's type system and can lead to undefined
/// behavior, memory corruption, and security vulnerabilities. Even when used with proper
/// documentation and size/alignment checks, transmute is fragile and becomes unsafe when
/// code evolves. This checker takes a zero-tolerance approach - ALL transmute usage is prohibited.
///
/// ## Why This Matters
///
/// 1. **Undefined behavior**: Wrong assumptions about bit patterns cause crashes or corruption
/// 2. **Security vulnerabilities**: Type confusion bugs can be exploited by attackers
/// 3. **Maintenance hazards**: "Safe" transmutes become unsafe when types change
/// 4. **Type system bypass**: Violates Rust's fundamental safety guarantees
/// 5. **Better alternatives exist**: Safe conversion methods work for 99% of use cases
///
/// ## The Right Solutions
///
/// ### Option 1: Use dedicated conversion methods
/// ```rust
/// // Instead of transmute for bit reinterpretation
/// let bits: u32 = 0x42280000;
/// let f = f32::from_bits(bits);  // Safe!
/// let back = f.to_bits();         // Safe!
/// ```
///
/// ### Option 2: Use TryFrom for fallible conversions
/// ```rust
/// use std::convert::TryFrom;
///
/// let big: u64 = 42;
/// let small = i32::try_from(big)?;  // Returns Err if doesn't fit
/// ```
///
/// ### Option 3: Implement proper conversion traits
/// ```rust
/// impl From<SourceType> for TargetType {
///     fn from(value: SourceType) -> Self {
///         // Proper field-by-field conversion
///         TargetType {
///             field1: value.field1,
///             field2: value.field2,
///         }
///     }
/// }
/// ```
///
/// ### Option 4: Use byte slices for serialization
/// ```rust
/// // Instead of transmuting to bytes
/// let value: u32 = 42;
/// let bytes = value.to_ne_bytes();  // Safe!
/// let back = u32::from_ne_bytes(bytes);  // Safe!
/// ```
///
/// Mitigation: Enable E1017 to completely prohibit transmute. Use E1006 if you need
/// conditional checking. Use `#![forbid(unsafe_code)]` to prevent all unsafe operations.

// ============================================================================
// PROBLEMATIC PATTERNS
// ============================================================================

/// PROBLEM E1017: Even "documented" transmute is prohibited
#[allow(unnecessary_transmutes)] // We're demonstrating the bad pattern
pub fn e1017_bad_transmute_with_checks() {
    let x: u32 = 42;

    // Even with compile-time checks and documentation, transmute is prohibited
    const _: () = assert!(std::mem::size_of::<u32>() == std::mem::size_of::<f32>());

    // PROBLEM E1003: Direct use of unsafe code
    // PROBLEM E1904: No safety documentation
    // PROBLEM E1017: transmute is completely prohibited
    let _y: f32 = unsafe { std::mem::transmute(x) };
}

/// PROBLEM E1017: Transmute for type confusion
#[allow(unnecessary_transmutes)]
pub fn e1017_bad_type_confusion() {
    #[repr(C)]
    struct TypeA {
        x: u32,
        y: u32,
    }

    #[repr(C)]
    struct TypeB {
        a: u64,
    }

    let type_a = TypeA { x: 1, y: 2 };

    // PROBLEM E1017: Transmute between different struct types
    let _type_b: TypeB = unsafe { std::mem::transmute(type_a) };
}

/// PROBLEM E1017: Transmute with core::mem
#[allow(unnecessary_transmutes)]
pub fn e1017_bad_core_transmute() {
    let x: u32 = 42;
    // PROBLEM E1017: core::mem::transmute is also prohibited
    let _y: f32 = unsafe { core::mem::transmute(x) };
}

/// PROBLEM E1017: Imported transmute
#[allow(unnecessary_transmutes)]
pub fn e1017_bad_imported_transmute() {
    use std::mem::transmute;

    let x: u32 = 42;
    // PROBLEM E1017: Even imported transmute is caught
    let _y: f32 = unsafe { transmute(x) };
}

/// Entry point for problem demonstration
pub fn e1017_entry() -> Result<(), Box<dyn std::error::Error>> {
    // Don't call the bad examples - they demonstrate prohibited patterns
    // The good examples show safe alternatives
    let _ = e1017_good_from_bits();
    let _ = e1017_good_to_bits();
    let _ = e1017_try_from();
    let _ = e1017_good_byte_conversion();
    Ok(())
}

// ============================================================================
// GOOD ALTERNATIVES
// ============================================================================

use crate::test_constants::IEEE_754_FORTY_TWO;
use crate::test_constants::MAGIC_F32;
use crate::test_constants::MAGIC_I32;
use crate::test_constants::MAGIC_U32;

/// GOOD: Use from_bits for bit reinterpretation
pub fn e1017_good_from_bits() -> f32 {
    let x: u32 = IEEE_754_FORTY_TWO; // IEEE 754 representation of 42.0
    f32::from_bits(x) // Safe, explicit, and clear!
}

/// GOOD: Use to_bits for reverse conversion
pub fn e1017_good_to_bits() -> u32 {
    let x: f32 = MAGIC_F32;
    x.to_bits() // Safe!
}

/// GOOD: Use TryFrom for fallible conversions
pub fn e1017_try_from() -> Result<i32, std::num::TryFromIntError> {
    let x: u64 = MAGIC_I32 as u64;
    i32::try_from(x) // Returns Err if value doesn't fit
}

/// GOOD: Use byte conversion methods
pub fn e1017_good_byte_conversion() -> u32 {
    let value: u32 = MAGIC_U32;
    let bytes = value.to_ne_bytes(); // Safe serialization
    u32::from_ne_bytes(bytes) // Safe deserialization
}

/// GOOD: Implement proper conversion traits
pub fn e1017_good_implement_from() {
    struct Source {
        x: i32,
        y: i32,
    }
    struct Target {
        x: i32,
        y: i32,
    }

    impl From<Source> for Target {
        fn from(value: Source) -> Self {
            Target {
                x: value.x,
                y: value.y,
            }
        }
    }

    let source = Source { x: 1, y: 2 };
    let _target = Target::from(source); // Safe and clear!
}

/// GOOD: Use as for simple numeric conversions
pub fn e1017_good_as_conversion() -> f32 {
    let x: i32 = MAGIC_I32;
    x as f32 // Safe for numeric types
}

/// GOOD: Use cast crate for more complex conversions
pub fn e1017_good_bytemuck() {
    // For zero-copy conversions with proper validation:
    // use bytemuck::{Pod, Zeroable};
    // bytemuck::cast<u32, f32>(value)  // With safety traits
}

// ============================================================================
// GOOD EXAMPLES unit tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_bits_conversion() {
        let value = e1017_good_from_bits();
        assert!((value - 42.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_to_bits_conversion() {
        let bits = e1017_good_to_bits();
        assert_eq!(bits, 0x42280000);
    }

    #[test]
    fn test_try_from_success() {
        let result = e1017_try_from();
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 42);
    }

    #[test]
    fn test_byte_conversion_round_trip() {
        let result = e1017_good_byte_conversion();
        assert_eq!(result, 42);
    }

    #[test]
    fn test_as_conversion() {
        let result = e1017_good_as_conversion();
        assert_eq!(result, 42.0);
    }
}
