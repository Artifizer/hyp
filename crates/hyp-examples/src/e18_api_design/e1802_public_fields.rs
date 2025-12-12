/// E1802: Public fields without validation
/// Severity: MED
/// LLM confusion: 2 (LOW)
///
/// Description: Public fields can be set to any value by external code, bypassing validation.
/// This struct has a public `age` field that could be set to negative values, and an `email`
/// field that could be set to invalid email addresses. Fix by making fields private and providing
/// getter/setter methods that validate inputs.
///
/// ## The Unvalidated Data Problem
///
/// ```text
/// struct User {
///     pub age: i32,
///     pub email: String,
/// }
///
/// let mut user = User { age: 25, email: "valid@example.com".into() };
/// user.age = -100;              // No validation!
/// user.email = "not an email".into();  // No validation!
/// ```
///
/// ## Why This Matters
///
/// 1. **Invalid state**: Data can violate invariants
/// 2. **No encapsulation**: Can't change internal representation
/// 3. **Security risk**: Malicious input not validated
/// 4. **Hard to debug**: Invalid data causes errors elsewhere
///
/// ## The Right Solutions
///
/// ### Option 1: Private fields with validated setters
/// ```rust
/// struct User {
///     age: u32,  // Private, use u32 for non-negative
///     email: String,
/// }
///
/// impl User {
///     pub fn set_age(&mut self, age: u32) {
///         self.age = age;
///     }
///
///     pub fn set_email(&mut self, email: &str) -> Result<(), &'static str> {
///         if email.contains('@') {
///             self.email = email.to_string();
///             Ok(())
///         } else {
///             Err("Invalid email")
///         }
///     }
/// }
/// ```
///
/// ### Option 2: Builder pattern with validation
/// ```rust
/// impl UserBuilder {
///     pub fn email(mut self, email: &str) -> Result<Self, &'static str> {
///         if !email.contains('@') {
///             return Err("Invalid email");
///         }
///         self.email = Some(email.to_string());
///         Ok(self)
///     }
/// }
/// ```
///
/// Mitigation: Make fields private by default. Provide getter methods for read access and setter
/// methods (or builder pattern) for write access with validation. Use newtype patterns for values
/// that need validation. Consider using the `typed-builder` crate for complex construction.

// ============================================================================
// PROBLEMATIC PATTERNS
// ============================================================================

/// PROBLEM E1802: Public fields can be set to invalid values
pub struct BadPublicFields {
    pub age: i32,      // Could be negative!
    pub email: String, // Could be invalid!
    pub balance: f64,  // Could be NaN or negative!
}

impl BadPublicFields {
    pub fn new() -> Self {
        Self {
            age: 0,
            email: String::new(),
            balance: 0.0,
        }
    }
}

impl Default for BadPublicFields {
    fn default() -> Self {
        Self::new()
    }
}

/// Entry point for problem demonstration
pub fn e1802_entry() -> Result<(), Box<dyn std::error::Error>> {
    let mut user = BadPublicFields::new();
    user.age = -100; // Allowed but invalid!
    user.email = "not an email".to_string(); // Allowed but invalid!
    Ok(())
}

// ============================================================================
// GOOD ALTERNATIVES
// ============================================================================

/// GOOD: Private fields with validated accessors
pub struct GoodValidatedFields {
    age: u32,      // Private, use u32 for non-negative
    email: String, // Private
    balance: f64,  // Private
}

impl GoodValidatedFields {
    /// Create a new user with validated fields
    pub fn new(age: u32, email: &str, balance: f64) -> Result<Self, &'static str> {
        if !email.contains('@') {
            return Err("Invalid email");
        }
        if balance.is_nan() || balance < 0.0 {
            return Err("Invalid balance");
        }
        Ok(Self {
            age,
            email: email.to_string(),
            balance,
        })
    }

    pub fn age(&self) -> u32 {
        self.age
    }

    pub fn email(&self) -> &str {
        &self.email
    }

    pub fn balance(&self) -> f64 {
        self.balance
    }

    pub fn set_age(&mut self, age: u32) {
        self.age = age;
    }

    pub fn set_email(&mut self, email: &str) -> Result<(), &'static str> {
        if !email.contains('@') {
            return Err("Invalid email");
        }
        self.email = email.to_string();
        Ok(())
    }

    pub fn set_balance(&mut self, balance: f64) -> Result<(), &'static str> {
        if balance.is_nan() || balance < 0.0 {
            return Err("Invalid balance");
        }
        self.balance = balance;
        Ok(())
    }
}

/// GOOD: Newtype pattern for validated values
#[derive(Debug, Clone)]
pub struct Email(String);

impl Email {
    pub fn new(email: &str) -> Result<Self, &'static str> {
        if !email.contains('@') {
            return Err("Invalid email format");
        }
        Ok(Self(email.to_string()))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Age(u32);

impl Age {
    pub fn new(age: u32) -> Result<Self, &'static str> {
        if age > 150 {
            return Err("Age too high");
        }
        Ok(Self(age))
    }

    pub fn value(&self) -> u32 {
        self.0
    }
}

/// GOOD: Using newtypes in struct
pub struct GoodNewtypeFields {
    age: Age,
    email: Email,
}

impl GoodNewtypeFields {
    pub fn new(age: Age, email: Email) -> Self {
        Self { age, email }
    }

    pub fn age(&self) -> u32 {
        self.age.value()
    }

    pub fn email(&self) -> &str {
        self.email.as_str()
    }
}

/// GOOD: Builder pattern with validation
#[derive(Default)]
pub struct UserBuilder {
    age: Option<u32>,
    email: Option<String>,
}

impl UserBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn age(mut self, age: u32) -> Self {
        self.age = Some(age);
        self
    }

    pub fn email(mut self, email: &str) -> Result<Self, &'static str> {
        if !email.contains('@') {
            return Err("Invalid email");
        }
        self.email = Some(email.to_string());
        Ok(self)
    }

    pub fn build(self) -> Result<GoodValidatedFields, &'static str> {
        let age = self.age.ok_or("Age is required")?;
        let email = self.email.ok_or("Email is required")?;
        GoodValidatedFields::new(age, &email, 0.0)
    }
}

// ============================================================================
// GOOD EXAMPLES unit tests
// ============================================================================


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validated_fields() {
        let user = GoodValidatedFields::new(25, "test@example.com", 100.0);
        assert!(user.is_ok());
    }

    #[test]
    fn test_invalid_email() {
        let user = GoodValidatedFields::new(25, "invalid", 100.0);
        assert!(user.is_err());
    }

    #[test]
    fn test_newtype_email() {
        let email = Email::new("test@example.com");
        assert!(email.is_ok());

        let invalid = Email::new("invalid");
        assert!(invalid.is_err());
    }

    #[test]
    fn test_builder() {
        let user = UserBuilder::new()
            .age(25)
            .email("test@example.com")
            .unwrap()
            .build();
        assert!(user.is_ok());
    }
}
