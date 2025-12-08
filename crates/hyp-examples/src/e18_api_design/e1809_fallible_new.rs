/// E1809: Fallible new()
/// Severity: MED
/// LLM confusion: 2 (LOW)
///
/// Description: By convention, `new()` should be infallible (never fail). If construction can
/// fail, use a different name like `try_new()`, `from_*()`, or `with_*()`. This `new()` panics
/// on invalid input, which violates user expectations. Fix by returning `Result` and renaming
/// to `try_new()` or similar.
///
/// ## The Unexpected Panic Problem
///
/// ```text
/// impl Config {
///     pub fn new(path: &str) -> Self {
///         let content = std::fs::read_to_string(path).unwrap();  // Panics!
///         // ...
///     }
/// }
///
/// let config = Config::new("missing.toml");  // Crashes!
/// ```
///
/// ## Why This Matters
///
/// 1. **Unexpected panics**: Users expect new() to succeed
/// 2. **Hard to handle**: Can't recover from panics easily
/// 3. **Convention violation**: Breaks Rust idioms
/// 4. **Testing difficulty**: Panics make testing harder
///
/// ## The Right Solutions
///
/// ### Option 1: Use try_new() for fallible construction
/// ```rust
/// impl Config {
///     pub fn try_new(path: &str) -> Result<Self, Error> {
///         let content = std::fs::read_to_string(path)?;
///         // ...
///         Ok(Self { ... })
///     }
/// }
/// ```
///
/// ### Option 2: Use from_*() for conversion
/// ```rust
/// impl Config {
///     pub fn from_file(path: &Path) -> Result<Self, Error> { ... }
///     pub fn from_str(s: &str) -> Result<Self, Error> { ... }
/// }
/// ```
///
/// ### Option 3: Use builder pattern
/// ```rust
/// let config = ConfigBuilder::new()
///     .path("config.toml")
///     .build()?;
/// ```
///
/// Mitigation: Make `new()` infallible - it should always succeed. Use `try_new()` or `from_*()`
/// for fallible construction that returns `Result`. Use `default()` for default construction.
/// Follow Rust API conventions to meet user expectations.

// ============================================================================
// PROBLEMATIC PATTERNS
// ============================================================================

/// PROBLEM E1809: new() panics on invalid input
pub struct BadFallibleNew {
    value: String,
}

impl BadFallibleNew {
    // PROBLEM E1809: new() should not fail, use try_new() or from_*()
    pub fn new(value: &str) -> Self {
        if value.is_empty() {
            print!("going to panic now");
            panic!("Value cannot be empty");
        }
        Self {
            value: value.to_string(),
        }
    }
}

/// PROBLEM E1809: new() does I/O that can fail
pub struct BadFileConfig {
    content: String,
}

impl BadFileConfig {
    // PROBLEM: I/O in new() can fail
    pub fn new(path: &str) -> Self {
        let content = std::fs::read_to_string(path).expect("Failed to read file");
        Self { content }
    }
}

/// Entry point for problem demonstration
pub fn e1809_entry() -> Result<(), Box<dyn std::error::Error>> {
    // Don't actually call these - they would panic
    Ok(())
}

// ============================================================================
// GOOD ALTERNATIVES
// ============================================================================

/// GOOD: Infallible new() with try_new() for fallible construction
pub struct GoodTryNew {
    value: String,
}

impl GoodTryNew {
    /// Creates a new instance with a default value.
    /// This always succeeds.
    pub fn new() -> Self {
        Self {
            value: String::new(),
        }
    }

    /// Tries to create a new instance with the given value.
    /// Returns an error if the value is empty.
    pub fn try_new(value: &str) -> Result<Self, &'static str> {
        if value.is_empty() {
            return Err("Value cannot be empty");
        }
        Ok(Self {
            value: value.to_string(),
        })
    }

    /// Creates from a validated value.
    pub fn with_value(value: String) -> Self {
        Self { value }
    }

    pub fn value(&self) -> &str {
        &self.value
    }
}

impl Default for GoodTryNew {
    fn default() -> Self {
        Self::new()
    }
}

/// GOOD: Use from_*() for fallible conversions
pub struct GoodFromPattern {
    content: String,
}

