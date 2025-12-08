/// E1203: Complicated borrowing patterns
/// Severity: MED
/// LLM confusion: 4 (HIGH)
///
/// Description: This code has complex patterns of borrowing (taking references to data) that
/// interleave read and write access in confusing ways. In Rust, you can have many readers OR
/// one writer, but not both at the same time. Fix by simplifying the borrow scope, using
/// variables only when needed, and avoiding unnecessary reference manipulation.
///
/// Mitigation: Use `#![warn(clippy::dropping_references)]` to catch useless drop calls on
/// references. Keep borrow scopes as small as possible. Avoid creating references you don't
/// actually need.

// This function demonstrates complex borrowing patterns.
// It creates multiple immutable borrows (r1, r2) of 'data', which is allowed.
// Then it uses those borrows to read values (_len, _first).
// After those borrows end, it creates a mutable borrow (r3) to modify 'data'.
// The complexity comes from juggling multiple borrows and tracking when each one ends.
// In Rust, you can't have a mutable borrow while immutable borrows exist.
//
// PROBLEM E1203: Complex interleaving of borrows
#[allow(clippy::get_first)]
pub fn e1203_bad_complicated_borrowing() {
    let mut data = vec![1, 2, 3];
    let r1 = &data;
    let r2 = &data;
    let _len = r1.len();
    let _first = r2.get(0);
    // Borrows end here, allowing mutable borrow below
    let r3 = &mut data;
    r3.push(4);
}

pub fn e1203_entry() -> Result<(), Box<dyn std::error::Error>> {
    e1203_bad_complicated_borrowing();
    Ok(())
}
