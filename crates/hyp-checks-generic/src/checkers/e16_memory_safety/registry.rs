//! E16 group checker registry.

use crate::{
    checker::Checker,
    checkers::e16_memory_safety::{
        E1603Config, E1603DanglingReference, E1604BufferOverflow, E1604Config, E1605Config,
        E1605RcCycle, E1606Config, E1606UnnecessaryClone, E1607Config, E1607ForgetDrop,
        E1609Config, E1609InvalidSlice, E1610Config, E1610UnalignedDeref, E1611Config,
        E1611ConsumingSelf,
    },
    register_checker,
    registry::CheckerRegistration,
};

/// Get all E16 group checker registrations.
pub fn e16_registrations() -> Vec<CheckerRegistration> {
    vec![
        register_checker!(E1603DanglingReference, E1603Config),
        register_checker!(E1604BufferOverflow, E1604Config),
        register_checker!(E1605RcCycle, E1605Config),
        register_checker!(E1606UnnecessaryClone, E1606Config),
        register_checker!(E1607ForgetDrop, E1607Config),
        register_checker!(E1609InvalidSlice, E1609Config),
        register_checker!(E1610UnalignedDeref, E1610Config),
        register_checker!(E1611ConsumingSelf, E1611Config),
    ]
}
