//! E11 - Code surface complexity checkers.

pub mod e1101_high_cyclomatic_complexity;
pub mod e1102_deeply_nested_logic;
pub mod e1103_too_many_parameters;
pub mod e1104_large_struct;
pub mod e1105_boolean_parameter_hell;
pub mod e1106_long_function;
pub mod e1107_deeply_nested_conditionals;
pub mod e1108_deeply_nested_match;
pub mod e1109_excessive_chaining;
pub mod e1110_deeply_nested_closures;
pub mod e1111_excessive_tuple_complexity;
pub mod e1112_magic_numbers;
pub mod registry;

pub use e1101_high_cyclomatic_complexity::{E1101Config, E1101HighCyclomaticComplexity};
pub use e1102_deeply_nested_logic::{E1102Config, E1102DeeplyNestedLogic};
pub use e1103_too_many_parameters::{E1103Config, E1103TooManyParameters};
pub use e1104_large_struct::{E1104Config, E1104LargeStruct};
pub use e1105_boolean_parameter_hell::{E1105Config, E1105BooleanParameterHell};
pub use e1106_long_function::{E1106Config, E1106LongFunction};
pub use e1107_deeply_nested_conditionals::{E1107Config, E1107DeeplyNestedConditionals};
pub use e1108_deeply_nested_match::{E1108Config, E1108DeeplyNestedMatch};
pub use e1109_excessive_chaining::{E1109Config, E1109ExcessiveChaining};
pub use e1110_deeply_nested_closures::{E1110Config, E1110DeeplyNestedClosures};
pub use e1111_excessive_tuple_complexity::{E1111Config, E1111ExcessiveTupleComplexity};
pub use e1112_magic_numbers::{E1112Config, E1112MagicNumbers};
