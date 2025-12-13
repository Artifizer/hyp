//! E1905: Suspicious Code Examples
//!
//! Examples of code patterns that are almost certainly bugs.

/// Entry point for E1905 demonstration
pub fn e1905_entry() -> Result<(), Box<dyn std::error::Error>> {
    println!("E1905: Demonstrating suspicious code patterns (these are almost always bugs)");
    println!();

    // Note: These functions contain intentionally suspicious code for demonstration
    // The checker will flag these patterns

    demonstrate_eq_op();
    demonstrate_impossible_comparisons();
    demonstrate_self_assignment();
    demonstrate_ifs_same_cond();
    demonstrate_never_loop();
    demonstrate_while_immutable_condition();

    Ok(())
}

// PROBLEM E1905: Comparing identical expressions is suspicious
fn demonstrate_eq_op() {
    let x = 42;

    // PROBLEM E1905: Suspicious: x == x is always true
    if x == x {
        println!("  eq_op: 'x == x' is always true - likely a typo");
    }

    // PROBLEM E1905: Suspicious: x != x is always false (except NaN)
    if x != x {
        println!("  This never prints");
    }

    // PROBLEM E1905: Suspicious: x && x is redundant
    let a = true;
    if a && a {
        println!("  eq_op: 'a && a' is redundant");
    }

    // PROBLEM E1905: Suspicious: x ^ x is always 0
    let y = x ^ x;
    println!("  eq_op: 'x ^ x' is always 0, got: {}", y);

    // PROBLEM E1905: Suspicious: x - x is always 0
    let z = x - x;
    println!("  eq_op: 'x - x' is always 0, got: {}", z);
}

// PROBLEM E1905: Impossible comparisons with identical expressions
fn demonstrate_impossible_comparisons() {
    let x = 42;

    // PROBLEM E1905: Impossible: x < x is always false
    if x < x {
        println!("  This never prints");
    } else {
        println!("  impossible_comparison: 'x < x' is always false");
    }

    // PROBLEM E1905: Impossible: x > x is always false
    if x > x {
        println!("  This never prints");
    } else {
        println!("  impossible_comparison: 'x > x' is always false");
    }

    // PROBLEM E1905: Trivial: x <= x is always true
    if x <= x {
        println!("  impossible_comparison: 'x <= x' is always true");
    }

    // PROBLEM E1905: Trivial: x >= x is always true
    if x >= x {
        println!("  impossible_comparison: 'x >= x' is always true");
    }
}

// PROBLEM E1905: Self-assignment has no effect
fn demonstrate_self_assignment() {
    let mut x = 42;

    // PROBLEM E1905: Suspicious: assigning x to itself does nothing
    x = x;

    println!(
        "  self_assignment: 'x = x' has no effect, x is still: {}",
        x
    );
}

// PROBLEM E1905: Same condition in if-else chain is likely a typo
fn demonstrate_ifs_same_cond() {
    let x = 42;

    // PROBLEM E1905: Suspicious: same condition used twice in if-else chain
    if x > 0 {
        println!("  ifs_same_cond: first branch with 'x > 0'");
    } else if x > 0 {
        // This is the duplicate/suspicious condition
        println!("  This never executes - duplicate condition");
    } else {
        println!("  else branch");
    }
}

// PROBLEM E1905: Loop that never iterates
fn demonstrate_never_loop() {
    println!("  never_loop: Loop with unconditional break at start");

    let mut count = 0;
    loop {
        break; // Unconditional break - loop never iterates
        #[allow(unreachable_code)]
        {
            count += 1;
        }
    }

    println!("  Loop executed {} times (should be 0)", count);
}

// PROBLEM E1905: While loop with immutable condition
fn demonstrate_while_immutable_condition() {
    // Suspicious: while false { } - body never executes
    println!("  while_immutable_condition: 'while false' body never executes");
    let mut count = 0;
    while false {
        count += 1;
    }
    println!("  Count after 'while false': {} (should be 0)", count);

    // Note: 'while true' is flagged but can be legitimate if there's a break
    println!("  while_immutable_condition: 'while true' is infinite unless broken");
    let mut iterations = 0;
    #[allow(while_true)]
    while true {
        iterations += 1;
        if iterations >= 1 {
            break; // Without this, it would loop forever
        }
    }
    println!(
        "  Controlled 'while true' loop executed {} time(s)",
        iterations
    );
}
