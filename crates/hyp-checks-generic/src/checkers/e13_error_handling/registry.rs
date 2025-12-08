//! E13 group checker registry.

use crate::{
    checker::Checker,
    checkers::e13_error_handling::{
        E1301Config, E1301UnhandledResult, E1302Config, E1302ConstructorWithoutResult,
        E1303Config, E1303IgnoredErrors, E1304Config, E1304UnwrapInErrorPath, E1305Config,
        E1305NonExhaustiveMatch, E1306Config, E1306SwallowedErrors, E1307Config,
        E1307StringErrorType, E1308Config, E1308NotUsingQuestionMark, E1309Config,
        E1309PanicInDrop, E1310Config, E1310ErrorContextLoss,
    },
    register_checker,
    registry::CheckerRegistration,
};

/// Get all E13 group checker registrations.
pub fn e13_registrations() -> Vec<CheckerRegistration> {
    vec![
        register_checker!(E1301UnhandledResult, E1301Config),
        register_checker!(E1302ConstructorWithoutResult, E1302Config),
        register_checker!(E1303IgnoredErrors, E1303Config),
        register_checker!(E1304UnwrapInErrorPath, E1304Config),
        register_checker!(E1305NonExhaustiveMatch, E1305Config),
        register_checker!(E1306SwallowedErrors, E1306Config),
        register_checker!(E1307StringErrorType, E1307Config),
        register_checker!(E1308NotUsingQuestionMark, E1308Config),
        register_checker!(E1309PanicInDrop, E1309Config),
        register_checker!(E1310ErrorContextLoss, E1310Config),
    ]
}
