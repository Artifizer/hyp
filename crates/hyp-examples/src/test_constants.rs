//! Common test constants used across examples
//!
//! This module provides consistent test values to avoid magic numbers
//! and ensure semantic meaning across example code.

/// Standard test integer value used in examples
pub const MAGIC_I32: i32 = 42;

/// Standard test integer value used in examples
pub const MAGIC_U32: u32 = 42;

/// Standard test integer value used in examples
pub const MAGIC_U64: u64 = 42;

/// Standard test float value used in examples
pub const MAGIC_F32: f32 = 42.0;

/// Standard test float value used in examples
pub const MAGIC_F64: f64 = 42.0;

/// Standard test string value
pub const TEST_STRING: &str = "test";

/// Standard buffer size for I/O operations in examples
pub const BUFFER_SIZE: usize = 1024;

/// Standard test float value (IEEE 754 representation of 1.0)
pub const IEEE_754_ONE: u32 = 0x3f800000;

/// Standard test float value (IEEE 754 representation of 42.0)
pub const IEEE_754_FORTY_TWO: u32 = 0x42280000;

/// Standard test array size
pub const TEST_ARRAY_SIZE: usize = 5;