impl GoodFromPattern {
    /// Creates an empty config. Always succeeds.
    pub fn new() -> Self {
        Self {
            content: String::new(),
        }
    }

    /// Creates from a file path. Can fail.
    pub fn from_file(path: &str) -> Result<Self, std::io::Error> {
        let content = std::fs::read_to_string(path)?;
        Ok(Self { content })
    }

    /// Creates from a string. Can fail.
    pub fn from_str(s: &str) -> Result<Self, &'static str> {
        if s.is_empty() {
            return Err("Content cannot be empty");
        }
        Ok(Self {
            content: s.to_string(),
        })
    }

    /// Creates from bytes. Can fail.
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, std::str::Utf8Error> {
        let content = std::str::from_utf8(bytes)?;
        Ok(Self {
            content: content.to_string(),
        })
    }

    pub fn content(&self) -> &str {
        &self.content
    }
}

impl Default for GoodFromPattern {
    fn default() -> Self {
        Self::new()
    }
}

/// GOOD: Builder pattern for complex construction
#[derive(Default)]
pub struct ConfigBuilder {
    path: Option<String>,
    content: Option<String>,
    validate: bool,
}

impl ConfigBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn path(mut self, path: &str) -> Self {
        self.path = Some(path.to_string());
        self
    }

    pub fn content(mut self, content: &str) -> Self {
        self.content = Some(content.to_string());
        self
    }

    pub fn validate(mut self, validate: bool) -> Self {
        self.validate = validate;
        self
    }

    pub fn build(self) -> Result<Config, &'static str> {
        let content = if let Some(path) = self.path {
            std::fs::read_to_string(&path).map_err(|_| "Failed to read file")?
        } else if let Some(content) = self.content {
            content
        } else {
            return Err("Either path or content must be provided");
        };

        if self.validate && content.is_empty() {
            return Err("Content cannot be empty");
        }

        Ok(Config { content })
    }
}

pub struct Config {
    content: String,
}

impl Config {
    pub fn content(&self) -> &str {
        &self.content
    }
}

/// GOOD: Newtype with validated construction
pub struct NonEmptyString(String);

impl NonEmptyString {
    /// Tries to create a non-empty string.
    pub fn new(s: &str) -> Option<Self> {
        if s.is_empty() {
            None
        } else {
            Some(Self(s.to_string()))
        }
    }

    /// Creates from string, returning None if empty.
    pub fn from_string(s: String) -> Option<Self> {
        if s.is_empty() {
            None
        } else {
            Some(Self(s))
        }
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn into_inner(self) -> String {
        self.0
    }
}

/// GOOD: Using TryFrom trait
pub struct PositiveInt(i32);

impl TryFrom<i32> for PositiveInt {
    type Error = &'static str;

    fn try_from(value: i32) -> Result<Self, Self::Error> {
        if value <= 0 {
            Err("Value must be positive")
        } else {
            Ok(Self(value))
        }
    }
}

impl PositiveInt {
    pub fn value(&self) -> i32 {
        self.0
    }
}

// ============================================================================
// GOOD EXAMPLES unit tests
// ============================================================================


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_try_new() {
        let good = GoodTryNew::try_new("hello");
        assert!(good.is_ok());

        let bad = GoodTryNew::try_new("");
        assert!(bad.is_err());
    }

    #[test]
    fn test_from_str() {
        let good = GoodFromPattern::from_str("content");
        assert!(good.is_ok());

        let bad = GoodFromPattern::from_str("");
        assert!(bad.is_err());
    }

    #[test]
    fn test_builder() {
        let config = ConfigBuilder::new()
            .content("some content")
            .validate(true)
            .build();
        assert!(config.is_ok());
    }

    #[test]
    fn test_non_empty_string() {
        assert!(NonEmptyString::new("hello").is_some());
        assert!(NonEmptyString::new("").is_none());
    }

    #[test]
    fn test_try_from() {
        let good: Result<PositiveInt, _> = 42.try_into();
        assert!(good.is_ok());

        let bad: Result<PositiveInt, _> = (-1).try_into();
        assert!(bad.is_err());
    }
}
