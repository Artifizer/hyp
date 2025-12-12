//! Checker registry for external registration and grouping.

use crate::{
    checker::{Checker, CheckerDescriptor},
    checkers::{e10_unsafe_code, e11_code_surface_complexity, e12_code_pattern_complexity, e13_error_handling, e14_type_safety, e15_concurrency, e16_memory_safety, e17_performance, e18_api_design, e19_hygiene},
    config::AnalyzerConfig,
};

/// A factory function that creates a checker instance from config.
pub type CheckerFactory = fn(&AnalyzerConfig) -> Option<Box<dyn Checker>>;

/// Registry entry for a checker.
pub struct CheckerRegistration {
    /// Descriptor with default metadata.
    pub descriptor: CheckerDescriptor,
    /// Factory function to create the checker.
    pub factory: CheckerFactory,
}

/// Logical groups of checkers (e.g. by problem family like e10, e11, ...).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CheckerGroup {
    /// E10 - unsafe code / panic-related checkers.
    E10,
    /// E11 - complexity-related checkers.
    E11,
    /// E12 - pattern complexity checkers.
    E12,
    /// E13 - error handling checkers.
    E13,
    /// E14 - type safety checkers.
    E14,
    /// E15 - concurrency checkers.
    E15,
    /// E16 - memory safety checkers.
    E16,
    /// E17 - performance checkers.
    E17,
    /// E18 - API design checkers.
    E18,
    /// E19 - Code hygiene checkers.
    E19,
}

/// All checkers that belong to the E10 group.
pub fn group_e10() -> Vec<CheckerRegistration> {
    e10_unsafe_code::registry::e10_registrations()
}

/// All checkers that belong to the E11 group.
pub fn group_e11() -> Vec<CheckerRegistration> {
    e11_code_surface_complexity::registry::e11_registrations()
}

/// All checkers that belong to the E12 group.
pub fn group_e12() -> Vec<CheckerRegistration> {
    e12_code_pattern_complexity::registry::e12_registrations()
}

/// All checkers that belong to the E13 group.
pub fn group_e13() -> Vec<CheckerRegistration> {
    e13_error_handling::registry::e13_registrations()
}

/// All checkers that belong to the E14 group.
pub fn group_e14() -> Vec<CheckerRegistration> {
    e14_type_safety::registry::e14_registrations()
}

/// All checkers that belong to the E15 group.
pub fn group_e15() -> Vec<CheckerRegistration> {
    e15_concurrency::registry::e15_registrations()
}

/// All checkers that belong to the E16 group.
pub fn group_e16() -> Vec<CheckerRegistration> {
    e16_memory_safety::registry::e16_registrations()
}

/// All checkers that belong to the E17 group.
pub fn group_e17() -> Vec<CheckerRegistration> {
    e17_performance::registry::e17_registrations()
}

/// All checkers that belong to the E18 group.
pub fn group_e18() -> Vec<CheckerRegistration> {
    e18_api_design::registry::e18_registrations()
}

/// All checkers that belong to the E19 group.
pub fn group_e19() -> Vec<CheckerRegistration> {
    e19_hygiene::registry::e19_registrations()
}

/// Build a flat list of registrations for the given groups.
/// CLIs can use this to register an entire family of checkers at once.
pub fn checkers_for_groups(groups: &[CheckerGroup]) -> Vec<CheckerRegistration> {
    let mut out = Vec::new();
    for g in groups {
        match g {
            CheckerGroup::E10 => out.extend(group_e10()),
            CheckerGroup::E11 => out.extend(group_e11()),
            CheckerGroup::E12 => out.extend(group_e12()),
            CheckerGroup::E13 => out.extend(group_e13()),
            CheckerGroup::E14 => out.extend(group_e14()),
            CheckerGroup::E15 => out.extend(group_e15()),
            CheckerGroup::E16 => out.extend(group_e16()),
            CheckerGroup::E17 => out.extend(group_e17()),
            CheckerGroup::E18 => out.extend(group_e18()),
            CheckerGroup::E19 => out.extend(group_e19()),
        }
    }
    out
}

/// Get all available checker registrations (all groups combined).
pub fn get_all_checkers() -> Vec<CheckerRegistration> {
    checkers_for_groups(&[
        CheckerGroup::E10,
        CheckerGroup::E11,
        CheckerGroup::E12,
        CheckerGroup::E13,
        CheckerGroup::E14,
        CheckerGroup::E15,
        CheckerGroup::E16,
        CheckerGroup::E17,
        CheckerGroup::E18,
        CheckerGroup::E19,
    ])
}
