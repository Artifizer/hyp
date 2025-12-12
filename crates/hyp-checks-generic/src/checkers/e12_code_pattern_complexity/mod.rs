//! E12 - Code pattern complexity checkers.

pub mod e1201_complex_generics;
pub mod e1202_complex_lifetimes;
pub mod e1203_complicated_borrowing;
pub mod e1204_trait_method_ambiguity;
pub mod e1205_nested_trait_bounds;
pub mod e1206_nested_generics;
pub mod e1207_complex_constraints;
pub mod e1208_phantom_types;
pub mod e1209_hrtb;
pub mod e1210_recursive_types;
pub mod e1211_trait_object_complexity;
pub mod e1212_gat_complexity;
pub mod e1213_const_generic_complexity;
pub mod e1214_macro_impl;
pub mod e1215_type_level_programming;
pub mod e1216_associated_type_chains;
pub mod e1217_abba_deadlock;
pub mod registry;

pub use e1201_complex_generics::{E1201ComplexGenerics, E1201Config};
pub use e1202_complex_lifetimes::{E1202ComplexLifetimes, E1202Config};
pub use e1203_complicated_borrowing::{E1203ComplicatedBorrowing, E1203Config};
pub use e1204_trait_method_ambiguity::{E1204Config, E1204TraitMethodAmbiguity};
pub use e1205_nested_trait_bounds::{E1205Config, E1205NestedTraitBounds};
pub use e1206_nested_generics::{E1206Config, E1206NestedGenerics};
pub use e1207_complex_constraints::{E1207ComplexConstraints, E1207Config};
pub use e1208_phantom_types::{E1208Config, E1208PhantomTypes};
pub use e1209_hrtb::{E1209Config, E1209Hrtb};
pub use e1210_recursive_types::{E1210Config, E1210RecursiveTypes};
pub use e1211_trait_object_complexity::{E1211Config, E1211TraitObjectComplexity};
pub use e1212_gat_complexity::{E1212Config, E1212GatComplexity};
pub use e1213_const_generic_complexity::{E1213Config, E1213ConstGenericComplexity};
pub use e1214_macro_impl::{E1214Config, E1214MacroImpl};
pub use e1215_type_level_programming::{E1215Config, E1215TypeLevelProgramming};
pub use e1216_associated_type_chains::{E1216AssociatedTypeChains, E1216Config};
pub use e1217_abba_deadlock::{E1217AbbaDeadlock, E1217Config};
