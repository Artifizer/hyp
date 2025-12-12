//! E18 - API design checkers.

pub mod e1801_glob_imports;
pub mod e1802_public_fields;
pub mod e1803_bad_naming;
pub mod e1804_inconsistent_error_types;
pub mod e1805_missing_documentation;
pub mod e1806_exposing_internal_details;
pub mod e1807_non_idiomatic_builder;
pub mod e1808_mutable_getter;
pub mod e1809_fallible_new;
pub mod e1810_string_instead_of_str;
pub mod e1812_non_exhaustive_enum;
pub mod registry;

pub use e1801_glob_imports::{E1801Config, E1801GlobImports};
pub use e1802_public_fields::{E1802Config, E1802PublicFields};
pub use e1803_bad_naming::{E1803Config, E1803BadNaming};
pub use e1804_inconsistent_error_types::{E1804Config, E1804InconsistentErrorTypes};
pub use e1805_missing_documentation::{E1805Config, E1805MissingDocumentation};
pub use e1806_exposing_internal_details::{E1806Config, E1806ExposingInternalDetails};
pub use e1807_non_idiomatic_builder::{E1807Config, E1807NonIdiomaticBuilder};
pub use e1808_mutable_getter::{E1808Config, E1808MutableGetter};
pub use e1809_fallible_new::{E1809Config, E1809FallibleNew};
pub use e1810_string_instead_of_str::{E1810Config, E1810StringInsteadOfStr};
pub use e1812_non_exhaustive_enum::{E1812Config, E1812NonExhaustiveEnum};
