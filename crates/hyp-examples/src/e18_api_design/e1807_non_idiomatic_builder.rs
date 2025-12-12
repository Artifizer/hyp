/// E1807: Non-idiomatic builder
/// Severity: LOW
/// LLM confusion: 2 (LOW)
///
/// Description: The builder pattern in Rust typically has methods return `self` to allow method
/// chaining like `builder.set_a(1).set_b(2).build()`. This builder doesn't return self, so you
/// can't chain calls. Fix by returning `self` (or `&mut self`) from builder methods to enable
/// chaining.
///
/// ## The Unchainable Builder Problem
///
/// ```text
/// impl Builder {
///     pub fn set_name(&mut self, name: &str) {
///         self.name = name.to_string();
///     }
///     pub fn set_age(&mut self, age: u32) {
///         self.age = age;
///     }
/// }
///
/// // Awkward usage:
/// let mut builder = Builder::new();
/// builder.set_name("Alice");
/// builder.set_age(30);
/// let result = builder.build();
/// ```
///
/// ## Why This Matters
///
/// 1. **Verbose**: Requires separate statements for each field
/// 2. **Non-idiomatic**: Doesn't follow Rust conventions
/// 3. **Error-prone**: Easy to forget to set fields
/// 4. **Less readable**: Can't see full configuration at a glance
///
/// ## The Right Solutions
///
/// ### Option 1: Return Self (consuming)
/// ```rust
/// impl Builder {
///     pub fn name(mut self, name: &str) -> Self {
///         self.name = Some(name.to_string());
///         self
///     }
/// }
///
/// let result = Builder::new().name("Alice").age(30).build();
/// ```
///
/// ### Option 2: Return &mut Self (borrowing)
/// ```rust
/// impl Builder {
///     pub fn name(&mut self, name: &str) -> &mut Self {
///         self.name = Some(name.to_string());
///         self
///     }
/// }
///
/// let result = Builder::new().name("Alice").age(30).build();
/// ```
///
/// Mitigation: Builder methods should return `Self` or `&mut Self` to enable chaining. Use the
/// `typed-builder` crate for automatic builder generation. Follow the pattern: `pub fn field(mut self, value: T) -> Self { self.field = value; self }`.

// ============================================================================
// PROBLEMATIC PATTERNS
// ============================================================================

/// PROBLEM E1807: Builder methods don't return self
pub struct BadBuilder {
    name: Option<String>,
    age: Option<u32>,
    email: Option<String>,
}

impl BadBuilder {
    pub fn new() -> Self {
        Self {
            name: None,
            age: None,
            email: None,
        }
    }

    // PROBLEM: Doesn't return self, can't chain
    pub fn set_name(&mut self, name: &str) {
        self.name = Some(name.to_string());
    }

    // PROBLEM: Doesn't return self, can't chain
    pub fn set_age(&mut self, age: u32) {
        self.age = Some(age);
    }

    // PROBLEM: Doesn't return self, can't chain
    pub fn set_email(&mut self, email: &str) {
        self.email = Some(email.to_string());
    }

    pub fn build(self) -> Result<User, &'static str> {
        Ok(User {
            name: self.name.ok_or("name required")?,
            age: self.age.ok_or("age required")?,
            email: self.email,
        })
    }
}

impl Default for BadBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug)]
pub struct User {
    pub name: String,
    pub age: u32,
    pub email: Option<String>,
}

/// Entry point for problem demonstration
pub fn e1807_entry() -> Result<(), Box<dyn std::error::Error>> {
    // Awkward, verbose usage
    let mut builder = BadBuilder::new();
    builder.set_name("Alice");
    builder.set_age(30);
    builder.set_email("alice@example.com");
    let _user = builder.build()?;
    Ok(())
}

// ============================================================================
// GOOD ALTERNATIVES
// ============================================================================

/// GOOD: Builder with consuming self (move semantics)
#[derive(Default)]
pub struct GoodBuilderOwned {
    name: Option<String>,
    age: Option<u32>,
    email: Option<String>,
}

impl GoodBuilderOwned {
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the name. Consumes and returns self for chaining.
    pub fn name(mut self, name: &str) -> Self {
        self.name = Some(name.to_string());
        self
    }

    /// Sets the age. Consumes and returns self for chaining.
    pub fn age(mut self, age: u32) -> Self {
        self.age = Some(age);
        self
    }

    /// Sets the email. Consumes and returns self for chaining.
    pub fn email(mut self, email: &str) -> Self {
        self.email = Some(email.to_string());
        self
    }

    /// Builds the User, consuming the builder.
    pub fn build(self) -> Result<User, &'static str> {
        Ok(User {
            name: self.name.ok_or("name required")?,
            age: self.age.ok_or("age required")?,
            email: self.email,
        })
    }
}

/// GOOD: Builder with borrowing self (reference semantics)
#[derive(Default)]
pub struct GoodBuilderBorrowed {
    name: Option<String>,
    age: Option<u32>,
    email: Option<String>,
}

