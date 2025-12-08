//! E14 group checker registry.

use crate::{
    checker::Checker,
    checkers::e14_type_safety::{
        E1401Config, E1401IntegerOverflow, E1402Config, E1402DivisionByZero,
        E1403Config, E1403ModuloByZero, E1404Config, E1404NarrowingConversion,
        E1405Config, E1405IntegerDivisionRounding, E1406Config, E1406SignedUnsignedMismatch,
        E1407Config, E1407LossyFloatConversion, E1408Config, E1408UncheckedIndexing,
        E1409Config, E1409PartialInitialization, E1410Config, E1410FloatEquality,
        E1412Config, E1412ProhibitUnions, E1413Config, E1413SixtyFourIntegerOverflow,
    },
    register_checker,
    registry::CheckerRegistration,
};

/// Get all E14 group checker registrations.
pub fn e14_registrations() -> Vec<CheckerRegistration> {
    vec![
        register_checker!(E1401IntegerOverflow, E1401Config),
        register_checker!(E1402DivisionByZero, E1402Config),
        register_checker!(E1403ModuloByZero, E1403Config),
        register_checker!(E1404NarrowingConversion, E1404Config),
        register_checker!(E1405IntegerDivisionRounding, E1405Config),
        register_checker!(E1406SignedUnsignedMismatch, E1406Config),
        register_checker!(E1407LossyFloatConversion, E1407Config),
        register_checker!(E1408UncheckedIndexing, E1408Config),
        register_checker!(E1409PartialInitialization, E1409Config),
        register_checker!(E1410FloatEquality, E1410Config),
        register_checker!(E1412ProhibitUnions, E1412Config),
        register_checker!(E1413SixtyFourIntegerOverflow, E1413Config),
    ]
}
