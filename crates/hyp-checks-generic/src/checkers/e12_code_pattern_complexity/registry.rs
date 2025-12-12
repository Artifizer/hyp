//! E12 group checker registry.

use crate::{
    checker::Checker,
    checkers::e12_code_pattern_complexity::{
        E1201ComplexGenerics, E1201Config, E1202ComplexLifetimes, E1202Config,
        E1203ComplicatedBorrowing, E1203Config, E1204Config, E1204TraitMethodAmbiguity,
        E1205Config, E1205NestedTraitBounds, E1206Config, E1206NestedGenerics,
        E1207ComplexConstraints, E1207Config, E1208Config, E1208PhantomTypes, E1209Config,
        E1209Hrtb, E1210Config, E1210RecursiveTypes, E1211Config, E1211TraitObjectComplexity,
        E1212Config, E1212GatComplexity, E1213Config, E1213ConstGenericComplexity, E1214Config,
        E1214MacroImpl, E1215Config, E1215TypeLevelProgramming, E1216AssociatedTypeChains,
        E1216Config, E1217AbbaDeadlock, E1217Config,
    },
    register_checker,
    registry::CheckerRegistration,
};

/// Get all E12 group checker registrations.
pub fn e12_registrations() -> Vec<CheckerRegistration> {
    vec![
        register_checker!(E1201ComplexGenerics, E1201Config),
        register_checker!(E1202ComplexLifetimes, E1202Config),
        register_checker!(E1203ComplicatedBorrowing, E1203Config),
        register_checker!(E1204TraitMethodAmbiguity, E1204Config),
        register_checker!(E1205NestedTraitBounds, E1205Config),
        register_checker!(E1206NestedGenerics, E1206Config),
        register_checker!(E1207ComplexConstraints, E1207Config),
        register_checker!(E1208PhantomTypes, E1208Config),
        register_checker!(E1209Hrtb, E1209Config),
        register_checker!(E1210RecursiveTypes, E1210Config),
        register_checker!(E1211TraitObjectComplexity, E1211Config),
        register_checker!(E1212GatComplexity, E1212Config),
        register_checker!(E1213ConstGenericComplexity, E1213Config),
        register_checker!(E1214MacroImpl, E1214Config),
        register_checker!(E1215TypeLevelProgramming, E1215Config),
        register_checker!(E1216AssociatedTypeChains, E1216Config),
        register_checker!(E1217AbbaDeadlock, E1217Config),
    ]
}
