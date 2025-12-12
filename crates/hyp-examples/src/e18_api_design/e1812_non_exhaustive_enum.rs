/// E1812: Public enum without #[non_exhaustive]
/// Severity: LOW
/// LLM confusion: 2 (LOW)
///
/// Description: Public enums that might grow in the future should be marked with
/// #[non_exhaustive] to allow adding variants without breaking downstream code.
/// Without it, adding a new variant is a breaking change that forces all users
/// to update their match statements.
///
/// Mitigation: Add #[non_exhaustive] to public enums that might gain variants,
/// or document that the enum is intentionally closed (like Option/Result).

/// PROBLEM E1812: Public enum without non_exhaustive
/// Adding a variant later will break all downstream match statements!
pub enum Status {
    Active,
    Inactive,
    Pending,
}

/// PROBLEM E1812: Error enum that might grow
pub enum ApiError {
    NotFound,
    Unauthorized,
    BadRequest,
    // Adding ServerError later breaks all matches!
}

/// PROBLEM E1812: State machine enum likely to evolve
pub enum OrderState {
    Created,
    Paid,
    Shipped,
    Delivered,
    // What about Cancelled, Refunded, OnHold?
}

// Downstream code that will break when variants are added:
pub fn e1812_bad_handle_status(status: &Status) -> &'static str {
    match status {
        Status::Active => "active",
        Status::Inactive => "inactive",
        Status::Pending => "pending",
        // If Status::Suspended is added, this won't compile!
    }
}

pub fn e1812_entry() -> Result<(), Box<dyn std::error::Error>> {
    let status = Status::Active;
    let _ = e1812_bad_handle_status(&status);
    Ok(())
}

// ============================================================================
// GOOD EXAMPLES - Using #[non_exhaustive]
// ============================================================================

/// GOOD: Non-exhaustive public enum
#[non_exhaustive]
pub enum StatusGood {
    Active,
    Inactive,
    Pending,
}

/// GOOD: Error types should almost always be non_exhaustive
#[non_exhaustive]
#[derive(Debug)]
pub enum ApiErrorGood {
    NotFound,
    Unauthorized,
    BadRequest,
    // Can add more without breaking downstream!
}

/// GOOD: State machines that evolve
#[non_exhaustive]
pub enum OrderStateGood {
    Created,
    Paid,
    Shipped,
    Delivered,
}

// Downstream code MUST handle unknown variants:
// Note: Allow unreachable patterns since we're in the same crate.
// External crates using this enum MUST have the wildcard.
#[allow(unreachable_patterns)]
pub fn e1812_handle_status_good(status: &StatusGood) -> &'static str {
    match status {
        StatusGood::Active => "active",
        StatusGood::Inactive => "inactive",
        StatusGood::Pending => "pending",
        // Wildcard required due to #[non_exhaustive] when used from other crates
        _ => "unknown",
    }
}

// Alternatively, handle specific cases and have a catch-all:
// Note: Allow unreachable patterns since we're in the same crate.
// External crates using this enum MUST have the wildcard.
#[allow(unreachable_patterns)]
pub fn e1812_handle_order_state(state: &OrderStateGood) -> &'static str {
    match state {
        OrderStateGood::Created => "Your order has been created",
        OrderStateGood::Paid => "Payment received",
        OrderStateGood::Shipped => "Your order is on the way",
        OrderStateGood::Delivered => "Your order has been delivered",
        _ => "Unknown order state",
    }
}

/// When non_exhaustive is NOT needed:

/// OK: Private enum (can't be matched externally anyway)
enum InternalState {
    Ready,
    Processing,
    Done,
}

fn _use_internal() {
    let _ = InternalState::Ready;
}

/// OK: Intentionally closed enum (like boolean states)
pub enum Toggle {
    On,
    Off,
}

/// OK: Enums that are truly exhaustive by nature
pub enum Comparison {
    Less,
    Equal,
    Greater,
}

// ============================================================================
// GOOD EXAMPLES unit tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_good_status_handling() {
        assert_eq!(e1812_handle_status_good(&StatusGood::Active), "active");
        assert_eq!(e1812_handle_status_good(&StatusGood::Pending), "pending");
    }

    #[test]
    fn test_order_state() {
        assert_eq!(
            e1812_handle_order_state(&OrderStateGood::Shipped),
            "Your order is on the way"
        );
    }
}
