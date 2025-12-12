/// E1202: Complex lifetime annotations
/// Severity: MED
/// LLM confusion: 5 (HIGHEST)
///
/// Description: This function uses multiple lifetime parameters with complex relationships,
/// making it very difficult to understand when and how long data will remain valid. Lifetimes
/// are a Rust-specific concept that track how long references are valid. Fix by simplifying
/// lifetime relationships, using fewer lifetime parameters, or restructuring the code to avoid
/// complex lifetime dependencies.
///
/// Mitigation: Minimize the number of explicit lifetime parameters. Use lifetime elision rules
/// where possible (Rust can often infer lifetimes automatically). Consider restructuring to
/// return owned data instead of references when lifetime complexity becomes unmanageable.

// This function has 4 lifetime parameters ('a, 'b, 'c, 'd).
// Lifetimes tell Rust how long references (borrowed data) stay valid.
// The function takes 3 string references (x, y, z), each with its own lifetime.
// The 'where' clause says 'a, 'b, and 'c must all live at least as long as 'd.
// This means all input references must be valid for the entire output lifetime.
// The function returns a reference that lives for lifetime 'd.
//
// PROBLEM E1202: Overly complex lifetime relationships
pub fn e1202_bad_complex_lifetimes<'a, 'b, 'c, 'd>(x: &'a str, y: &'b str, z: &'c str) -> &'d str
where
    'a: 'd,
    'b: 'd,
    'c: 'd,
{
    if x.len() > y.len() {
        x
    } else {
        y
    }
}

pub fn e1202_entry() -> Result<(), Box<dyn std::error::Error>> {
    // Use a single concrete lifetime by keeping all strings in the same scope
    let x = "hello";
    let y = "world";
    let z = "!";
    let _ = e1202_bad_complex_lifetimes(x, y, z);
    Ok(())
}
