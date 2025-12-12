/// E1201: Overly complex generics
/// Severity: MED
/// LLM confusion: 4 (HIGH)
///
/// Description: This function has too many generic type parameters with complex trait bounds,
/// making it difficult to understand what types are actually needed and why. This creates a high
/// cognitive burden for anyone reading or using the code. Fix by reducing the number of type
/// parameters, using concrete types where generics aren't needed, or grouping related parameters
/// into a single trait or struct.
///
/// Mitigation: Use `#![warn(clippy::type_complexity)]` to catch overly complex types. Consider
/// using trait objects (`Box<dyn Trait>`) for simpler signatures when performance isn't critical.
/// Limit type parameters to 3 or fewer when possible.

// This function takes 5 generic type parameters (T, U, V, W, X).
// Generic parameters are like placeholders - the function works with ANY types
// that meet the requirements listed in the 'where' clause.
// For example, T must be Clone (copyable), Send (thread-safe to send), Sync (thread-safe to share),
// Debug (can be printed for debugging), and have a 'static lifetime (lives for entire program).
// The function returns a tuple of the first 3 parameters wrapped in a Result.
//
// PROBLEM E1201: Too many type parameters and complex bounds
pub fn e1201_bad_complex_generics<T, U, V, W, X>(
    _a: T,
    _b: U,
    _c: V,
    _d: W,
    _e: X,
) -> Result<(T, U, V), Box<dyn std::error::Error>>
where
    T: Clone + Send + Sync + std::fmt::Debug + 'static,
    U: Clone + Send + Sync + std::fmt::Display + Into<String>,
    V: Clone + Send + Sync + Default + PartialEq + Eq,
    W: Clone + Send + Sync,
    X: Clone + Send + Sync,
{
    Ok((_a, _b, _c))
}

pub fn e1201_entry() -> Result<(), Box<dyn std::error::Error>> {
    let _ = e1201_bad_complex_generics(1_i32, String::from("b"), 0_i32, 2_i32, 3_i32);
    Ok(())
}
