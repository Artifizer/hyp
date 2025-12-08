//! Registry for E19 (Code Hygiene) checkers

use crate::checker::Checker;
use crate::checkers::e19_hygiene::{
    E1901AllowedNames, E1901Config, E1902Config, E1902InlineDirectives, E1903Config,
    E1903FileLocation, E1904Config, E1904UnsafeJustification,
};
use crate::{register_checker, CheckerRegistration};

/// Returns all E19 (Code Hygiene) checker registrations
pub fn e19_registrations() -> Vec<CheckerRegistration> {
    vec![
        register_checker!(E1901AllowedNames, E1901Config),
        register_checker!(E1902InlineDirectives, E1902Config),
        register_checker!(E1903FileLocation, E1903Config),
        register_checker!(E1904UnsafeJustification, E1904Config),
    ]
}
