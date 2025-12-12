/// E1209: Higher-ranked trait bounds (HRTB)
/// Severity: HIGH
/// LLM confusion: 5 (HIGHEST)
///
/// Description: This code uses higher-ranked trait bounds (for<'a>) which express that a type
/// must work for ALL possible lifetimes, not just one specific lifetime. This is an extremely
/// advanced feature that's very hard to understand because it involves quantification over
/// lifetimes. It's like saying "this must work no matter what lifetime you give it" which is
/// a very abstract concept. Fix by avoiding HRTBs when possible, or using concrete lifetimes.
///
/// Mitigation: Avoid HRTBs unless absolutely necessary. When needed, add extensive documentation
/// explaining why. Use concrete lifetimes when possible. Understand that `for<'a>` means "for all
/// lifetimes" not "for some lifetime". Provide examples showing how the HRTB is used.

// This function takes a closure (f) that works with string references.
// The `for<'a>` syntax means f must work for ANY lifetime 'a - not just one specific lifetime.
// This is called a "higher-ranked trait bound" (HRTB).
// The function can call f with strings that have different lifetimes (s1 and s2),
// and f must handle both correctly. This is more flexible than a regular lifetime bound.
//
// PROBLEM E1209: Higher-ranked trait bound - works for ANY lifetime
pub fn e1209_bad_hrtb_example<F>(f: F)
where
    F: for<'a> Fn(&'a str) -> &'a str,
{
    let s1 = "hello";
    let s2 = "world";

    // F must work for both lifetimes
    let _r1 = f(s1);
    let _r2 = f(s2);
}

// This trait has a lifetime parameter 'a in its definition.
// Implementations can specify what 'a means for their specific type.
//
// PROBLEM E1209: HRTB with multiple lifetime parameters
pub trait Processor<'a> {
    type Output;
    fn process(&self, input: &'a str) -> Self::Output;
}

// This function takes a closure that needs TWO lifetime parameters ('a and 'b).
// The `for<'a, 'b>` means the closure must work for ANY combination of lifetimes.
// It takes two string references (potentially with different lifetimes)
// and returns a reference with the same lifetime as the first one.
pub fn e1209_bad_complex_hrtb<T>(processor: T)
where
    T: for<'a, 'b> Fn(&'a str, &'b str) -> &'a str,
{
    let _result = processor("foo", "bar");
}

// This trait has a method that itself takes a closure with an HRTB.
// The map method receives a function f that must work for all lifetimes 'a.
//
// PROBLEM E1209: HRTB in trait bounds
pub trait Mapper {
    fn map<F>(&self, f: F)
    where
        F: for<'a> Fn(&'a i32) -> &'a i32;
}

// This function shows HRTBs nested inside other HRTBs - extremely complex!
// F is a function that works for all lifetimes 'a.
// G is a function that takes F and returns a Box containing ANOTHER function
// that itself works for all lifetimes 'c.
// So we have: for<'a> inside for<'b> inside for<'c> - three levels of "for all lifetimes"!
//
// PROBLEM E1209: Nested HRTB
pub fn e1209_bad_nested_hrtb<F, G>(f: F, g: G)
where
    F: for<'a> Fn(&'a str) -> &'a str,
    G: for<'b> Fn(F) -> Box<dyn for<'c> Fn(&'c str) -> &'c str>,
{
    unimplemented!("Nested higher-ranked trait bounds")
}

pub fn e1209_entry() -> Result<(), Box<dyn std::error::Error>> {
    e1209_bad_hrtb_example(|s| s);
    Ok(())
}
