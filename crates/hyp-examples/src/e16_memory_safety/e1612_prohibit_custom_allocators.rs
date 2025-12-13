/// E1612: Prohibit custom allocators
/// Severity: MEDIUM
/// LLM confusion: 2 (LOW) - LLMs sometimes suggest jemalloc for "performance"
///
/// Description: Custom allocators (#[global_allocator]) add complexity that
/// microservices rarely need. They are appropriate for HPC, low-latency trading,
/// embedded systems, and real-time game loops, but not for typical I/O-bound
/// microservices.
///
/// Why this matters:
/// 1. Adds debugging complexity - memory issues become harder to diagnose
/// 2. Breaks common tools - profilers and sanitizers assume default allocator
/// 3. Risk of subtle bugs - custom allocator implementations require expertise
/// 4. Minimal benefit - microservices are I/O-bound, not allocation-bound

// ============================================================================
// BAD EXAMPLES - Custom allocators in microservices
// ============================================================================

// PROBLEM E1612: Using jemalloc when the default allocator would suffice
// BAD: This adds complexity without measurable benefit
// #[global_allocator]
// static GLOBAL: tikv_jemallocator::Jemalloc = tikv_jemallocator::Jemalloc;

// PROBLEM E1612: Using mimalloc
// BAD: Premature optimization for a web service
// #[global_allocator]
// static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

// PROBLEM E1612: Even System allocator override is unnecessary
// BAD: If you want the default, just don't specify one!
// #[global_allocator]
// static GLOBAL: std::alloc::System = std::alloc::System;

/// Example of code that claims to need a custom allocator but doesn't
pub mod unnecessary_allocator {
    /// This HTTP handler does not benefit from jemalloc
    /// The bottleneck is I/O (network, database), not memory allocation
    pub async fn handle_request(_request: &str) -> String {
        // Simulated work - I/O bound, not memory bound
        let response = format!("Hello, world! Processing: {}", _request);
        response
    }

    /// Even with many allocations, the default allocator is fine
    pub fn process_data(items: Vec<u32>) -> Vec<u32> {
        items.into_iter().map(|x| x * 2).collect()
    }
}

// ============================================================================
// GOOD EXAMPLES - When NOT using custom allocators
// ============================================================================

/// GOOD: Just use the default allocator (no code needed!)
/// The default allocator is:
/// - Well-tested across all platforms
/// - Compatible with debugging tools (valgrind, sanitizers, heaptrack)
/// - Continuously optimized by the Rust team
/// - Good enough for I/O-bound services
pub mod default_allocator {
    /// Simple microservice handler - default allocator is perfect
    pub fn handle_request(data: &str) -> String {
        format!("Processed: {}", data)
    }

    /// Data processing - still fine with default allocator
    pub fn process_batch(items: Vec<String>) -> Vec<String> {
        items.into_iter().map(|s| s.to_uppercase()).collect()
    }
}

// ============================================================================
// LEGITIMATE USE CASES (where custom allocators ARE appropriate)
// ============================================================================

// Note: These are legitimate use cases where you might want to disable E1612

/// 1. High-Performance Computing (HPC)
/// When processing terabytes of data with precise memory requirements
// #[global_allocator]
// static GLOBAL: jemalloc = jemalloc; // Appropriate for HPC

/// 2. Low-latency trading systems
/// When microseconds matter and you need predictable allocation times
// #[global_allocator]
// static GLOBAL: mimalloc = mimalloc; // Appropriate for trading

/// 3. Embedded systems
/// When you need custom memory regions or have no heap
// Custom allocator for embedded is legitimate

/// 4. Real-time game loops
/// When you need deterministic memory behavior per frame
// Arena allocator per frame is legitimate

pub fn e1612_entry() -> Result<(), Box<dyn std::error::Error>> {
    // This example doesn't actually use custom allocators
    // The BAD examples are commented out to avoid compilation issues
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_allocator_works() {
        let result = default_allocator::handle_request("test");
        assert!(result.contains("test"));
    }
}
