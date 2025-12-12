/// E1810: String instead of &str
/// Severity: LOW
/// LLM confusion: 2 (LOW)
///
/// Description: Accepting `String` as a parameter forces callers to own the string, requiring
/// allocation even if they have a string literal or `&str`. This is unnecessarily restrictive.
/// Fix by accepting `&str` instead, which works with string literals, `&str`, and `&String`
/// (via deref coercion).
///
/// ## The Forced Allocation Problem
///
/// ```text
/// fn greet(name: String) {
///     println!("Hello, {}!", name);
/// }
///
/// greet("Alice".to_string());  // Forced to allocate!
/// greet(name.clone());         // Forced to clone!
/// ```
///
/// ## Why This Matters
///
/// 1. **Unnecessary allocation**: Callers must allocate even for literals
/// 2. **Forced cloning**: Callers must clone owned strings
/// 3. **Inflexible API**: Harder to use than necessary
/// 4. **Performance**: Allocations are expensive
///
/// ## The Right Solutions
///
/// ### Option 1: Accept &str
/// ```rust
/// fn greet(name: &str) {
///     println!("Hello, {}!", name);
/// }
///
/// greet("Alice");           // Works!
/// greet(&owned_string);     // Works via deref!
/// ```
///
/// ### Option 2: Use impl AsRef<str>
/// ```rust
/// fn greet(name: impl AsRef<str>) {
///     println!("Hello, {}!", name.as_ref());
/// }
///
/// greet("Alice");           // Works!
/// greet(String::from("Bob")); // Works!
/// ```
///
/// ### Option 3: Use Cow for conditional ownership
/// ```rust
/// fn process(name: Cow<str>) -> String {
///     if needs_modification {
///         name.into_owned()  // Only allocate if needed
///     } else {
///         name.to_string()
///     }
/// }
/// ```
///
/// Mitigation: Use `#![warn(clippy::needless_pass_by_value)]` to catch this. Accept `&str` for
/// string parameters unless you need ownership. Use `impl AsRef<str>` or `impl Into<String>` for
/// maximum flexibility. Only take `String` if you need to store or modify it.

// ============================================================================
// PROBLEMATIC PATTERNS
// ============================================================================

/// PROBLEM E1810: Forces caller to allocate, should accept &str
pub fn e1810_bad_string_param(s: String) {
    println!("{s}");
}

/// PROBLEM E1810: Forces allocation for comparison
pub fn e1810_bad_string_compare(a: String, b: String) -> bool {
    a == b
}

/// PROBLEM E1810: Forces allocation just to check length
pub fn e1810_bad_string_length(s: String) -> usize {
    s.len()
}

/// Entry point for problem demonstration
pub fn e1810_entry() -> Result<(), Box<dyn std::error::Error>> {
    e1810_bad_string_param("hello".to_string()); // Forced allocation!
    Ok(())
}

// ============================================================================
// GOOD ALTERNATIVES
// ============================================================================

/// GOOD: Accept &str for read-only access
pub fn e1810_good_str_param(s: &str) {
    println!("{s}");
}

/// GOOD: Accept &str for comparison
pub fn e1810_good_str_compare(a: &str, b: &str) -> bool {
    a == b
}

/// GOOD: Accept &str for length
pub fn e1810_good_str_length(s: &str) -> usize {
    s.len()
}

/// GOOD: Use impl AsRef<str> for maximum flexibility
pub fn e1810_good_asref(s: impl AsRef<str>) {
    let s = s.as_ref();
    println!("{s}");
}

/// GOOD: Use impl Into<String> when you need ownership
pub fn e1810_good_into_string(s: impl Into<String>) -> String {
    let owned = s.into();
    format!("Processed: {}", owned)
}

/// GOOD: Accept String when you need to store it
pub struct NamedThing {
    name: String, // Needs to own the string
}

impl NamedThing {
    /// Takes &str, clones internally
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
        }
    }

    /// Takes String, moves it in (no clone if caller has String)
    pub fn with_name(name: String) -> Self {
        Self { name }
    }

    /// Takes impl Into<String> for flexibility
    pub fn from_name(name: impl Into<String>) -> Self {
        Self { name: name.into() }
    }

    pub fn name(&self) -> &str {
        &self.name
    }
}

/// GOOD: Use Cow for conditional ownership
use std::borrow::Cow;

pub fn e1810_good_cow(s: Cow<str>) -> String {
    if s.contains("modify") {
        // Need to modify, so take ownership
        let mut owned = s.into_owned();
        owned.push_str(" (modified)");
        owned
    } else {
        // No modification needed
        s.into_owned()
    }
}

/// GOOD: Accept &str, return String when creating new data
pub fn e1810_good_transform(s: &str) -> String {
    s.to_uppercase()
}

/// GOOD: Use &str for struct fields when lifetime is acceptable
pub struct StrView<'a> {
    content: &'a str,
}

impl<'a> StrView<'a> {
    pub fn new(content: &'a str) -> Self {
        Self { content }
    }

    pub fn content(&self) -> &str {
        self.content
    }
}

/// Decision guide for String vs &str
pub fn e1810_when_to_use_what() {
    println!("Use &str when:");
    println!("  - Reading/inspecting the string");
    println!("  - Comparing strings");
    println!("  - Passing to functions that take &str");
    println!();
    println!("Use String when:");
    println!("  - Storing in a struct (ownership needed)");
    println!("  - Modifying the string");
    println!("  - Returning a newly created string");
    println!();
    println!("Use impl AsRef<str> when:");
    println!("  - You want to accept both &str and String");
    println!("  - API flexibility is important");
    println!();
    println!("Use Cow<str> when:");
    println!("  - You might need to modify, but usually don't");
    println!("  - Avoiding allocation in the common case");
}

// ============================================================================
// GOOD EXAMPLES unit tests
// ============================================================================


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_str_param() {
        e1810_good_str_param("literal");
        e1810_good_str_param(&String::from("owned"));
    }

    #[test]
    fn test_str_compare() {
        assert!(e1810_good_str_compare("hello", "hello"));
        assert!(!e1810_good_str_compare("hello", "world"));
    }

    #[test]
    fn test_asref() {
        e1810_good_asref("literal");
        e1810_good_asref(String::from("owned"));
    }

    #[test]
    fn test_into_string() {
        let result = e1810_good_into_string("literal");
        assert!(result.contains("Processed"));

        let result = e1810_good_into_string(String::from("owned"));
        assert!(result.contains("Processed"));
    }

    #[test]
    fn test_named_thing() {
        let t1 = NamedThing::new("from &str");
        let t2 = NamedThing::with_name(String::from("from String"));
        let t3 = NamedThing::from_name("flexible");

        assert_eq!(t1.name(), "from &str");
        assert_eq!(t2.name(), "from String");
        assert_eq!(t3.name(), "flexible");
    }

    #[test]
    fn test_cow() {
        let borrowed: Cow<str> = Cow::Borrowed("hello");
        let result = e1810_good_cow(borrowed);
        assert_eq!(result, "hello");

        let needs_mod: Cow<str> = Cow::Borrowed("modify me");
        let result = e1810_good_cow(needs_mod);
        assert!(result.contains("modified"));
    }
}
