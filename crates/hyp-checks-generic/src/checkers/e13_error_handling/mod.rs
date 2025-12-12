//! E13 - Error handling checkers.

pub mod e1301_unhandled_result;
pub mod e1302_constructor_without_result;
pub mod e1303_ignored_errors;
pub mod e1304_unwrap_in_error_path;
pub mod e1305_non_exhaustive_match;
pub mod e1306_swallowed_errors;
pub mod e1307_string_error_type;
pub mod e1308_not_using_question_mark;
pub mod e1309_panic_in_drop;
pub mod e1310_error_context_loss;
pub mod registry;

pub use e1301_unhandled_result::{E1301Config, E1301UnhandledResult};
pub use e1302_constructor_without_result::{E1302Config, E1302ConstructorWithoutResult};
pub use e1303_ignored_errors::{E1303Config, E1303IgnoredErrors};
pub use e1304_unwrap_in_error_path::{E1304Config, E1304UnwrapInErrorPath};
pub use e1305_non_exhaustive_match::{E1305Config, E1305NonExhaustiveMatch};
pub use e1306_swallowed_errors::{E1306Config, E1306SwallowedErrors};
pub use e1307_string_error_type::{E1307Config, E1307StringErrorType};
pub use e1308_not_using_question_mark::{E1308Config, E1308NotUsingQuestionMark};
pub use e1309_panic_in_drop::{E1309Config, E1309PanicInDrop};
pub use e1310_error_context_loss::{E1310Config, E1310ErrorContextLoss};
