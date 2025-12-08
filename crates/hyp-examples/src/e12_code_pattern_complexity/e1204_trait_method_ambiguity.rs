/// E1204: Multiple traits with same method names
/// Severity: MED
/// LLM confusion: 3 (MED)
///
/// Description: This code has multiple traits that define methods with the same name, and a struct
/// implements all of them. When calling these methods, you must use fully qualified syntax to
/// specify which trait's method you want. This is confusing because the same method name means
/// different things depending on which trait you're referring to. It's like having multiple
/// interfaces with identically named methods - you need extra syntax to disambiguate. Fix by
/// using different method names in different traits, or accepting that users will need to use
/// qualified syntax.
///
/// Mitigation: Avoid naming conflicts between traits when possible. Use descriptive, unique method
/// names. When conflicts are unavoidable, document which trait provides which behavior. Use
/// fully qualified syntax: `<Type as Trait>::method()` to be explicit.

// Three traits (Display, Serialize, Debug) each define a method called 'format'.
// Traits are like interfaces - they define behavior that types can implement.
// The Data struct implements all three traits, so it has three different 'format' methods.
// Each implementation does something different: Display shows human-readable text,
// Serialize outputs JSON format, and Debug shows debug information.
//
// PROBLEM E1204: Same method name in multiple traits
pub trait Display {
    fn format(&self) -> String;
}

pub trait Serialize {
    fn format(&self) -> String;
}

pub trait Debug {
    fn format(&self) -> String;
}

pub struct Data {
    value: i32,
}

impl Display for Data {
    fn format(&self) -> String {
        format!("Display: {}", self.value)
    }
}

impl Serialize for Data {
    fn format(&self) -> String {
        format!("{{\"value\":{}}}", self.value)
    }
}

impl Debug for Data {
    fn format(&self) -> String {
        format!("Data {{ value: {} }}", self.value)
    }
}

// This function shows why having the same method name in multiple traits is confusing.
// You can't just call data.format() because Rust wouldn't know WHICH format method you want.
// Instead, you must use the syntax <Type as Trait>::method(&value) to specify which trait.
// For example, <Data as Display>::format(&data) explicitly calls the Display trait's format method.
//
// PROBLEM E1204: Must use fully qualified syntax to disambiguate
pub fn e1204_bad_trait_method_ambiguity() {
    let data = Data { value: 42 };

    let _display = <Data as Display>::format(&data);
    let _serialize = <Data as Serialize>::format(&data);
    let _debug = <Data as Debug>::format(&data);
}

pub fn e1204_entry() -> Result<(), Box<dyn std::error::Error>> {
    e1204_bad_trait_method_ambiguity();
    Ok(())
}