impl GoodBuilderBorrowed {
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the name. Returns &mut self for chaining.
    pub fn name(&mut self, name: &str) -> &mut Self {
        self.name = Some(name.to_string());
        self
    }

    /// Sets the age. Returns &mut self for chaining.
    pub fn age(&mut self, age: u32) -> &mut Self {
        self.age = Some(age);
        self
    }

    /// Sets the email. Returns &mut self for chaining.
    pub fn email(&mut self, email: &str) -> &mut Self {
        self.email = Some(email.to_string());
        self
    }

    /// Builds the User. Takes &self so builder can be reused.
    pub fn build(&self) -> Result<User, &'static str> {
        Ok(User {
            name: self.name.clone().ok_or("name required")?,
            age: self.age.ok_or("age required")?,
            email: self.email.clone(),
        })
    }
}

/// GOOD: Builder with validation at each step
#[derive(Debug, Default)]
pub struct ValidatingBuilder {
    name: Option<String>,
    age: Option<u32>,
    email: Option<String>,
}

impl ValidatingBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the name with validation.
    pub fn name(mut self, name: &str) -> Result<Self, &'static str> {
        if name.is_empty() {
            return Err("name cannot be empty");
        }
        self.name = Some(name.to_string());
        Ok(self)
    }

    /// Sets the age with validation.
    pub fn age(mut self, age: u32) -> Result<Self, &'static str> {
        if age > 150 {
            return Err("age too high");
        }
        self.age = Some(age);
        Ok(self)
    }

    /// Sets the email with validation.
    pub fn email(mut self, email: &str) -> Result<Self, &'static str> {
        if !email.contains('@') {
            return Err("invalid email");
        }
        self.email = Some(email.to_string());
        Ok(self)
    }

    pub fn build(self) -> Result<User, &'static str> {
        Ok(User {
            name: self.name.ok_or("name required")?,
            age: self.age.ok_or("age required")?,
            email: self.email,
        })
    }
}

/// GOOD: Typestate builder pattern (compile-time validation)
pub mod typestate {
    pub struct NoName;
    pub struct HasName;
    pub struct NoAge;
    pub struct HasAge;

    pub struct TypestateBuilder<N, A> {
        name: Option<String>,
        age: Option<u32>,
        _name_state: std::marker::PhantomData<N>,
        _age_state: std::marker::PhantomData<A>,
    }

    impl TypestateBuilder<NoName, NoAge> {
        pub fn new() -> Self {
            Self {
                name: None,
                age: None,
                _name_state: std::marker::PhantomData,
                _age_state: std::marker::PhantomData,
            }
        }
    }

    impl Default for TypestateBuilder<NoName, NoAge> {
        fn default() -> Self {
            Self::new()
        }
    }

    impl<A> TypestateBuilder<NoName, A> {
        pub fn name(self, name: &str) -> TypestateBuilder<HasName, A> {
            TypestateBuilder {
                name: Some(name.to_string()),
                age: self.age,
                _name_state: std::marker::PhantomData,
                _age_state: std::marker::PhantomData,
            }
        }
    }

    impl<N> TypestateBuilder<N, NoAge> {
        pub fn age(self, age: u32) -> TypestateBuilder<N, HasAge> {
            TypestateBuilder {
                name: self.name,
                age: Some(age),
                _name_state: std::marker::PhantomData,
                _age_state: std::marker::PhantomData,
            }
        }
    }

    impl TypestateBuilder<HasName, HasAge> {
        /// Can only build when both name and age are set!
        pub fn build(self) -> super::User {
            super::User {
                name: self.name.unwrap(),
                age: self.age.unwrap(),
                email: None,
            }
        }
    }
}

// ============================================================================
// GOOD EXAMPLES unit tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_owned_builder() {
        let user = GoodBuilderOwned::new()
            .name("Alice")
            .age(30)
            .email("alice@example.com")
            .build()
            .unwrap();

        assert_eq!(user.name, "Alice");
        assert_eq!(user.age, 30);
    }

    #[test]
    fn test_borrowed_builder() {
        let user = GoodBuilderBorrowed::new()
            .name("Bob")
            .age(25)
            .build()
            .unwrap();

        assert_eq!(user.name, "Bob");
        assert_eq!(user.age, 25);
    }

    #[test]
    fn test_validating_builder() {
        let result = ValidatingBuilder::new()
            .name("Charlie")
            .unwrap()
            .age(35)
            .unwrap()
            .email("charlie@example.com")
            .unwrap()
            .build();

        assert!(result.is_ok());
    }

    #[test]
    fn test_validating_builder_error() {
        let result = ValidatingBuilder::new().name("").unwrap_err();
        assert_eq!(result, "name cannot be empty");
    }

    #[test]
    fn test_typestate_builder() {
        // This won't compile if you try to build without name and age:
        // typestate::TypestateBuilder::new().build(); // Error!

        let user = typestate::TypestateBuilder::new()
            .name("Diana")
            .age(40)
            .build();

        assert_eq!(user.name, "Diana");
        assert_eq!(user.age, 40);
    }
}
