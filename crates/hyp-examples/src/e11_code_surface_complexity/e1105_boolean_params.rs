/// E1105: Boolean parameter hell
/// Severity: LOW
/// LLM confusion: 2 (LOW)
///
/// Description: This function takes multiple boolean (true/false) parameters, making it very
/// confusing at the call site. When you see `func(true, false, true, false)`, it's impossible
/// to know what each boolean means without checking the function signature. Fix by using an
/// enum or a configuration struct with named fields instead of boolean parameters.
///
/// Mitigation: Use `#![warn(clippy::fn_params_excessive_bools)]` to detect functions with too
/// many boolean parameters. Replace boolean parameters with enums that have descriptive names,
/// or use a builder pattern with named methods.

pub fn e1105_bad_boolean_params(
    enable_feature_a: bool,
    enable_feature_b: bool,
    enable_feature_c: bool,
    enable_feature_d: bool,
) {
    // PROBLEM E1105: Multiple boolean parameters (confusing at call site)
    if enable_feature_a && enable_feature_b {
        // do something
    }
}

pub fn e1105_entry() -> Result<(), Box<dyn std::error::Error>> {
    e1105_bad_boolean_params(true, false, true, false);
    Ok(())
}

// ============================================================================
// GOOD EXAMPLES - Proper alternatives
// ============================================================================

/// GOOD: Use enums instead of booleans
#[derive(Clone, Copy)]
pub enum FeatureA {
    Enabled,
    Disabled,
}

#[derive(Clone, Copy)]
pub enum FeatureB {
    Enabled,
    Disabled,
}

pub fn e1105_good_enums(feature_a: FeatureA, feature_b: FeatureB) {
    // Call site is now clear: e1105_good_enums(FeatureA::Enabled, FeatureB::Disabled)
    if matches!(feature_a, FeatureA::Enabled) && matches!(feature_b, FeatureB::Enabled) {
        // do something
    }
}

/// GOOD: Use a configuration struct with named fields
#[derive(Default)]
pub struct FeatureFlags {
    pub feature_a: bool,
    pub feature_b: bool,
    pub feature_c: bool,
    pub feature_d: bool,
}

pub fn e1105_good_config_struct(flags: FeatureFlags) {
    // Call site is now clear with named fields
    if flags.feature_a && flags.feature_b {
        // do something
    }
}

/// GOOD: Use builder pattern for complex flag combinations
pub struct FlagsBuilder {
    flags: FeatureFlags,
}

impl FlagsBuilder {
    pub fn new() -> Self {
        Self { flags: FeatureFlags::default() }
    }

    pub fn e1105_good_enable_feature_a(mut self) -> Self {
        self.flags.feature_a = true;
        self
    }

    pub fn e1105_good_enable_feature_b(mut self) -> Self {
        self.flags.feature_b = true;
        self
    }

    pub fn e1105_good_build(self) -> FeatureFlags {
        self.flags
    }
}

impl Default for FlagsBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// GOOD: Use bitflags for performance-critical cases
pub mod feature_flags {
    pub const FEATURE_A: u8 = 0b0001;
    pub const FEATURE_B: u8 = 0b0010;
    pub const FEATURE_C: u8 = 0b0100;
    pub const FEATURE_D: u8 = 0b1000;
}

pub fn e1105_good_bitflags(flags: u8) {
    // Call site: e1105_good_bitflags(FEATURE_A | FEATURE_C)
    if flags & feature_flags::FEATURE_A != 0 && flags & feature_flags::FEATURE_B != 0 {
        // do something
    }
}

// ============================================================================
// GOOD EXAMPLES unit tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn e1105_good_enums_are_explicit() {
        e1105_good_enums(FeatureA::Enabled, FeatureB::Disabled);
    }

    #[test]
    fn e1105_good_config_struct_uses_named_fields() {
        let flags = FeatureFlags {
            feature_a: true,
            feature_b: false,
            ..Default::default()
        };
        e1105_good_config_struct(flags);
    }

    #[test]
    fn e1105_good_builder_sets_feature_a() {
        let flags = FlagsBuilder::new()
            .e1105_good_enable_feature_a()
            .e1105_good_build();
        assert!(flags.feature_a);
    }
}
