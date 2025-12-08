/// e1411: Type confusion with transmute
/// Severity: HIGH
/// LLM confusion: 5 (HIGHEST)
///
/// Description: Transmute reinterprets the raw bits of one type as another type without any
/// validation. This is extremely dangerous and can cause undefined behavior if the types aren't
/// compatible. It bypasses all type safety. It's like telling the computer 'treat these bytes as
/// a completely different type' without checking if that makes sense.
///
/// ## The Transmute Danger
///
/// ```text
/// let x: f32 = 1.0;
/// let y: u32 = unsafe { std::mem::transmute(x) };
/// // y now contains the bit pattern of 1.0f32 interpreted as u32
/// // This is 0x3f800000, not 1!
/// ```
///
/// ## Why This Matters
///
/// 1. **Undefined behavior**: Wrong size transmutes cause UB
/// 2. **Type safety bypass**: All type checking is disabled
/// 3. **Endianness issues**: Bit patterns vary by platform
/// 4. **Maintenance hazard**: Changes to types can break transmutes silently
///
/// ## The Right Solutions
///
/// ### Option 1: Use safe conversion methods
/// ```rust
/// let bytes: [u8; 4] = [0x12, 0x34, 0x56, 0x78];
/// let value = u32::from_ne_bytes(bytes);
/// ```
///
/// ### Option 2: Implement conversion traits
/// ```rust
/// impl From<MyType> for OtherType {
///     fn from(value: MyType) -> Self {
///         // Safe conversion logic
///     }
/// }
/// ```
///
/// ### Option 3: Use as for numeric conversions
/// ```rust
/// let x: f32 = 1.0;
/// let y: i32 = x as i32;  // Converts value, not bits
/// ```
///
/// Mitigation: Use `#![warn(clippy::transmute_ptr_to_ptr)]` and related lints. Avoid transmute
/// unless absolutely necessary. Use `from_ne_bytes()`, `to_ne_bytes()`, or implement proper
/// conversion traits. If transmute is needed, add extensive safety documentation.

// ============================================================================
// PROBLEMATIC PATTERNS
// ============================================================================

/// PROBLEM e1411: Transmuting between unrelated types
#[allow(unnecessary_transmutes)]
pub fn e1411_bad_transmute_bytes() -> u32 {
    let x: [u8; 4] = [0x12, 0x34, 0x56, 0x78];

    unsafe {
        // PROBLEM e1411: Transmuting without validation
        std::mem::transmute(x)
    }
}

/// PROBLEM e1411: Transmuting float to int (bit pattern, not value)
#[allow(clippy::unnecessary_transmute)]
#[allow(unnecessary_transmutes)]
pub fn e1411_bad_transmute_float() -> u32 {
    let x: f32 = 1.0;

    unsafe {
        // PROBLEM e1411: Gets bit pattern, not numeric value!
        std::mem::transmute(x)
    }
}

/// PROBLEM e1411: Transmuting references (dangerous)
pub fn e1411_bad_transmute_ref() -> &'static str {
    let x: &[u8] = b"hello";

    unsafe {
        // PROBLEM e1411: Transmuting references can violate invariants
        std::mem::transmute(x)
    }
}

/// Entry point for problem demonstration
pub fn e1411_entry() -> Result<(), Box<dyn std::error::Error>> {
    let _ = e1411_bad_transmute_bytes();
    let _ = e1411_bad_transmute_float();
    Ok(())
}

// ============================================================================
// GOOD ALTERNATIVES
// ============================================================================

/// GOOD: Use from_ne_bytes for byte array to integer
pub fn e1411_good_from_bytes() -> u32 {
    let bytes: [u8; 4] = [0x12, 0x34, 0x56, 0x78];
    u32::from_ne_bytes(bytes)
}

/// GOOD: Use from_le_bytes for little-endian
pub fn e1411_good_from_le_bytes() -> u32 {
    let bytes: [u8; 4] = [0x12, 0x34, 0x56, 0x78];
    u32::from_le_bytes(bytes)
}

