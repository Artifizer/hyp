/// E1611: Methods consuming self when &self or &mut self would suffice
/// Severity: MEDIUM
/// LLM confusion: 2 (LOW)
///
/// Description: Taking `self` by value (consuming ownership) forces callers to give up
/// ownership even when they just want to read or inspect the value. This is particularly
/// problematic for getters and validation methods that should borrow instead.
///
/// Mitigation: Use &self for read-only access, &mut self for mutation. Reserve self
/// for builder pattern methods that return Self, or into_* conversion methods.

/// PROBLEM E1611: Getter that consumes self - forces caller to give up ownership!
pub struct Config {
    pub port: u16,
    pub host: String,
}

impl Config {
    /// PROBLEM E1611: Takes ownership just to read a value
    pub fn e1611_get_port_bad(self) -> u16 {
        self.port
    }

    /// PROBLEM E1611: Consumes self just to check a condition
    pub fn e1611_is_valid_bad(self) -> bool {
        self.port > 0 && !self.host.is_empty()
    }

    /// PROBLEM E1611: Consumes self to format
    pub fn e1611_to_connection_string_bad(self) -> String {
        format!("{}:{}", self.host, self.port)
    }
}

/// Example of problematic usage - config is moved after first call
pub fn e1611_problematic_usage() {
    let config = Config {
        port: 8080,
        host: "localhost".to_string(),
    };

    let _port = config.e1611_get_port_bad();
    // config is now moved/consumed!
    // let _valid = config.e1611_is_valid_bad(); // ERROR: use after move!
}

pub fn e1611_entry() -> Result<(), Box<dyn std::error::Error>> {
    e1611_problematic_usage();
    Ok(())
}

// ============================================================================
// GOOD EXAMPLES - Proper borrowing
// ============================================================================

pub struct ConfigGood {
    pub port: u16,
    pub host: String,
}

impl ConfigGood {
    /// GOOD: Borrows self, can be called multiple times
    pub fn e1611_good_get_port(&self) -> u16 {
        self.port
    }

    /// GOOD: Borrows self for inspection
    pub fn e1611_good_is_valid(&self) -> bool {
        self.port > 0 && !self.host.is_empty()
    }

    /// GOOD: Borrows self for formatting
    pub fn e1611_good_to_connection_string(&self) -> String {
        format!("{}:{}", self.host, self.port)
    }

    /// GOOD: Take self by value only for conversions/builders
    pub fn e1611_good_into_address(self) -> std::net::SocketAddr {
        format!("{}:{}", self.host, self.port)
            .parse()
            .expect("invalid address")
    }
}

/// Example of correct usage - config can be reused
pub fn e1611_good_usage() {
    let config = ConfigGood {
        port: 8080,
        host: "127.0.0.1".to_string(),
    };

    // Can use config multiple times!
    let port = config.e1611_good_get_port();
    let is_valid = config.e1611_good_is_valid();
    let conn_str = config.e1611_good_to_connection_string();

    println!("Port: {}, Valid: {}, ConnStr: {}", port, is_valid, conn_str);

    // Can still use config
    let _addr = config.e1611_good_into_address(); // Explicitly consume when needed
}

/// When consuming self IS appropriate:
pub struct Builder {
    value: i32,
}

impl Builder {
    pub fn new() -> Self {
        Builder { value: 0 }
    }

    /// GOOD: Builder pattern - returns Self
    pub fn e1611_good_with_value(mut self, v: i32) -> Self {
        self.value = v;
        self
    }

    /// GOOD: Final build consumes the builder
    pub fn e1611_good_build(self) -> Product {
        Product { value: self.value }
    }
}

impl Default for Builder {
    fn default() -> Self {
        Self::new()
    }
}

pub struct Product {
    pub value: i32,
}

/// GOOD: into_* methods should consume
pub struct Wrapper<T>(T);

impl<T> Wrapper<T> {
    /// GOOD: into_inner explicitly consumes to extract value
    pub fn e1611_good_into_inner(self) -> T {
        self.0
    }
}

// ============================================================================
// GOOD EXAMPLES unit tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_good_config_reuse() {
        let config = ConfigGood {
            port: 8080,
            host: "127.0.0.1".to_string(),
        };

        // Can call multiple times
        assert_eq!(config.e1611_good_get_port(), 8080);
        assert_eq!(config.e1611_good_get_port(), 8080); // Works!
        assert!(config.e1611_good_is_valid());
    }

    #[test]
    fn test_builder_pattern() {
        let product = Builder::new()
            .e1611_good_with_value(42)
            .e1611_good_build();
        assert_eq!(product.value, 42);
    }
}
