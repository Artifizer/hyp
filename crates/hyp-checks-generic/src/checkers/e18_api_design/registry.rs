//! E18 group checker registry.

use crate::{
    checker::Checker,
    checkers::e18_api_design::{
        E1801Config, E1801GlobImports, E1802Config, E1802PublicFields, E1803Config,
        E1803BadNaming, E1804Config, E1804InconsistentErrorTypes, E1805Config,
        E1805MissingDocumentation, E1806Config, E1806ExposingInternalDetails, E1807Config,
        E1807NonIdiomaticBuilder, E1808Config, E1808MutableGetter, E1809Config, E1809FallibleNew,
        E1810Config, E1810StringInsteadOfStr, E1812Config, E1812NonExhaustiveEnum,
    },
    register_checker,
    registry::CheckerRegistration,
};

/// Get all E18 group checker registrations.
pub fn e18_registrations() -> Vec<CheckerRegistration> {
    vec![
        register_checker!(E1801GlobImports, E1801Config),
        register_checker!(E1802PublicFields, E1802Config),
        register_checker!(E1803BadNaming, E1803Config),
        register_checker!(E1804InconsistentErrorTypes, E1804Config),
        register_checker!(E1805MissingDocumentation, E1805Config),
        register_checker!(E1806ExposingInternalDetails, E1806Config),
        register_checker!(E1807NonIdiomaticBuilder, E1807Config),
        register_checker!(E1808MutableGetter, E1808Config),
        register_checker!(E1809FallibleNew, E1809Config),
        register_checker!(E1810StringInsteadOfStr, E1810Config),
        register_checker!(E1812NonExhaustiveEnum, E1812Config),
    ]
}
