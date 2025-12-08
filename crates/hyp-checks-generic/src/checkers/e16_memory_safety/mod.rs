//! E16 - Memory safety checkers.

pub mod e1603_dangling_reference;
pub mod e1604_buffer_overflow;
pub mod e1605_rc_cycle;
pub mod e1606_unnecessary_clone;
pub mod e1607_forget_drop;
pub mod e1609_invalid_slice;
pub mod e1610_unaligned_deref;
pub mod e1611_consuming_self;
pub mod registry;

pub use e1603_dangling_reference::{E1603Config, E1603DanglingReference};
pub use e1604_buffer_overflow::{E1604BufferOverflow, E1604Config};
pub use e1605_rc_cycle::{E1605Config, E1605RcCycle};
pub use e1606_unnecessary_clone::{E1606Config, E1606UnnecessaryClone};
pub use e1607_forget_drop::{E1607Config, E1607ForgetDrop};
pub use e1609_invalid_slice::{E1609Config, E1609InvalidSlice};
pub use e1610_unaligned_deref::{E1610Config, E1610UnalignedDeref};
pub use e1611_consuming_self::{E1611Config, E1611ConsumingSelf};