/// GOOD: Use from_be_bytes for big-endian
pub fn e1411_good_from_be_bytes() -> u32 {
    let bytes: [u8; 4] = [0x12, 0x34, 0x56, 0x78];
    u32::from_be_bytes(bytes)
}

/// GOOD: Use to_ne_bytes for integer to byte array
pub fn e1411_good_to_bytes(value: u32) -> [u8; 4] {
    value.to_ne_bytes()
}

/// GOOD: Use as for numeric conversion (value, not bits)
pub fn e1411_good_float_to_int() -> i32 {
    let x: f32 = 1.0;
    x as i32  // Converts value: 1.0 -> 1
}

/// GOOD: Use to_bits/from_bits for float bit manipulation
pub fn e1411_good_float_bits() -> u32 {
    let x: f32 = 1.0;
    x.to_bits()  // Safe way to get bit pattern
}

pub fn e1411_good_from_bits(bits: u32) -> f32 {
    f32::from_bits(bits)  // Safe way to create float from bits
}

/// GOOD: Use proper string conversion
pub fn e1411_good_bytes_to_str(bytes: &[u8]) -> Result<&str, std::str::Utf8Error> {
    std::str::from_utf8(bytes)
}

/// GOOD: Implement From trait for custom conversions
#[derive(Debug, Clone, Copy)]
pub struct Celsius(f32);

#[derive(Debug, Clone, Copy)]
pub struct Fahrenheit(f32);

impl From<Celsius> for Fahrenheit {
    fn from(c: Celsius) -> Self {
        Fahrenheit(c.0 * 9.0 / 5.0 + 32.0)
    }
}

impl From<Fahrenheit> for Celsius {
    fn from(f: Fahrenheit) -> Self {
        Celsius((f.0 - 32.0) * 5.0 / 9.0)
    }
}

/// GOOD: Use union for type punning (still unsafe, but more explicit)
#[repr(C)]
pub union FloatBits {
    f: f32,
    bits: u32,
}

pub fn e1411_good_union_conversion(x: f32) -> u32 {
    // Still unsafe, but more explicit about what's happening
    unsafe { FloatBits { f: x }.bits }
}

// ============================================================================
// GOOD EXAMPLES unit tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_bytes() {
        let value = e1411_good_from_bytes();
        // Result depends on endianness
        assert!(value > 0);
    }

    #[test]
    fn test_from_le_bytes() {
        let bytes: [u8; 4] = [0x12, 0x34, 0x56, 0x78];
        let value = u32::from_le_bytes(bytes);
        assert_eq!(value, 0x78563412);
    }

    #[test]
    fn test_from_be_bytes() {
        let bytes: [u8; 4] = [0x12, 0x34, 0x56, 0x78];
        let value = u32::from_be_bytes(bytes);
        assert_eq!(value, 0x12345678);
    }

    #[test]
    fn test_to_bytes() {
        let value: u32 = 0x12345678;
        let bytes = e1411_good_to_bytes(value);
        assert_eq!(bytes.len(), 4);
    }

    #[test]
    fn test_float_to_int() {
        assert_eq!(e1411_good_float_to_int(), 1);
    }

    #[test]
    fn test_float_bits() {
        let bits = e1411_good_float_bits();
        let reconstructed = e1411_good_from_bits(bits);
        assert_eq!(reconstructed, 1.0);
    }

    #[test]
    fn test_bytes_to_str() {
        let bytes = b"hello";
        let result = e1411_good_bytes_to_str(bytes);
        assert_eq!(result.unwrap(), "hello");
    }

    #[test]
    fn test_temperature_conversion() {
        let c = Celsius(0.0);
        let f: Fahrenheit = c.into();
        assert!((f.0 - 32.0).abs() < 0.01);

        let f = Fahrenheit(32.0);
        let c: Celsius = f.into();
        assert!(c.0.abs() < 0.01);
    }
}
