//! E10 group checker registry.

use crate::{
    checker::Checker,
    checkers::e10_unsafe_code::{
        E1001Config, E1001DirectPanic, E1002Config, E1002DirectUnwrapExpect, E1003Config,
        E1003UnsafeCode, E1004Config, E1004TodoUnimplemented, E1005Config, E1005RawPointerDeref,
        E1006Config, E1006UnsafeTransmute, E1007Config, E1007NullPointerDeref, E1008Config,
        E1008UnsafeTraitImpl, E1009Config, E1009UnsafeCellMisuse, E1010Config, E1010MutableStatic,
        E1011Config, E1011UninitializedMemory, E1012Config, E1012UnsafeAutoTrait, E1013Config,
        E1013UnionFieldAccess, E1014Config, E1014RawPointerArithmetic, E1015Config, E1015UnwrapExpect,
        E1016Config, E1016MutexUnwrap, E1018Config,
        E1018ProhibitTransmute,
    },
    register_checker,
    registry::CheckerRegistration,
};

/// Get all E10 group checker registrations.
pub fn e10_registrations() -> Vec<CheckerRegistration> {
    vec![
        register_checker!(E1001DirectPanic, E1001Config),
        register_checker!(E1002DirectUnwrapExpect, E1002Config),
        register_checker!(E1003UnsafeCode, E1003Config),
        register_checker!(E1004TodoUnimplemented, E1004Config),
        register_checker!(E1005RawPointerDeref, E1005Config),
        register_checker!(E1006UnsafeTransmute, E1006Config),
        register_checker!(E1007NullPointerDeref, E1007Config),
        register_checker!(E1008UnsafeTraitImpl, E1008Config),
        register_checker!(E1009UnsafeCellMisuse, E1009Config),
        register_checker!(E1010MutableStatic, E1010Config),
        register_checker!(E1011UninitializedMemory, E1011Config),
        register_checker!(E1012UnsafeAutoTrait, E1012Config),
        register_checker!(E1013UnionFieldAccess, E1013Config),
        register_checker!(E1014RawPointerArithmetic, E1014Config),
        register_checker!(E1015UnwrapExpect, E1015Config),
        register_checker!(E1016MutexUnwrap, E1016Config),
        register_checker!(E1018ProhibitTransmute, E1018Config),
    ]
}
