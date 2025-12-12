//! E14 - Type safety checkers.

pub mod e1401_integer_overflow;
pub mod e1402_division_by_zero;
pub mod e1403_modulo_by_zero;
pub mod e1404_narrowing_conversion;
pub mod e1405_integer_division_rounding;
pub mod e1406_signed_unsigned_mismatch;
pub mod e1407_lossy_float_conversion;
pub mod e1408_unchecked_indexing;
pub mod e1409_partial_initialization;
pub mod e1410_float_equality;
pub mod e1412_prohibit_unions;
pub mod e1413_64bit_integer_overflow;
pub mod registry;

pub use e1401_integer_overflow::{E1401Config, E1401IntegerOverflow};
pub use e1402_division_by_zero::{E1402Config, E1402DivisionByZero};
pub use e1403_modulo_by_zero::{E1403Config, E1403ModuloByZero};
pub use e1404_narrowing_conversion::{E1404Config, E1404NarrowingConversion};
pub use e1405_integer_division_rounding::{E1405Config, E1405IntegerDivisionRounding};
pub use e1406_signed_unsigned_mismatch::{E1406Config, E1406SignedUnsignedMismatch};
pub use e1407_lossy_float_conversion::{E1407Config, E1407LossyFloatConversion};
pub use e1408_unchecked_indexing::{E1408Config, E1408UncheckedIndexing};
pub use e1409_partial_initialization::{E1409Config, E1409PartialInitialization};
pub use e1410_float_equality::{E1410Config, E1410FloatEquality};
pub use e1412_prohibit_unions::{E1412Config, E1412ProhibitUnions};
pub use e1413_64bit_integer_overflow::{E1413Config, E1413SixtyFourIntegerOverflow};
