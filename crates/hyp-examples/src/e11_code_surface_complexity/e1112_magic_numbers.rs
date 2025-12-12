/// E1112: Hardcoded magic numbers
/// Severity: LOW
/// LLM confusion: 2 (LOW)
///
/// Description: Using unexplained numeric literals in code makes it hard to understand
/// and maintain. Named constants provide context and prevent typos. Magic numbers
/// obscure the meaning of the code and make it difficult to update values consistently
/// across the codebase.
///
/// Mitigation: Extract magic numbers into named constants using `const NAME: Type = value`.
/// Group related constants together and document their purpose.

/// PROBLEM E1112: Magic numbers in shipping calculation
pub fn e1112_bad_shipping_calculation(weight_kg: f64, distance_km: f64) -> f64 {
    weight_kg * 2.5 + distance_km * 0.15 + 499.0 // What do these numbers mean?
}

/// PROBLEM E1112: Magic timeout and retry values
pub fn e1112_bad_connect_with_retry() -> bool {
    for _ in 0..3 {
        // What is 3? Max retries?
        std::thread::sleep(std::time::Duration::from_millis(1500)); // Why 1500?
        if e1112_bad_try_connect() {
            return true;
        }
    }
    false
}

fn e1112_bad_try_connect() -> bool {
    false // Stub
}

/// PROBLEM E1112: Magic buffer size
pub fn e1112_bad_read_file_chunked() -> Vec<u8> {
    let mut buffer = vec![0u8; 8192]; // Why 8192?
    let _ = &mut buffer;
    vec![]
}

/// PROBLEM E1112: Magic discount thresholds
pub fn e1112_bad_calculate_discount(total: f64, items: u32) -> f64 {
    if total > 100.0 && items >= 5 {
        total * 0.15 // 15% discount - but why these thresholds?
    } else if total > 50.0 {
        total * 0.05 // 5% discount
    } else {
        0.0
    }
}

pub fn e1112_entry() -> Result<(), Box<dyn std::error::Error>> {
    let _ = e1112_bad_shipping_calculation(10.0, 50.0);
    let _ = e1112_bad_connect_with_retry();
    let _ = e1112_bad_calculate_discount(120.0, 6);
    let _ = e1112_bad_read_file_chunked();
    Ok(())
}

// ============================================================================
// GOOD EXAMPLES - Named constants
// ============================================================================

// Shipping constants with clear names
const WEIGHT_RATE_PER_KG: f64 = 2.5;
const DISTANCE_RATE_PER_KM: f64 = 0.15;
const BASE_SHIPPING_FEE_CENTS: i32 = 499;

/// GOOD: Self-documenting with named constants
pub fn e1112_good_shipping(weight_kg: f64, distance_km: f64) -> f64 {
    weight_kg * WEIGHT_RATE_PER_KG
        + distance_km * DISTANCE_RATE_PER_KM
        + BASE_SHIPPING_FEE_CENTS as f64
}

// Connection constants
const MAX_CONNECTION_RETRIES: u32 = 3;
const RETRY_DELAY_MS: u64 = 1500;

fn e1112_good_try_connect() -> bool {
    false
}

/// GOOD: Clear retry logic with named constants
pub fn e1112_good_connect_with_retry() -> bool {
    for _ in 0..MAX_CONNECTION_RETRIES {
        std::thread::sleep(std::time::Duration::from_millis(RETRY_DELAY_MS));
        if e1112_good_try_connect() {
            return true;
        }
    }
    false
}

// Business rule constants
const BULK_DISCOUNT_THRESHOLD: f64 = 100.0;
const BULK_DISCOUNT_MIN_ITEMS: u32 = 5;
const BULK_DISCOUNT_PERCENT: f64 = 0.15;
const SMALL_DISCOUNT_THRESHOLD: f64 = 50.0;
const SMALL_DISCOUNT_PERCENT: f64 = 0.05;

/// GOOD: Business logic is self-documenting
pub fn e1112_good_calculate_discount(total: f64, items: u32) -> f64 {
    if total > BULK_DISCOUNT_THRESHOLD && items >= BULK_DISCOUNT_MIN_ITEMS {
        total * BULK_DISCOUNT_PERCENT
    } else if total > SMALL_DISCOUNT_THRESHOLD {
        total * SMALL_DISCOUNT_PERCENT
    } else {
        0.0
    }
}

// ============================================================================
// GOOD EXAMPLES unit tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn e1112_good_shipping_calculation() {
        let cost = e1112_good_shipping(10.0, 100.0);
        assert!(cost > 0.0);
    }

    #[test]
    fn e1112_good_discount_bulk() {
        let discount = e1112_good_calculate_discount(150.0, 6);
        assert!((discount - 22.5).abs() < 0.01); // 15% of 150
    }
}
