/// E1208: Phantom types and zero-sized markers
/// Severity: MED
/// LLM confusion: 5 (HIGHEST)
///
/// Description: This code uses PhantomData and zero-sized types to encode state or behavior at
/// the type level, making the code work differently based on types that don't exist at runtime.
/// This is a very advanced Rust pattern that's extremely confusing because types are used for
/// logic, not just data. It's like having invisible markers that change how code behaves. Fix by
/// using runtime state instead of type-level state, or accepting that this pattern is inherently
/// complex and documenting it extensively.
///
/// Mitigation: Avoid phantom types unless absolutely necessary for API safety. Document why
/// phantom types are used and what invariants they enforce. Consider if runtime checks would be
/// simpler. Use descriptive names for marker types. Provide examples of correct usage.
use std::marker::PhantomData;

// PROBLEM E1208: Zero-sized marker types for type-level state
pub struct Locked;
pub struct Unlocked;

pub struct Database<State> {
    connection: String,
    _state: PhantomData<State>,
}

impl Database<Unlocked> {
    pub fn new(connection: String) -> Self {
        Database {
            connection,
            _state: PhantomData,
        }
    }

    pub fn lock(self) -> Database<Locked> {
        Database {
            connection: self.connection,
            _state: PhantomData,
        }
    }
}

impl Database<Locked> {
    // PROBLEM E1208: This method only exists for Locked state
    pub fn execute_query(&self, _query: &str) -> Result<(), String> {
        Ok(())
    }

    pub fn unlock(self) -> Database<Unlocked> {
        Database {
            connection: self.connection,
            _state: PhantomData,
        }
    }
}

// PROBLEM E1208: Multiple phantom type parameters
pub struct TypedBuilder<T, HasName, HasAge, HasEmail> {
    name: Option<String>,
    age: Option<u32>,
    email: Option<String>,
    _phantom: PhantomData<(T, HasName, HasAge, HasEmail)>,
}

pub struct Yes;
pub struct No;

pub fn e1208_bad_phantom_types() {
    let db = Database::<Unlocked>::new("localhost".to_string());
    let locked_db = db.lock();

    // PROBLEM E1208: Type state prevents calling execute_query on unlocked database
    // This is enforced at compile time through phantom types
    let _ = locked_db.execute_query("SELECT * FROM users");
}

pub fn e1208_entry() -> Result<(), Box<dyn std::error::Error>> {
    e1208_bad_phantom_types();
    Ok(())
}
