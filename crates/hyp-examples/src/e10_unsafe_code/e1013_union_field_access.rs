/// E1013: Union with unsafe field access
/// Severity: HIGH
/// LLM confusion: 4 (HIGH)
///
/// Description: Unions allow storing different types in the same memory location, but accessing
/// the wrong field causes undefined behavior. Unlike enums, unions don't track which variant is
/// active - you must remember it yourself. Reading from the wrong union field is like interpreting
/// bytes as the wrong type. This is extremely unsafe and confusing because there's no compiler
/// help. Fix by using enums instead, or being extremely careful with manual tracking.
///
/// Mitigation: Prefer enums over unions - they're safer and the compiler tracks the active variant.
/// If you must use unions, extensively document which field is valid when. Use wrapper types that
/// track the active field. Never access union fields without unsafe blocks.

// PROBLEM E1013: Union with different types - accessing wrong field is UB
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

// PROBLEM E1013: Manual tracking of union variant - error-prone
pub struct TaggedValue {
    tag: ValueTag,
    value: Value,
}

impl TaggedValue {
    pub fn init_union_with_int(val: i32) -> Self {
        TaggedValue {
            tag: ValueTag::Int,
            value: Value { int: val },
        }
    }

    pub fn init_union_with_float(val: f32) -> Self {
        TaggedValue {
            tag: ValueTag::Float,
            value: Value { float: val },
        }
    }

    // PROBLEM E1013: Easy to call get_int when value is actually float
    pub unsafe fn e1013_bad_get_int(&self) -> i32 {
        self.value.int
    }

    pub unsafe fn e1013_bad_get_float(&self) -> f32 {
        self.value.float
    }
}

// PROBLEM E1013: Transmuting between union fields
pub fn e1013_bad_union_transmute() {
    let val = Value {
        int: 0x3f80_0000_u32 as i32,
    };

    // PROBLEM E1003: Direct use of unsafe code
    unsafe {
        // PROBLEM E1004: No safety documentation
        // PROBLEM E1013: Reading float interpretation of int bits
        let as_float = val.float;
        let _ = as_float; // Should be 1.0
    }
}

pub fn e1013_entry() -> Result<(), Box<dyn std::error::Error>> {
    let tagged = TaggedValue::init_union_with_int(10);
    unsafe {
        let _ = tagged.e1013_bad_get_int();
    }
    e1013_bad_union_transmute();
    Ok(())
}

// ============================================================================
// GOOD EXAMPLES - Proper alternatives
// ============================================================================

use crate::test_constants::IEEE_754_ONE;

/// GOOD: Use an enum instead of a union - compiler tracks active variant
#[derive(Debug, Clone)]
pub enum SafeValue {
    Int(i32),
    Float(f32),
    Bytes([u8; 4]),
}

impl SafeValue {
    pub fn e1013_good_as_int(&self) -> Option<i32> {
        match self {
            SafeValue::Int(v) => Some(*v),
            _ => None,
        }
    }

    pub fn e1013_good_as_float(&self) -> Option<f32> {
        match self {
            SafeValue::Float(v) => Some(*v),
            _ => None,
        }
    }
}

/// GOOD: For bit reinterpretation, use explicit safe methods
pub fn e1013_good_bit_conversion() {
    let int_bits: u32 = IEEE_754_ONE;

    // Safe way to reinterpret bits
    let as_float = f32::from_bits(int_bits);
    println!("Float value: {}", as_float); // 1.0

    // Convert back
    let back_to_bits = as_float.to_bits();
    assert_eq!(int_bits, back_to_bits);
}

/// GOOD: If union is required, encapsulate safely
pub struct SafeTaggedValue {
    tag: ValueTag,
    value: Value,
}

impl SafeTaggedValue {
    pub fn e1013_new_int(val: i32) -> Self {
        SafeTaggedValue {
            tag: ValueTag::Int,
            value: Value { int: val },
        }
    }

    pub fn e1013_new_float(val: f32) -> Self {
        SafeTaggedValue {
            tag: ValueTag::Float,
            value: Value { float: val },
        }
    }

    /// Safe accessor - checks tag before accessing union
    pub fn e1013_get_int(&self) -> Option<i32> {
        match self.tag {
            ValueTag::Int => {
                // SAFETY: We verified the tag is Int
                Some(unsafe { self.value.int })
            }
            _ => None,
        }
    }

    pub fn e1013_get_float(&self) -> Option<f32> {
        match self.tag {
            ValueTag::Float => {
                // SAFETY: We verified the tag is Float
                Some(unsafe { self.value.float })
            }
            _ => None,
        }
    }
}

// ============================================================================
// GOOD EXAMPLES unit tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn e1013_good_safe_value_accessors_work() {
        let val = SafeValue::Int(42);
        assert_eq!(val.e1013_good_as_int(), Some(42));
        assert_eq!(val.e1013_good_as_float(), None);
    }


    #[test]
    fn e1013_good_bit_conversion_round_trips() {
        e1013_good_bit_conversion();
    }
}
