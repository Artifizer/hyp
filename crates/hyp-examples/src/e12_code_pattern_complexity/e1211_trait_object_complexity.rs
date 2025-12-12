/// E1211: Trait object coercion complexity
/// Severity: MED
/// LLM confusion: 4 (HIGH)
///
/// Description: This code uses trait objects (dyn Trait) with complex combinations of traits,
/// lifetimes, and Send/Sync bounds, making it very hard to understand what types are actually
/// compatible. Trait objects erase the concrete type, and adding multiple trait bounds with
/// lifetimes creates a complex set of requirements. It's like having a variable that could be
/// any type as long as it meets a long list of requirements. Fix by simplifying trait bounds,
/// using concrete types when possible, or using generics instead of trait objects.
///
/// Mitigation: Limit trait object bounds to 1-2 traits. Use `+ Send + Sync` only when necessary.
/// Avoid complex lifetime bounds on trait objects. Consider using generics instead of trait
/// objects for better performance and clarity. Use type aliases for complex trait objects.
use std::fmt::Debug;

// This type alias defines a complex callback function stored in a Box.
// dyn Fn means "any type that implements the Fn trait" (the concrete type is erased).
// The callback takes a &str, returns a Result, AND must be Send + Sync (thread-safe).
// The 'a lifetime means the callback can reference data that lives for 'a.
// Combining dyn + multiple traits + lifetimes creates a complex type that's hard to understand.
//
// PROBLEM E1211: Complex trait object with multiple bounds
pub type ComplexCallback<'a> = Box<
    dyn Fn(&str) -> Result<String, Box<dyn std::error::Error + Send + Sync>> + Send + Sync + 'a,
>;

// This function takes a vector of boxed trait objects (handlers).
// Each handler is "dyn Fn(i32) -> i32" meaning any function type.
// The handlers must be Send + Sync + 'a (thread-safe and live for lifetime 'a).
// It returns a boxed Iterator trait object with lifetime 'a.
// Trait objects with lifetimes and multiple bounds are confusing because you need to track
// which concrete types satisfy all the requirements.
//
// PROBLEM E1211: Trait object with lifetime and multiple trait bounds
pub fn e1211_bad_trait_object_complexity<'a>(
    handlers: Vec<Box<dyn Fn(i32) -> i32 + Send + Sync + 'a>>,
) -> Box<dyn Iterator<Item = i32> + 'a> {
    let combined: Box<dyn Iterator<Item = i32> + 'a> = Box::new(
        handlers
            .into_iter()
            .enumerate()
            .map(|(i, _handler)| i as i32),
    );
    combined
}

// This struct holds three different vectors of trait objects.
// handlers: functions that take &str (don't return anything)
// filters: functions that take &str and return bool
// transformers: functions that take String and return String
// Each has different trait bound combinations (Send + Sync, or just Send).
// Managing multiple different trait object types in one struct is confusing.
//
// PROBLEM E1211: Multiple nested trait objects
pub struct EventSystem {
    handlers: Vec<Box<dyn Fn(&str) + Send + Sync>>,
    filters: Vec<Box<dyn Fn(&str) -> bool + Send + Sync>>,
    transformers: Vec<Box<dyn Fn(String) -> String + Send>>,
}

// This trait has an associated type Output with its own trait bounds (Debug + Send).
// The function returns a trait object (dyn Handler) where Output is fixed to String.
// Combining trait objects with associated type constraints is complex because
// you're constraining both the trait AND its associated types.
//
// PROBLEM E1211: Trait object with associated type constraints
pub trait Handler {
    type Output: Debug + Send;
    fn handle(&self, input: &str) -> Self::Output;
}

/// A concrete handler that implements the Handler trait
struct StringHandler;

impl Handler for StringHandler {
    type Output = String;
    fn handle(&self, input: &str) -> Self::Output {
        format!("Handled: {}", input)
    }
}

pub fn e1211_bad_complex_trait_object(flag: i32) -> Box<dyn Handler<Output = String>> {
    if flag > 0 {
        // PROBLEM E1211: Returning trait objects with associated type constraints is complex
        unimplemented!("Complex trait object with associated types")
    } else {
        Box::new(StringHandler)
    }
}

// This function combines trait objects (dyn Fn) with generic type parameters (T).
// The _processor is a trait object that works with generic type T.
// The _validator is also a trait object that works with T.
// But T itself has trait bounds (Clone + Debug + Send + 'static).
// Mixing generics with trait objects creates complexity - you have TWO levels of abstraction.
//
// PROBLEM E1211: Combining trait objects with generics
pub fn e1211_bad_mixed_complexity<T>(
    _processor: Box<dyn Fn(T) -> T + Send>,
    _validator: Box<dyn Fn(&T) -> bool + Sync>,
) where
    T: Clone + Debug + Send + 'static,
{
    unimplemented!("Mixed trait objects and generics")
}

pub fn e1211_entry() -> Result<(), Box<dyn std::error::Error>> {
    let handlers: Vec<Box<dyn Fn(i32) -> i32 + Send + Sync>> = vec![];
    let _ = e1211_bad_trait_object_complexity(handlers);
    // Call with flag <= 0 to avoid unimplemented branch
    let handler = e1211_bad_complex_trait_object(0);
    let _ = handler.handle("test");
    // Note: e1211_bad_mixed_complexity uses unimplemented!() so we don't call it
    Ok(())
}
