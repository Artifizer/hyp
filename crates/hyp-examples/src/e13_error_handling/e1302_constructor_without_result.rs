/// E1302: Constructors returning bare values instead of Result
/// Severity: MED
/// LLM confusion: 3 (MED)
///
/// Description: This constructor (a function that creates a new instance) can fail but returns
/// a plain value instead of a Result. This forces the constructor to panic when it fails,
/// which crashes the program. Fix by returning `Result<Self, Error>` from constructors that
/// can fail, allowing callers to handle errors gracefully.
///
/// ## The Panic Problem
///
/// ```text
/// User calls: Config::new("invalid")
///     ↓
/// Constructor tries to parse "invalid" as integer
///     ↓
/// parse().unwrap() panics!
///     ↓
/// Entire program crashes - no chance to recover
/// ```
///
/// ## Why This Matters
///
/// 1. **Crashes the program**: No opportunity for graceful error handling
/// 2. **Poor user experience**: Users see panic messages instead of helpful errors
/// 3. **Breaks the "new() should not fail" convention**: Rust convention says `new()` is infallible
/// 4. **Testing difficulties**: Can't test error paths without crashing tests
///
/// ## The Right Solutions
///
/// ### Option 1: Use try_new() that returns Result
/// ```rust
/// use std::num::ParseIntError;
///
/// struct Config { value: i32 }
///
/// impl Config {
///     pub fn try_new(s: &str) -> Result<Self, ParseIntError> {
///         let value = s.parse()?;
///         Ok(Self { value })
///     }
/// }
/// ```
///
/// ### Option 2: Make new() infallible with defaults
/// ```rust
/// struct Config { value: i32 }
///
/// impl Config {
///     pub fn new() -> Self {
///         Self { value: 0 } // Always succeeds
///     }
///
///     pub fn with_value(value: i32) -> Self {
///         Self { value }
///     }
/// }
/// ```
///
/// ### Option 3: Use the builder pattern
/// ```rust
/// struct ConfigBuilder { value: Option<i32> }
/// struct Config { value: i32 }
///
/// impl ConfigBuilder {
///     pub fn new() -> Self { Self { value: None } }
///     pub fn value(mut self, v: i32) -> Self { self.value = Some(v); self }
///     pub fn build(self) -> Result<Config, &'static str> {
///         Ok(Config { value: self.value.ok_or("value required")? })
///     }
/// }
/// ```
///
/// Mitigation: Use `#![warn(clippy::new_without_default)]` and follow the convention that `new()`
/// should never fail. For fallible construction, use `try_new()`, `from_*()`, or similar names
/// that signal the operation can fail.

// ============================================================================
// DANGEROUS PATTERNS - NEVER DO THIS
// ============================================================================

/// PROBLEM E1302: Constructor can fail but doesn't return Result
pub struct ConfigBad {
    pub value: i32,
}

impl ConfigBad {
    /// PROBLEM E1302: Constructor can fail but doesn't return Result
    pub fn new(s: &str) -> Self {
        // PROBLEM E1002: direct unwrap/expect
        let value = s.parse().unwrap(); // Can panic!
        Self { value }
    }
}

/// PROBLEM E1302: Another example with validation that panics
pub struct PortBad {
    value: u16,
}

impl PortBad {
    /// PROBLEM E1302: Panics on invalid input
    pub fn new(port: u16) -> Self {
        if port == 0 {
            panic!("Port cannot be zero"); // PROBLEM E1001: panic in production
        }
        Self { value: port }
    }
}

/// Entry point for problem demonstration
pub fn e1302_entry() -> Result<(), Box<dyn std::error::Error>> {
    let _ = ConfigBad::new("42");
    Ok(())
}

// ============================================================================
// GOOD ALTERNATIVES
// ============================================================================

/// GOOD: Constructor that returns Result
#[derive(Default)]
pub struct ConfigGood {
    pub value: i32,
}

impl ConfigGood {
    /// GOOD: Fallible constructor returns Result
    pub fn try_new(s: &str) -> Result<Self, std::num::ParseIntError> {
        let value = s.parse()?;
        Ok(Self { value })
    }

    /// GOOD: Infallible constructor for valid input
    pub fn new(value: i32) -> Self {
        Self { value }
    }
}


/// GOOD: Port with validation returning Result
#[derive(Debug, PartialEq)]
pub struct PortGood {
    value: u16,
}

#[derive(Debug, PartialEq)]
pub enum PortError {
    Zero,
    Reserved,
}

impl std::fmt::Display for PortError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PortError::Zero => write!(f, "Port cannot be zero"),
            PortError::Reserved => write!(f, "Port is in reserved range (1-1023)"),
        }
    }
}

impl std::error::Error for PortError {}

impl PortGood {
    /// GOOD: Fallible constructor with descriptive error
    pub fn try_new(port: u16) -> Result<Self, PortError> {
        if port == 0 {
            return Err(PortError::Zero);
        }
        if port < 1024 {
            return Err(PortError::Reserved);
        }
        Ok(Self { value: port })
    }

    /// GOOD: Infallible constructor for known-valid input
    pub fn new_unchecked(port: u16) -> Self {
        debug_assert!(port >= 1024, "Use try_new for validation");
        Self { value: port }
    }

    pub fn value(&self) -> u16 {
        self.value
    }
}

// ============================================================================
// GOOD EXAMPLES unit tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_good_try_new_success() {
        let config = ConfigGood::try_new("42");
        assert!(config.is_ok());
        assert_eq!(config.unwrap().value, 42);
    }

    #[test]
    fn test_config_good_try_new_failure() {
        let config = ConfigGood::try_new("not_a_number");
        assert!(config.is_err());
    }

    #[test]
    fn test_port_good_try_new_zero() {
        let port = PortGood::try_new(0);
        assert_eq!(port, Err(PortError::Zero));
    }

    #[test]
    fn test_port_good_try_new_reserved() {
        let port = PortGood::try_new(80);
        assert_eq!(port, Err(PortError::Reserved));
    }

    #[test]
    fn test_port_good_try_new_valid() {
        let port = PortGood::try_new(8080);
        assert!(port.is_ok());
        assert_eq!(port.unwrap().value(), 8080);
    }
}
