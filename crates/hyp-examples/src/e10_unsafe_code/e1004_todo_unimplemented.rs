/// e1004: todo!() and unimplemented!() in production code
/// Severity: HIGH
/// LLM confusion: 2 (LOW)
///
/// Description: The todo!() and unimplemented!() macros panic at runtime when executed.
/// These are placeholder macros that should be replaced with actual implementations
/// or proper error handling before deployment. LLMs frequently leave these in generated
/// code, causing runtime crashes in production.
///
/// Mitigation: Use `#![warn(clippy::todo, clippy::unimplemented)]` to catch these.
/// Replace with actual implementation or return Result/Option for incomplete features.

/// PROBLEM E1004: todo!() left in code - will panic at runtime
pub fn e1004_bad_todo_in_code(amount: f64) -> Result<(), String> {
    if amount > 1000.0 {
        todo!("implement large payment processing");
    }
    Ok(())
}

/// PROBLEM E1004: unimplemented!() in trait implementation
pub trait Serializable {
    fn serialize(&self) -> Vec<u8>;
    fn deserialize(data: &[u8]) -> Self;
}

pub struct User {
    pub name: String,
}

impl Serializable for User {
    fn serialize(&self) -> Vec<u8> {
        self.name.as_bytes().to_vec()
    }

    fn deserialize(_data: &[u8]) -> Self {
        unimplemented!("deserialization not yet implemented")
    }
}

/// PROBLEM E1004: todo!() in match arm - easy to miss
pub fn e1004_bad_todo_in_match(code: u32) -> &'static str {
    match code {
        200 => "OK",
        404 => "Not Found",
        500 => "Internal Server Error",
        _ => todo!(), // Will panic on unexpected status!
    }
}

pub fn e1004_entry() -> Result<(), Box<dyn std::error::Error>> {
    let _ = e1004_bad_todo_in_code(500.0);
    let _ = e1004_bad_todo_in_match(500);
    Ok(())
}

// ============================================================================
// GOOD EXAMPLES - Proper alternatives
// ============================================================================

use crate::test_constants::MAGIC_F64;

/// Error type for payment processing
#[derive(Debug, Clone, PartialEq)]
pub enum PaymentError {
    LargePaymentNotSupported,
}

/// GOOD: Return Result with proper error type for incomplete features
pub fn e1004_good_return_result(amount: f64) -> Result<(), PaymentError> {
    if amount > MAGIC_F64 {
        return Err(PaymentError::LargePaymentNotSupported);
    }
    Ok(())
}

/// GOOD: Use default for optional features
pub fn e1004_good_default_value(code: u32) -> &'static str {
    match code {
        200 => "OK",
        404 => "Not Found",
        500 => "Internal Server Error",
        _ => "Unknown Status",
    }
}

// ============================================================================
// GOOD EXAMPLES unit tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[should_panic]
    fn e1004_good_bad_todo_panics() {
        let _ = e1004_bad_todo_in_code(2000.0);
    }

    #[test]
    fn e1004_good_proper_error_handling() {
        assert!(e1004_good_return_result(2000.0).is_err());
    }
}
