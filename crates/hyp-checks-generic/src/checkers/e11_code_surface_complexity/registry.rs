//! E11 group checker registry.

use crate::{
    checker::Checker,
    checkers::e11_code_surface_complexity::{
        E1101Config, E1101HighCyclomaticComplexity, E1102Config, E1102DeeplyNestedLogic,
        E1103Config, E1103TooManyParameters, E1104Config, E1104LargeStruct,
        E1105Config, E1105BooleanParameterHell, E1106Config, E1106LongFunction,
        E1107Config, E1107DeeplyNestedConditionals, E1108Config, E1108DeeplyNestedMatch,
        E1109Config, E1109ExcessiveChaining, E1110Config, E1110DeeplyNestedClosures,
        E1111Config, E1111ExcessiveTupleComplexity, E1112Config, E1112MagicNumbers,
    },
    register_checker,
    registry::CheckerRegistration,
};

/// Get all E11 group checker registrations.
pub fn e11_registrations() -> Vec<CheckerRegistration> {
    vec![
        register_checker!(E1101HighCyclomaticComplexity, E1101Config),
        register_checker!(E1102DeeplyNestedLogic, E1102Config),
        register_checker!(E1103TooManyParameters, E1103Config),
        register_checker!(E1104LargeStruct, E1104Config),
        register_checker!(E1105BooleanParameterHell, E1105Config),
        register_checker!(E1106LongFunction, E1106Config),
        register_checker!(E1107DeeplyNestedConditionals, E1107Config),
        register_checker!(E1108DeeplyNestedMatch, E1108Config),
        register_checker!(E1109ExcessiveChaining, E1109Config),
        register_checker!(E1110DeeplyNestedClosures, E1110Config),
        register_checker!(E1111ExcessiveTupleComplexity, E1111Config),
        register_checker!(E1112MagicNumbers, E1112Config),
    ]
}
