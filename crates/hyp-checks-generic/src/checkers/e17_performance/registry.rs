//! E17 group checker registry.

use crate::{
    checker::Checker,
    checkers::e17_performance::{
        E1701Config, E1701OversizedStructByValue, E1702Config, E1702UnnecessaryAllocation,
        E1703Config, E1703StringConcatInLoop, E1704Config, E1704UnnecessaryCollect,
        E1705Config, E1705CloneInHotPath, E1706Config, E1706NonTailRecursion, E1707Config,
        E1707UnboundedRecursion, E1708Config, E1708InefficientDataStructure, E1709Config,
        E1709UnnecessaryBoxing, E1710Config, E1710LargeStackAllocation, E1712Config,
        E1712ExpensiveOpsInLoop,
    },
    register_checker,
    registry::CheckerRegistration,
};

/// Get all E17 group checker registrations.
pub fn e17_registrations() -> Vec<CheckerRegistration> {
    vec![
        register_checker!(E1701OversizedStructByValue, E1701Config),
        register_checker!(E1702UnnecessaryAllocation, E1702Config),
        register_checker!(E1703StringConcatInLoop, E1703Config),
        register_checker!(E1704UnnecessaryCollect, E1704Config),
        register_checker!(E1705CloneInHotPath, E1705Config),
        register_checker!(E1706NonTailRecursion, E1706Config),
        register_checker!(E1707UnboundedRecursion, E1707Config),
        register_checker!(E1708InefficientDataStructure, E1708Config),
        register_checker!(E1709UnnecessaryBoxing, E1709Config),
        register_checker!(E1710LargeStackAllocation, E1710Config),
        register_checker!(E1712ExpensiveOpsInLoop, E1712Config),
    ]
}
