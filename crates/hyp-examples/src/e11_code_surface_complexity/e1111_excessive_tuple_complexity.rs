/// E1111: Excessive tuple complexity
/// Severity: LOW
/// LLM confusion: 2 (LOW)
///
/// Description: Tuples with many elements are hard to read and maintain. When you see
/// `(i32, i32, i32, i32, i32)`, it's unclear what each element represents. Named struct
/// fields provide clarity and prevent mistakes when accessing or passing data. Large
/// tuples also make refactoring difficult and are prone to ordering errors.
///
/// Mitigation: Replace tuples with more than 3 elements with structs that have named
/// fields. This makes code self-documenting and easier to maintain.

/// PROBLEM E1111: Large tuple for user data - unclear what each field means
pub fn e1111_bad_create_user() -> (String, u32, String, String, bool, bool) {
    (
        "Alice".to_string(),
        30,
        "alice@example.com".to_string(),
        "Engineering".to_string(),
        true,
        false,
    )
}

/// PROBLEM E1111: Large tuple in function parameter - easy to mix up arguments
pub fn e1111_bad_process_coordinates(coords: (f64, f64, f64, f64, f64, f64)) -> f64 {
    coords.0 + coords.1 + coords.2 + coords.3 + coords.4 + coords.5
}

/// PROBLEM E1111: Large tuple pattern destructuring - hard to track which is which
pub fn e1111_bad_parse_config() -> (String, u32, u32, bool, String, String) {
    (
        "localhost".to_string(),
        8080,
        100,
        false,
        "admin".to_string(),
        "secret".to_string(),
    )
}

pub fn e1111_bad_use_config() {
    let (host, port, max_conn, ssl, user, pass) = e1111_bad_parse_config();
    let _ = (host, port, max_conn, ssl, user, pass);
}

/// PROBLEM E1111: Nested tuple complexity - extremely hard to understand
pub fn e1111_bad_complex_data() -> ((i32, i32, i32, i32, i32, i32), (String, String, String, String, String, String)) {
    ((1, 2, 3, 4, 5, 6), ("a".into(), "b".into(), "c".into(), "d".into(), "e".into(), "f".into()))
}

/// PROBLEM E1111: Large tuple in struct field
pub struct E1111BadSensorData {
    pub readings: (f64, f64, f64, f64, f64, f64),
}

pub fn e1111_entry() -> Result<(), Box<dyn std::error::Error>> {
    let _ = e1111_bad_create_user();
    let _ = e1111_bad_process_coordinates((1.0, 2.0, 3.0, 4.0, 5.0, 6.0));
    e1111_bad_use_config();
    let _ = e1111_bad_complex_data();
    let _ = E1111BadSensorData {
        readings: (1.0, 2.0, 3.0, 4.0, 5.0, 6.0),
    };
    Ok(())
}

// ============================================================================
// GOOD EXAMPLES - Named structs with clear fields
// ============================================================================

/// GOOD: Self-documenting struct with named fields
pub struct User {
    pub name: String,
    pub age: u32,
    pub email: String,
    pub department: String,
    pub is_active: bool,
}

pub fn e1111_good_create_user() -> User {
    User {
        name: "Alice".to_string(),
        age: 30,
        email: "alice@example.com".to_string(),
        department: "Engineering".to_string(),
        is_active: true,
    }
}

/// GOOD: Named struct for coordinates
pub struct Coordinates {
    pub x: f64,
    pub y: f64,
    pub z: f64,
    pub velocity: f64,
    pub acceleration: f64,
}

pub fn e1111_good_process_coordinates(coords: &Coordinates) -> f64 {
    coords.x + coords.y + coords.z + coords.velocity + coords.acceleration
}

/// GOOD: Configuration struct with clear field names
pub struct ServerConfig {
    pub host: String,
    pub port: u32,
    pub max_connections: u32,
    pub use_ssl: bool,
    pub admin_user: String,
    pub admin_password: String,
}

pub fn e1111_good_parse_config() -> ServerConfig {
    ServerConfig {
        host: "localhost".to_string(),
        port: 8080,
        max_connections: 100,
        use_ssl: false,
        admin_user: "admin".to_string(),
        admin_password: "secret".to_string(),
    }
}

/// GOOD: Small tuples (2-3 elements) are acceptable for simple cases
pub fn e1111_good_get_dimensions() -> (u32, u32) {
    (1920, 1080) // Width and height - clear from context
}

/// GOOD: Point3D with just 3 coordinates is fine
pub fn e1111_good_get_point() -> (f64, f64, f64) {
    (1.0, 2.0, 3.0) // x, y, z - common pattern
}

/// GOOD: Sensor data with named fields
pub struct SensorReadings {
    pub temperature: f64,
    pub humidity: f64,
    pub pressure: f64,
    pub light_level: f64,
    pub sound_level: f64,
    pub air_quality: f64,
}

// ============================================================================
// GOOD EXAMPLES unit tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn e1111_good_user_creation() {
        let user = e1111_good_create_user();
        assert_eq!(user.name, "Alice");
        assert_eq!(user.age, 30);
        assert!(user.is_active);
    }

    #[test]
    fn e1111_good_coordinates_processing() {
        let coords = Coordinates {
            x: 1.0,
            y: 2.0,
            z: 3.0,
            velocity: 4.0,
            acceleration: 5.0,
        };
        let sum = e1111_good_process_coordinates(&coords);
        assert_eq!(sum, 15.0);
    }

    #[test]
    fn e1111_good_config_parsing() {
        let config = e1111_good_parse_config();
        assert_eq!(config.host, "localhost");
        assert_eq!(config.port, 8080);
        assert!(!config.use_ssl);
    }

    #[test]
    fn e1111_good_small_tuples_ok() {
        let (width, height) = e1111_good_get_dimensions();
        assert_eq!(width, 1920);
        assert_eq!(height, 1080);
    }
}
