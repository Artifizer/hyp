//! Registry for E19 (Code Hygiene) checkers

use crate::checker::Checker;
use crate::checkers::e19_hygiene::{
    E1901Config, E1901CriticalLintOverride, E1902Config, E1902MediumLintOverride, E1903Config,
    E1903MinorLintOverride, E1904AllowedNames, E1904Config, E1905Config, E1905SuspiciousCode,
    E1906Config, E1906FileLocation, E1907Config, E1907TestCoverageAttr, E1908Config,
    E1908UnsafeJustification,
};
use crate::{register_checker, CheckerRegistration};

/// Returns all E19 (Code Hygiene) checker registrations
pub fn e19_registrations() -> Vec<CheckerRegistration> {
    vec![
        register_checker!(E1904AllowedNames, E1904Config),
        register_checker!(E1905SuspiciousCode, E1905Config),
        register_checker!(E1906FileLocation, E1906Config),
        register_checker!(E1907TestCoverageAttr, E1907Config),
        register_checker!(E1908UnsafeJustification, E1908Config),
        register_checker!(E1901CriticalLintOverride, E1901Config),
        register_checker!(E1902MediumLintOverride, E1902Config),
        register_checker!(E1903MinorLintOverride, E1903Config),
    ]
}
