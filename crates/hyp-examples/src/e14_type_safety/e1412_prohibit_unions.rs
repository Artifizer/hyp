/// E1412: Union types prohibited
/// Severity: HIGH
/// LLM confusion: 4 (HIGH)
///
/// Description: Unions allow type confusion where you store one type and read another,
/// causing undefined behavior. Unlike enums, unions don't track which variant is active,
/// making it trivially easy to read the wrong field. This is fundamentally unsafe because
/// interpreting memory as the wrong type violates Rust's type system guarantees.
///
/// ## Why This Matters
///
/// 1. **Type confusion**: Store an i32, read it as f32 - complete garbage
/// 2. **No compiler help**: Unlike enums, no tracking of active variant
/// 3. **Undefined behavior**: Reading wrong field is instant UB
/// 4. **Memory reinterpretation**: Bits are reinterpreted as different types
/// 5. **Hard to review**: Manual tag tracking is error-prone
///
/// ## The Right Solutions
///
/// ### Option 1: Use enums (recommended)
/// ```rust
/// enum Value {
///     Int(i32),
///     Float(f32),
/// }
/// // Compiler tracks which variant is active
/// ```
///
/// ### Option 2: Use safe bit conversion
/// ```rust
/// let float_bits = my_float.to_bits();
/// let back = f32::from_bits(float_bits);
/// ```
///
/// ### Option 3: Use transmute (only if absolutely necessary)
/// ```rust
/// // Document extensively why this is safe
/// // SAFETY: Both types are same size and alignment
/// let converted = unsafe { std::mem::transmute::<T, U>(value) };
/// ```
///
/// Mitigation: Ban unions entirely. Use enums which provide type-safe tagged unions
/// with compiler-enforced variant tracking. For bit manipulation, use safe methods
/// like `to_bits()` and `from_bits()`.

// ============================================================================
// PROBLEMATIC PATTERNS
// ============================================================================

/// PROBLEM E1412: Union definition - enables type confusion
#[repr(C)]
pub union Value {
    int: i32,
    float: f32,
    bytes: [u8; 4],
}

pub enum ValueTag {
    Int,
    Float,
    Bytes,
}

/// PROBLEM E1412: Manual tag tracking with union
pub struct TaggedValue {
    tag: ValueTag,
    value: Value,
}

impl TaggedValue {
    /// PROBLEM E1412: Creating union instance
    pub fn e1412_bad_new_int(val: i32) -> Self {
        TaggedValue {
            tag: ValueTag::Int,
            value: Value { int: val },
        }
    }

    /// PROBLEM E1412: Creating union instance
    pub fn e1412_bad_new_float(val: f32) -> Self {
        TaggedValue {
            tag: ValueTag::Float,
            value: Value { float: val },
        }
    }

    /// PROBLEM E1412: Accessing union field - type confusion risk
    pub unsafe fn e1412_bad_get_int(&self) -> i32 {
        self.value.int
    }

    /// PROBLEM E1412: Accessing union field - type confusion risk
    pub unsafe fn e1412_bad_get_float(&self) -> f32 {
        self.value.float
    }
}

/// PROBLEM E1412: Union transmutation - reading wrong field
pub fn e1412_bad_type_confusion() {
    let val = Value { int: 42 };

    unsafe {
        // Store as int, read as float - complete garbage!
        let as_float = val.float;
        println!("Int 42 as float: {} (garbage!)", as_float);
    }
}

/// Entry point for problem demonstration.
pub fn e1412_entry() -> Result<(), Box<dyn std::error::Error>> {
    let tagged = TaggedValue::e1412_bad_new_int(42);
    unsafe {
        let _ = tagged.e1412_bad_get_int();
    }
    e1412_bad_type_confusion();
    Ok(())
}

// ============================================================================
// GOOD ALTERNATIVES
// ============================================================================

/// GOOD: Use enum instead of union - compiler tracks variant
#[derive(Debug, Clone)]
pub enum SafeValue {
    Int(i32),
    Float(f32),
    Bytes([u8; 4]),
}

impl SafeValue {
    /// GOOD: Type-safe accessor
    pub fn e1412_good_as_int(&self) -> Option<i32> {
        match self {
            SafeValue::Int(v) => Some(*v),
            _ => None,
        }
    }

    /// GOOD: Type-safe accessor
    pub fn e1412_good_as_float(&self) -> Option<f32> {
        match self {
            SafeValue::Float(v) => Some(*v),
            _ => None,
        }
    }
}

/// GOOD: For bit reinterpretation, use safe methods
pub fn e1412_good_bit_conversion() {
    let my_float: f32 = 1.5;

    // Safe way to get bit representation
    let bits = my_float.to_bits();
    println!("Float {} as bits: 0x{:08x}", my_float, bits);

    // Safe way to convert back
    let back = f32::from_bits(bits);
    assert_eq!(my_float, back);
}

/// GOOD: If you need C interop, keep union private and wrap safely
mod safe_c_interop {
    #[repr(C)]
    union CValue {
        int_val: i32,
        float_val: f32,
    }

    pub enum ValueType {
        Int,
        Float,
    }

    pub struct SafeCValue {
        value: CValue,
        value_type: ValueType,
    }

    impl SafeCValue {
        pub fn e1412_good_new_int(val: i32) -> Self {
            SafeCValue {
                value: CValue { int_val: val },
                value_type: ValueType::Int,
            }
        }

        pub fn e1412_good_get_int(&self) -> Option<i32> {
            match self.value_type {
                ValueType::Int => Some(unsafe { self.value.int_val }),
                _ => None,
            }
        }
    }
}

pub use safe_c_interop::SafeCValue;

// ============================================================================
// GOOD EXAMPLES unit tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn e1412_good_safe_value_works() {
        let val = SafeValue::Int(42);
        assert_eq!(val.e1412_good_as_int(), Some(42));
        assert_eq!(val.e1412_good_as_float(), None);
    }

    #[test]
    fn e1412_good_bit_conversion_round_trips() {
        e1412_good_bit_conversion();
    }

    #[test]
    fn e1412_good_safe_c_value_checks_type() {
        let val = SafeCValue::e1412_good_new_int(100);
        assert_eq!(val.e1412_good_get_int(), Some(100));
    }
}
