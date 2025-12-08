//! E17 - Performance checkers.

pub mod e1701_oversized_struct_by_value;
pub mod e1702_unnecessary_allocation;
pub mod e1703_string_concat_in_loop;
pub mod e1704_unnecessary_collect;
pub mod e1705_clone_in_hot_path;
pub mod e1706_non_tail_recursion;
pub mod e1707_unbounded_recursion;
pub mod e1708_inefficient_data_structure;
pub mod e1709_unnecessary_boxing;
pub mod e1710_large_stack_allocation;
pub mod e1712_expensive_ops_in_loop;
pub mod registry;

pub use e1701_oversized_struct_by_value::{E1701Config, E1701OversizedStructByValue};
pub use e1702_unnecessary_allocation::{E1702Config, E1702UnnecessaryAllocation};
pub use e1703_string_concat_in_loop::{E1703Config, E1703StringConcatInLoop};
pub use e1704_unnecessary_collect::{E1704Config, E1704UnnecessaryCollect};
pub use e1705_clone_in_hot_path::{E1705Config, E1705CloneInHotPath};
pub use e1706_non_tail_recursion::{E1706Config, E1706NonTailRecursion};
pub use e1707_unbounded_recursion::{E1707Config, E1707UnboundedRecursion};
pub use e1708_inefficient_data_structure::{E1708Config, E1708InefficientDataStructure};
pub use e1709_unnecessary_boxing::{E1709Config, E1709UnnecessaryBoxing};
pub use e1710_large_stack_allocation::{E1710Config, E1710LargeStackAllocation};
pub use e1712_expensive_ops_in_loop::{E1712Config, E1712ExpensiveOpsInLoop};
